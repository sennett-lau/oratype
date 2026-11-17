//! Plan `searxng-setup`: the Built-in web search provider. OraType downloads SearXNG with a
//! private Python into `<app data>/search/`, runs it on 127.0.0.1 only, and updates it.
//!
//! Nothing is downloaded until the user presses Set up. The parts:
//! - `uv` (MIT/Apache), one binary from its GitHub release, checked with the release's SHA-256.
//!   It downloads a standalone Python and installs SearXNG's libraries, so the Mac needs no
//!   Python, git or Docker.
//! - SearXNG (AGPL-3.0) from one commit of its `master` branch. It is fetched at the user's
//!   request and runs as its own program; it is never part of OraType's bundle.
//!
//! The server runs only while OraType runs: started by the first web search (or at launch when
//! Built-in is chosen) and stopped at quit, like the built-in AI server (`llm/builtin.rs`).
//! SearXNG does not log queries; its output goes to `search/searxng.log`.

#[cfg(not(target_os = "windows"))]
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tauri::{Emitter, Manager};

/// Progress events while setting up or updating (`SetupProgress`).
pub const SETUP_EVENT: &str = "search:setup_progress";
/// The port SearXNG gets when it is free.
pub const PREFERRED_PORT: u16 = 13721;
/// How long a search waits for a server that is starting.
pub const START_TIMEOUT: Duration = Duration::from_secs(15);
/// The Python version uv installs for SearXNG (it needs 3.11 or newer).
pub const PYTHON_VERSION: &str = "3.12";
const SEARXNG_REPO: &str = "searxng/searxng";
const UV_RELEASES: &str = "https://github.com/astral-sh/uv/releases/latest/download";

/// The files under `<app data>/search/`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchPaths {
    pub root: PathBuf,
}

impl SearchPaths {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }
    pub fn uv(&self) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.root.join("bin").join("uv.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.root.join("bin").join("uv")
        }
    }
    pub fn python_dir(&self) -> PathBuf {
        self.root.join("python")
    }
    pub fn cache_dir(&self) -> PathBuf {
        self.root.join("cache")
    }
    pub fn install_dir(&self, commit: &str) -> PathBuf {
        self.root.join(format!("searxng-{}", short_commit(commit)))
    }
    pub fn venv_python(&self, commit: &str) -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            self.install_dir(commit)
                .join(".venv")
                .join("Scripts")
                .join("python.exe")
        }
        #[cfg(not(target_os = "windows"))]
        {
            self.install_dir(commit).join(".venv/bin/python")
        }
    }
    pub fn settings(&self) -> PathBuf {
        self.root.join("settings.yml")
    }
    pub fn installed_file(&self) -> PathBuf {
        self.root.join("installed.json")
    }
    pub fn log(&self) -> PathBuf {
        self.root.join("searxng.log")
    }
    /// The running server's process id, so a OraType that crashed can stop it next time.
    pub fn pid(&self) -> PathBuf {
        self.root.join("searxng.pid")
    }
}

fn short_commit(commit: &str) -> &str {
    &commit[..commit.len().min(12)]
}

/// What is set up: SearXNG's commit, its date, and when OraType installed it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Installed {
    pub commit: String,
    /// The commit's date (ISO 8601), shown as the version.
    pub commit_date: String,
    /// Seconds since 1970 when it was set up.
    pub installed_at: u64,
    /// The random `secret_key` in settings.yml, kept across updates.
    pub secret_key: String,
}

/// What Settings shows about the installed version (no secret key).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledInfo {
    pub commit: String,
    pub commit_date: String,
    pub installed_at: u64,
}

impl From<Installed> for InstalledInfo {
    fn from(installed: Installed) -> Self {
        Self {
            commit: installed.commit,
            commit_date: installed.commit_date,
            installed_at: installed.installed_at,
        }
    }
}

/// One setup or update step, for the progress bar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SetupStep {
    DownloadingUv,
    InstallingPython,
    DownloadingSearxng,
    InstallingLibraries,
    Starting,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupProgress {
    pub step: SetupStep,
    /// Bytes downloaded so far and in all, for download steps.
    pub done: Option<u64>,
    pub total: Option<u64>,
}

/// Settings → Search: the Built-in provider's state.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct BuiltinSearchStatus {
    pub installed: Option<InstalledInfo>,
    pub running: bool,
    pub port: Option<u16>,
    /// A setup or update is in progress.
    pub busy: bool,
    /// The last progress step while busy, for a page opened after the setup started.
    pub progress: Option<SetupProgress>,
}

/// The latest SearXNG commit, for Check for updates.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCheck {
    pub latest_commit: String,
    pub latest_date: String,
    pub update_available: bool,
}

/// The search engines SearXNG asks. Its defaults (DuckDuckGo, Brave, Qwant, Google CSE, …)
/// answer a home connection with CAPTCHAs and "too many requests", so the general category
/// came back empty; these answer. Bing's web results are unrelated pages, so it is news only.
pub const ENGINES: &[&str] = &[
    "google",
    "yahoo",
    "startpage",
    "wikipedia",
    "bing news",
    "google news",
];

/// SearXNG's settings: this Mac only, JSON on, no limiter (it is not a public server), and
/// only `ENGINES` (`keep_only` drops the rest; the list below turns them on).
pub fn settings_yaml(port: u16, secret_key: &str) -> String {
    let keep_only = ENGINES.join(", ");
    let enabled: String = ENGINES
        .iter()
        .map(|engine| format!("  - name: {engine}\n    disabled: false\n"))
        .collect();
    format!(
        "# Written by OraType (plan `searxng-setup`). Changes are replaced on the next setup.\n\
         use_default_settings:\n  engines:\n    keep_only: [{keep_only}]\n\
         server:\n  bind_address: \"127.0.0.1\"\n  port: {port}\n  secret_key: \"{secret_key}\"\n  limiter: false\n  image_proxy: false\n\
         search:\n  formats: [html, json]\n\
         engines:\n{enabled}"
    )
}

/// The uv release file for this host platform.
pub fn uv_asset() -> &'static str {
    if cfg!(target_os = "windows") {
        if cfg!(target_arch = "aarch64") {
            "uv-aarch64-pc-windows-msvc.zip"
        } else {
            "uv-x86_64-pc-windows-msvc.zip"
        }
    } else if cfg!(target_arch = "x86_64") {
        "uv-x86_64-apple-darwin.tar.gz"
    } else {
        "uv-aarch64-apple-darwin.tar.gz"
    }
}

/// The hash in a `<file>.sha256` release file ("<hex>  <name>" or just "<hex>").
pub fn parse_sha256_file(text: &str) -> Option<String> {
    let hash = text.split_whitespace().next()?.to_ascii_lowercase();
    (hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())).then_some(hash)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `preferred` when it is free on 127.0.0.1, else a free port the system picks.
pub fn free_port(preferred: u16) -> u16 {
    if std::net::TcpListener::bind(("127.0.0.1", preferred)).is_ok() {
        return preferred;
    }
    std::net::TcpListener::bind(("127.0.0.1", 0))
        .and_then(|listener| listener.local_addr())
        .map(|addr| addr.port())
        .unwrap_or(preferred)
}

/// Normalizes a Windows/POSIX path and checks whether it names a Python interpreter inside
/// OraType's own `search_root` (preventing accidental termination of unrelated Python processes).
#[cfg(any(target_os = "windows", test))]
fn is_searxng_python_process_path(image_path: &str, search_root: &Path) -> bool {
    let normalize = |raw: &str| -> String {
        let slashed = raw.trim().replace('\\', "/");
        let without_verbatim = slashed.strip_prefix("//?/").unwrap_or(&slashed);
        without_verbatim.trim_end_matches('/').to_ascii_lowercase()
    };
    let norm_path = normalize(image_path);
    let norm_root = normalize(&search_root.to_string_lossy());
    if norm_path.is_empty() || norm_root.is_empty() {
        return false;
    }
    let Some(file_name) = norm_path.rsplit('/').next() else {
        return false;
    };
    let is_python = matches!(file_name, "python.exe" | "python3.exe");
    is_python && norm_path.starts_with(&format!("{norm_root}/"))
}

/// Stops a SearXNG a crashed OraType left behind (its id is in `paths.pid()`). Only a process
/// that really runs SearXNG from `paths` is stopped.
fn stop_stale_server(paths: &SearchPaths) {
    let pid_file = paths.pid();
    let Ok(text) = std::fs::read_to_string(&pid_file) else {
        return;
    };
    let _ = std::fs::remove_file(&pid_file);
    let Ok(pid) = text.trim().parse::<u32>() else {
        return;
    };
    if pid == 0 {
        return;
    }
    #[cfg(unix)]
    {
        let args = Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "args="])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_default();
        if args.contains("searx.webapp") {
            tracing::info!(
                "Built-in search: stopping a SearXNG left from an earlier run (pid {pid})"
            );
            let _ = Command::new("kill").arg(pid.to_string()).status();
        }
    }
    #[cfg(windows)]
    {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::Foundation::CloseHandle;
        use windows_sys::Win32::System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, TerminateProcess,
            PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_TERMINATE,
        };

        // SAFETY: `pid` was parsed as a non-zero Win32 PID; we open only with query + terminate
        // rights, check its full image path against OraType's private search folder, kill the
        // process tree if it matches, and unconditionally close the handle.
        unsafe {
            let handle = OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_TERMINATE,
                0,
                pid,
            );
            if handle.is_null() {
                return;
            }
            let mut buffer = [0u16; 1024];
            let mut size = buffer.len() as u32;
            let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
            if ok != 0 && size > 0 {
                let image_path = OsString::from_wide(&buffer[..size as usize])
                    .to_string_lossy()
                    .to_string();
                if is_searxng_python_process_path(&image_path, &paths.root) {
                    tracing::info!(
                        "Built-in search: stopping a SearXNG left from an earlier run (pid {pid})"
                    );
                    let mut taskkill = Command::new("taskkill");
                    crate::platform::configure_background_command(&mut taskkill);
                    let _ = taskkill
                        .args(["/PID", &pid.to_string(), "/T", "/F"])
                        .stdin(Stdio::null())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();
                    let _ = TerminateProcess(handle, 1);
                }
            }
            let _ = CloseHandle(handle);
        }
    }
}

fn new_secret() -> String {
    format!(
        "{}{}",
        uuid::Uuid::new_v4().simple(),
        uuid::Uuid::new_v4().simple()
    )
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Reads `installed.json`, or None when Built-in is not set up.
pub fn read_installed(paths: &SearchPaths) -> Option<Installed> {
    let text = std::fs::read_to_string(paths.installed_file()).ok()?;
    let installed: Installed = serde_json::from_str(&text).ok()?;
    paths
        .venv_python(&installed.commit)
        .exists()
        .then_some(installed)
}

fn write_installed(paths: &SearchPaths, installed: &Installed) -> Result<(), String> {
    let text = serde_json::to_string_pretty(installed).map_err(|e| e.to_string())?;
    std::fs::write(paths.installed_file(), text).map_err(|e| format!("save installed.json: {e}"))
}

/// Runs a setup command (uv, tar) and returns its error output when it fails.
fn run(command: &mut Command, what: &str) -> Result<(), String> {
    crate::platform::configure_background_command(command);
    let output = command
        .stdin(Stdio::null())
        .output()
        .map_err(|e| format!("{what}: {e}"))?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    let last: Vec<&str> = stderr.lines().rev().take(6).collect();
    Err(format!(
        "{what} failed: {}",
        last.into_iter().rev().collect::<Vec<_>>().join(" | ")
    ))
}

#[cfg(any(target_os = "windows", test))]
fn is_windows_safe_component(name: &str) -> bool {
    if name.is_empty() || name.ends_with('.') || name.ends_with(' ') {
        return false;
    }
    if name.chars().any(|c| {
        matches!(c, '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*') || (c as u32) < 0x20
    }) {
        return false;
    }
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    !matches!(
        stem.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    )
}

#[cfg(any(target_os = "windows", test))]
pub(crate) fn unpack_searxng_tar_gz(archive_bytes: &[u8], dest: &Path) -> Result<(), String> {
    use std::path::Component;

    let gz = flate2::read::GzDecoder::new(archive_bytes);
    let mut archive = tar::Archive::new(gz);
    let entries = archive
        .entries()
        .map_err(|e| format!("unpack SearXNG: read archive: {e}"))?;

    for entry_res in entries {
        let mut entry = entry_res.map_err(|e| format!("unpack SearXNG: entry: {e}"))?;
        let entry_type = entry.header().entry_type();
        if matches!(
            entry_type,
            tar::EntryType::XGlobalHeader
                | tar::EntryType::XHeader
                | tar::EntryType::GNULongName
                | tar::EntryType::GNULongLink
        ) {
            continue;
        }

        let raw_path = entry
            .path()
            .map_err(|e| format!("unpack SearXNG: invalid entry path: {e}"))?
            .into_owned();

        let mut normal_parts = Vec::new();
        for comp in raw_path.components() {
            match comp {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(format!(
                        "unpack SearXNG: unsafe path in archive: {}",
                        raw_path.display()
                    ));
                }
                Component::CurDir => {}
                Component::Normal(part) => normal_parts.push(part.to_os_string()),
            }
        }

        // Mirror `--strip-components 1` from GitHub's `<repo>-<commit>/...` tarball layout.
        if normal_parts.len() <= 1 {
            continue;
        }
        let stripped = &normal_parts[1..];

        // Skip symlinks/hardlinks (unprivileged Windows lacks symlink privileges) and
        // `utils/templates/...` server templates (Apache/nginx sample configs with colons/symlinks).
        if entry_type.is_symlink() || entry_type.is_hard_link() {
            continue;
        }
        if stripped.len() >= 2 && stripped[0] == "utils" && stripped[1] == "templates" {
            continue;
        }

        if !stripped
            .iter()
            .all(|part| part.to_str().is_some_and(is_windows_safe_component))
        {
            tracing::debug!(
                "Skipping SearXNG archive entry with Windows-incompatible name: {}",
                raw_path.display()
            );
            continue;
        }

        let mut out_path = dest.to_path_buf();
        for part in stripped {
            out_path.push(part);
        }

        match entry_type {
            tar::EntryType::Directory => {
                std::fs::create_dir_all(&out_path)
                    .map_err(|e| format!("unpack SearXNG: mkdir {}: {e}", out_path.display()))?;
            }
            tar::EntryType::Regular | tar::EntryType::Continuous => {
                if let Some(parent) = out_path.parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("unpack SearXNG: mkdir {}: {e}", parent.display()))?;
                }
                let mut out_file = std::fs::File::create(&out_path)
                    .map_err(|e| format!("unpack SearXNG: create {}: {e}", out_path.display()))?;
                std::io::copy(&mut entry, &mut out_file)
                    .map_err(|e| format!("unpack SearXNG: write {}: {e}", out_path.display()))?;
            }
            _ => {}
        }
    }

    Ok(())
}

fn uv_command(paths: &SearchPaths) -> Command {
    let mut command = Command::new(paths.uv());
    command
        .env("UV_PYTHON_INSTALL_DIR", paths.python_dir())
        .env("UV_CACHE_DIR", paths.cache_dir())
        .env("UV_NO_CONFIG", "1")
        // Only uv's own standalone Python, never one found on the Mac.
        .env("UV_PYTHON_PREFERENCE", "only-managed");
    #[cfg(target_os = "windows")]
    command
        .env("PYTHONUTF8", "1")
        .env("PYTHONIOENCODING", "utf-8");
    command
}

async fn blocking<T: Send + 'static>(
    job: impl FnOnce() -> Result<T, String> + Send + 'static,
) -> Result<T, String> {
    tokio::task::spawn_blocking(job)
        .await
        .map_err(|e| format!("setup task: {e}"))?
}

/// Downloads `url` into memory, reporting progress under `step`.
async fn download(
    client: &reqwest::Client,
    url: &str,
    step: SetupStep,
    progress: &(dyn Fn(SetupProgress) + Send + Sync),
) -> Result<Vec<u8>, String> {
    let mut response = client
        .get(url)
        .header("User-Agent", "OraType")
        .timeout(Duration::from_secs(600))
        .send()
        .await
        .map_err(|e| format!("download {url}: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "download {url}: HTTP {}",
            response.status().as_u16()
        ));
    }
    let total = response.content_length();
    let mut bytes = Vec::with_capacity(total.unwrap_or(0) as usize);
    let mut last_report = Instant::now();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("download {url}: {e}"))?
    {
        bytes.extend_from_slice(&chunk);
        if last_report.elapsed() > Duration::from_millis(200) {
            last_report = Instant::now();
            progress(SetupProgress {
                step,
                done: Some(bytes.len() as u64),
                total,
            });
        }
    }
    Ok(bytes)
}

/// The latest commit on SearXNG's `master` and its date.
pub async fn latest_commit(client: &reqwest::Client) -> Result<(String, String), String> {
    let url = format!("https://api.github.com/repos/{SEARXNG_REPO}/commits/master");
    let value: serde_json::Value = client
        .get(&url)
        .header("User-Agent", "OraType")
        .header("Accept", "application/vnd.github+json")
        .timeout(Duration::from_secs(20))
        .send()
        .await
        .map_err(|e| format!("ask GitHub for SearXNG's latest version: {e}"))?
        .error_for_status()
        .map_err(|e| format!("ask GitHub for SearXNG's latest version: {e}"))?
        .json()
        .await
        .map_err(|e| format!("read GitHub's answer: {e}"))?;
    parse_commit(&value).ok_or_else(|| "GitHub's answer had no commit".to_string())
}

/// `sha` and `commit.committer.date` from GitHub's commit JSON.
pub fn parse_commit(value: &serde_json::Value) -> Option<(String, String)> {
    let sha = value.get("sha")?.as_str()?.to_string();
    let date = value
        .pointer("/commit/committer/date")
        .and_then(|d| d.as_str())
        .unwrap_or_default()
        .to_string();
    (sha.len() >= 7).then_some((sha, date))
}

/// The running SearXNG, if any.
struct Running {
    child: Child,
    port: u16,
    commit: String,
    /// `/healthz` has answered, so the server takes searches.
    ready: bool,
}

/// Manages the Built-in SearXNG: setup, update, remove, and its process.
pub struct SearchServer {
    paths: Mutex<Option<SearchPaths>>,
    running: Mutex<Option<Running>>,
    busy: AtomicBool,
    /// The last progress step of the running setup or update (see `BuiltinSearchStatus`).
    last_progress: Mutex<Option<SetupProgress>>,
    /// Serialises starts, setups, updates and removals, so two searches do not start two
    /// servers and a search never restarts the server an update or removal is replacing.
    start_lock: tokio::sync::Mutex<()>,
}

static SERVER: OnceLock<SearchServer> = OnceLock::new();

pub fn server() -> &'static SearchServer {
    SERVER.get_or_init(|| SearchServer {
        paths: Mutex::new(None),
        running: Mutex::new(None),
        busy: AtomicBool::new(false),
        last_progress: Mutex::new(None),
        start_lock: tokio::sync::Mutex::new(()),
    })
}

/// Clears `busy` when a setup or update ends, however it ends.
struct BusyGuard<'a>(&'a AtomicBool);
impl Drop for BusyGuard<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::SeqCst);
    }
}

impl SearchServer {
    /// Called at startup with `<app data>/search`.
    pub fn set_root(&self, root: PathBuf) {
        let paths = SearchPaths::new(root);
        stop_stale_server(&paths);
        *self.paths.lock().unwrap_or_else(|e| e.into_inner()) = Some(paths);
    }

    /// Remembers a progress step, so `status` can show it to a page opened later.
    pub fn note_progress(&self, progress: SetupProgress) {
        *self.last_progress.lock().unwrap_or_else(|e| e.into_inner()) = Some(progress);
    }

    fn paths(&self) -> Result<SearchPaths, String> {
        self.paths
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(|| "search folder not set".to_string())
    }

    pub fn is_installed(&self) -> bool {
        self.paths().ok().and_then(|p| read_installed(&p)).is_some()
    }

    fn begin(&self) -> Result<BusyGuard<'_>, String> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err("A setup or update is already running.".to_string());
        }
        *self.last_progress.lock().unwrap_or_else(|e| e.into_inner()) = None;
        Ok(BusyGuard(&self.busy))
    }

    pub fn status(&self) -> BuiltinSearchStatus {
        let installed = self.paths().ok().and_then(|p| read_installed(&p));
        let mut running = self.running.lock().unwrap_or_else(|e| e.into_inner());
        let alive = match running.as_mut() {
            Some(server) => matches!(server.child.try_wait(), Ok(None)),
            None => false,
        };
        if !alive {
            *running = None;
        }
        let busy = self.busy.load(Ordering::SeqCst);
        BuiltinSearchStatus {
            installed: installed.map(InstalledInfo::from),
            running: alive,
            port: running.as_ref().map(|server| server.port),
            busy,
            progress: if busy {
                self.last_progress
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .clone()
            } else {
                None
            },
        }
    }

    /// The address of a running server, starting it when needed (plan: lazy start).
    pub async fn ensure_running(&self, client: &reqwest::Client) -> Result<String, String> {
        let _start = self.start_lock.lock().await;
        let status = self.status();
        let port = match (status.running, status.port) {
            // Started by an earlier call that was cancelled (Escape) before it answered.
            (true, Some(port)) if !self.is_ready() => port,
            (true, Some(port)) => return Ok(base_url(port)),
            _ => {
                let paths = self.paths()?;
                let installed = read_installed(&paths).ok_or("Built-in search is not set up")?;
                self.start(&paths, &installed)?
            }
        };
        wait_until_ready(client, port, START_TIMEOUT).await?;
        self.mark_ready();
        Ok(base_url(port))
    }

    fn is_ready(&self) -> bool {
        self.running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
            .is_some_and(|server| server.ready)
    }

    fn mark_ready(&self) {
        if let Some(server) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_mut()
        {
            server.ready = true;
        }
    }

    /// Starts `installed` on a free port (writing settings.yml). Returns the port.
    fn start(&self, paths: &SearchPaths, installed: &Installed) -> Result<u16, String> {
        self.stop();
        let port = free_port(PREFERRED_PORT);
        std::fs::write(paths.settings(), settings_yaml(port, &installed.secret_key))
            .map_err(|e| format!("write settings.yml: {e}"))?;
        let log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(paths.log())
            .map_err(|e| format!("open searxng.log: {e}"))?;
        let mut command = Command::new(paths.venv_python(&installed.commit));
        crate::platform::configure_background_command(&mut command);
        #[cfg(target_os = "windows")]
        command
            .env("PYTHONUTF8", "1")
            .env("PYTHONIOENCODING", "utf-8");
        // TODO(windows): verify `127.0.0.1:{port}` does not trigger Windows Defender Firewall prompts on fresh machines.
        let child = command
            .args(["-m", "searx.webapp"])
            .current_dir(paths.install_dir(&installed.commit))
            .env("SEARXNG_SETTINGS_PATH", paths.settings())
            .stdin(Stdio::null())
            .stdout(log.try_clone().map_err(|e| e.to_string())?)
            .stderr(log)
            .spawn()
            .map_err(|e| format!("start SearXNG: {e}"))?;
        tracing::info!(
            "Built-in search: SearXNG {} started on 127.0.0.1:{port} (pid {})",
            short_commit(&installed.commit),
            child.id()
        );
        let _ = std::fs::write(paths.pid(), child.id().to_string());
        *self.running.lock().unwrap_or_else(|e| e.into_inner()) = Some(Running {
            child,
            port,
            commit: installed.commit.clone(),
            ready: false,
        });
        Ok(port)
    }

    /// Stops the server (at quit, before an update or removal).
    pub fn stop(&self) {
        if let Some(mut server) = self
            .running
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .take()
        {
            #[cfg(windows)]
            {
                // Kill the full child process tree so uv's venv trampoline and its managed
                // python.exe worker both release the bound loopback port immediately.
                let mut taskkill = Command::new("taskkill");
                crate::platform::configure_background_command(&mut taskkill);
                let _ = taskkill
                    .args(["/PID", &server.child.id().to_string(), "/T", "/F"])
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
            }
            let _ = server.child.kill();
            let _ = server.child.wait();
            if let Ok(paths) = self.paths() {
                let _ = std::fs::remove_file(paths.pid());
            }
            tracing::info!(
                "Built-in search: SearXNG {} stopped",
                short_commit(&server.commit)
            );
        }
    }

    /// Set up: uv, Python, the latest SearXNG, its libraries; then start and test it.
    pub async fn install(
        &self,
        client: &reqwest::Client,
        progress: &(dyn Fn(SetupProgress) + Send + Sync),
    ) -> Result<Installed, String> {
        let _start = self.start_lock.lock().await;
        let _busy = self.begin()?;
        let paths = self.paths()?;
        std::fs::create_dir_all(&paths.root).map_err(|e| format!("create search folder: {e}"))?;
        let started = Instant::now();
        ensure_uv(client, &paths, progress).await?;
        progress(step(SetupStep::InstallingPython));
        {
            let paths = paths.clone();
            blocking(move || {
                run(
                    uv_command(&paths).args(["python", "install", "--no-bin", PYTHON_VERSION]),
                    "install Python",
                )
            })
            .await?;
        }
        let (commit, commit_date) = latest_commit(client).await?;
        let secret_key = read_installed(&paths)
            .map(|old| old.secret_key)
            .unwrap_or_else(new_secret);
        let installed = Installed {
            commit,
            commit_date,
            installed_at: now_secs(),
            secret_key,
        };
        install_searxng(client, &paths, &installed, progress).await?;
        progress(step(SetupStep::Starting));
        let old = read_installed(&paths);
        let port = self.start(&paths, &installed)?;
        if let Err(error) = wait_until_ready(client, port, START_TIMEOUT).await {
            self.stop();
            return Err(error);
        }
        write_installed(&paths, &installed)?;
        remove_old_installs(&paths, &installed.commit, old.as_ref());
        let _ = std::fs::remove_dir_all(paths.cache_dir());
        tracing::info!(
            "Built-in search: set up SearXNG {} in {:.0?}",
            short_commit(&installed.commit),
            started.elapsed()
        );
        progress(step(SetupStep::Done));
        Ok(installed)
    }

    /// Compares the installed commit with the latest one.
    pub async fn check_update(&self, client: &reqwest::Client) -> Result<UpdateCheck, String> {
        let installed = read_installed(&self.paths()?);
        let (latest_commit, latest_date) = latest_commit(client).await?;
        Ok(UpdateCheck {
            update_available: installed.is_none_or(|i| i.commit != latest_commit),
            latest_commit,
            latest_date,
        })
    }

    /// Update: installs the latest commit beside the old one and switches only when it works.
    pub async fn update(
        &self,
        client: &reqwest::Client,
        progress: &(dyn Fn(SetupProgress) + Send + Sync),
    ) -> Result<Installed, String> {
        let paths = self.paths()?;
        let Some(old) = read_installed(&paths) else {
            return self.install(client, progress).await;
        };
        let _start = self.start_lock.lock().await;
        let _busy = self.begin()?;
        let (commit, commit_date) = latest_commit(client).await?;
        if commit == old.commit {
            progress(step(SetupStep::Done));
            return Ok(old);
        }
        ensure_uv(client, &paths, progress).await?;
        let new = Installed {
            commit,
            commit_date,
            installed_at: now_secs(),
            secret_key: old.secret_key.clone(),
        };
        install_searxng(client, &paths, &new, progress).await?;
        progress(step(SetupStep::Starting));
        let was_running = self.status().running;
        let port = self.start(&paths, &new)?;
        if let Err(error) = wait_until_ready(client, port, START_TIMEOUT).await {
            // The new one does not work: keep the old one.
            self.stop();
            let _ = std::fs::remove_dir_all(paths.install_dir(&new.commit));
            if was_running {
                let _ = self.start(&paths, &old);
            }
            return Err(format!("{error} (kept the installed version)"));
        }
        write_installed(&paths, &new)?;
        remove_old_installs(&paths, &new.commit, Some(&old));
        let _ = std::fs::remove_dir_all(paths.cache_dir());
        tracing::info!(
            "Built-in search: updated SearXNG {} → {}",
            short_commit(&old.commit),
            short_commit(&new.commit)
        );
        progress(step(SetupStep::Done));
        Ok(new)
    }

    /// Stops the server and deletes everything under `search/`.
    pub async fn remove(&self) -> Result<(), String> {
        let _start = self.start_lock.lock().await;
        let _busy = self.begin()?;
        self.stop();
        let paths = self.paths()?;
        // A few hundred MB of files: off the main thread, so Settings keeps drawing.
        blocking(move || {
            if paths.root.exists() {
                std::fs::remove_dir_all(&paths.root)
                    .map_err(|e| format!("remove search folder: {e}"))?;
            }
            Ok(())
        })
        .await?;
        tracing::info!("Built-in search: removed");
        Ok(())
    }
}

fn step(step: SetupStep) -> SetupProgress {
    SetupProgress {
        step,
        done: None,
        total: None,
    }
}

pub fn base_url(port: u16) -> String {
    format!("http://127.0.0.1:{port}")
}

/// Waits until SearXNG answers `/healthz`.
async fn wait_until_ready(
    client: &reqwest::Client,
    port: u16,
    timeout: Duration,
) -> Result<(), String> {
    let url = format!("{}/healthz", base_url(port));
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if let Ok(response) = client
            .get(&url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            if response.status().is_success() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    Err(format!(
        "SearXNG did not start within {} s (see searxng.log)",
        timeout.as_secs()
    ))
}

/// True if a zip entry path stays strictly inside the extraction folder (blocks zip-slip `..`,
/// leading `/` or `\`, drive prefixes like `C:\...`, and NUL bytes).
pub fn is_safe_archive_entry_path(raw_name: &str) -> bool {
    if raw_name.is_empty()
        || raw_name.contains('\0')
        || raw_name.contains(':')
        || raw_name.starts_with(['/', '\\'])
    {
        return false;
    }
    raw_name.split(['/', '\\']).all(|segment| segment != "..")
}

/// Safely extracts `uv.exe` from a Windows `uv-*.zip` release archive into `bin_dir`.
/// Validates every entry in the archive before writing any byte to disk.
pub fn extract_uv_zip(archive: &[u8], bin_dir: &Path) -> Result<PathBuf, String> {
    let cursor = std::io::Cursor::new(archive);
    let mut zip = zip::ZipArchive::new(cursor).map_err(|e| format!("open uv zip: {e}"))?;
    let mut uv_index = None;
    for index in 0..zip.len() {
        let entry = zip
            .by_index(index)
            .map_err(|e| format!("read uv zip entry {index}: {e}"))?;
        let raw_name = entry.name();
        if !is_safe_archive_entry_path(raw_name)
            || entry.enclosed_name().is_none()
            || entry.is_symlink()
        {
            return Err(format!("unsafe path in uv archive: {raw_name}"));
        }
        if !entry.is_dir()
            && raw_name
                .rsplit(['/', '\\'])
                .next()
                .is_some_and(|leaf| leaf.eq_ignore_ascii_case("uv.exe"))
        {
            uv_index.get_or_insert(index);
        }
    }
    let Some(uv_index) = uv_index else {
        return Err("uv's archive had no uv.exe".to_string());
    };
    std::fs::create_dir_all(bin_dir).map_err(|e| format!("create bin folder: {e}"))?;
    let mut entry = zip
        .by_index(uv_index)
        .map_err(|e| format!("read uv.exe entry: {e}"))?;
    let dest = bin_dir.join("uv.exe");
    let mut out = std::fs::File::create(&dest).map_err(|e| format!("save uv.exe: {e}"))?;
    std::io::copy(&mut entry, &mut out).map_err(|e| format!("write uv.exe: {e}"))?;
    Ok(dest)
}

/// Downloads uv once, checked against the release's SHA-256.
async fn ensure_uv(
    client: &reqwest::Client,
    paths: &SearchPaths,
    progress: &(dyn Fn(SetupProgress) + Send + Sync),
) -> Result<(), String> {
    if paths.uv().exists() {
        // A download or unpack that stopped half way leaves a uv that does not run.
        let runs = run(Command::new(paths.uv()).arg("--version"), "run uv").is_ok();
        if runs {
            return Ok(());
        }
        tracing::warn!("Built-in search: the saved uv does not run; downloading it again");
        let _ = std::fs::remove_file(paths.uv());
    }
    progress(step(SetupStep::DownloadingUv));
    let asset = uv_asset();
    let hash_text = client
        .get(format!("{UV_RELEASES}/{asset}.sha256"))
        .header("User-Agent", "OraType")
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("download uv's checksum: {e}"))?
        .text()
        .await
        .map_err(|e| format!("download uv's checksum: {e}"))?;
    let expected = parse_sha256_file(&hash_text).ok_or("uv's checksum file is not valid")?;
    let archive = download(
        client,
        &format!("{UV_RELEASES}/{asset}"),
        SetupStep::DownloadingUv,
        progress,
    )
    .await?;
    let actual = sha256_hex(&archive);
    if actual != expected {
        return Err(format!(
            "uv's download does not match its checksum ({actual} ≠ {expected})"
        ));
    }
    let paths = paths.clone();
    blocking(move || {
        let bin = paths
            .uv()
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        std::fs::create_dir_all(&bin).map_err(|e| e.to_string())?;
        #[cfg(target_os = "windows")]
        {
            extract_uv_zip(&archive, &bin)?;
            if !paths.uv().exists() {
                return Err("uv's archive had no uv.exe".to_string());
            }
            Ok(())
        }
        #[cfg(not(target_os = "windows"))]
        {
            let tarball = paths.root.join("uv.tar.gz");
            std::fs::File::create(&tarball)
                .and_then(|mut f| f.write_all(&archive))
                .map_err(|e| format!("save uv: {e}"))?;
            // The archive holds `uv-<target>/uv` and `uvx`; keep only uv.
            let result = run(
                Command::new("tar")
                    .arg("-xzf")
                    .arg(&tarball)
                    .arg("-C")
                    .arg(&bin)
                    .args(["--strip-components", "1"]),
                "unpack uv",
            );
            let _ = std::fs::remove_file(&tarball);
            let _ = std::fs::remove_file(bin.join("uvx"));
            result?;
            if !paths.uv().exists() {
                return Err("uv's archive had no uv".to_string());
            }
            Ok(())
        }
    })
    .await
}

/// Downloads SearXNG at `installed.commit` into its folder and installs its libraries.
async fn install_searxng(
    client: &reqwest::Client,
    paths: &SearchPaths,
    installed: &Installed,
    progress: &(dyn Fn(SetupProgress) + Send + Sync),
) -> Result<(), String> {
    progress(step(SetupStep::DownloadingSearxng));
    let archive = download(
        client,
        &format!(
            "https://github.com/{SEARXNG_REPO}/archive/{}.tar.gz",
            installed.commit
        ),
        SetupStep::DownloadingSearxng,
        progress,
    )
    .await?;
    progress(step(SetupStep::InstallingLibraries));
    let paths = paths.clone();
    let commit = installed.commit.clone();
    blocking(move || {
        let dir = paths.install_dir(&commit);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        #[cfg(target_os = "windows")]
        let unpacked = unpack_searxng_tar_gz(&archive, &dir);
        #[cfg(not(target_os = "windows"))]
        let unpacked = {
            let tarball = paths.root.join("searxng.tar.gz");
            std::fs::File::create(&tarball)
                .and_then(|mut f| f.write_all(&archive))
                .map_err(|e| format!("save SearXNG: {e}"))?;
            let unpacked = run(
                Command::new("tar")
                    .arg("-xzf")
                    .arg(&tarball)
                    .arg("-C")
                    .arg(&dir)
                    .args(["--strip-components", "1"]),
                "unpack SearXNG",
            );
            let _ = std::fs::remove_file(&tarball);
            unpacked
        };
        let installed = unpacked
            .and_then(|()| {
                run(
                    uv_command(&paths)
                        .args(["venv", "--python", PYTHON_VERSION])
                        .arg(dir.join(".venv")),
                    "create SearXNG's Python environment",
                )
            })
            .and_then(|()| {
                // SearXNG's own install steps: its build tools first, then SearXNG itself.
                run(
                    uv_command(&paths)
                        .args(["pip", "install", "--python"])
                        .arg(paths.venv_python(&commit))
                        .args([
                            "pip",
                            "setuptools",
                            "wheel",
                            "pyyaml",
                            "msgspec",
                            "typing_extensions",
                        ]),
                    "install SearXNG's build tools",
                )
            })
            .and_then(|()| {
                run(
                    uv_command(&paths)
                        .args(["pip", "install", "--no-build-isolation", "--python"])
                        .arg(paths.venv_python(&commit))
                        .arg("-e")
                        .arg(&dir),
                    "install SearXNG",
                )
            });
        if installed.is_err() {
            let _ = std::fs::remove_dir_all(&dir);
        }
        installed
    })
    .await
}

/// Deletes install folders other than `keep` (the previous one once the new one works).
fn remove_old_installs(paths: &SearchPaths, keep: &str, _old: Option<&Installed>) {
    let keep_dir = paths.install_dir(keep);
    if let Ok(entries) = std::fs::read_dir(&paths.root) {
        for entry in entries.flatten() {
            let path = entry.path();
            let is_install = path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("searxng-"));
            if is_install && path.is_dir() && path != keep_dir {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }
}

/// Emits progress to every window, for Settings and onboarding.
fn emitter(app: &tauri::AppHandle) -> impl Fn(SetupProgress) + Send + Sync {
    let app = app.clone();
    move |progress| {
        server().note_progress(progress.clone());
        let _ = app.emit(SETUP_EVENT, progress);
    }
}

/// Called at startup: the search folder, and a start in the background when Built-in is chosen.
pub fn init(app: &tauri::AppHandle, config: &crate::storage::AppConfig) {
    if let Ok(data_dir) = app.path().app_data_dir() {
        server().set_root(data_dir.join("search"));
    }
    if config.web_search.provider == crate::web_search::SearchProviderKind::Builtin
        && server().is_installed()
    {
        let client = app.state::<reqwest::Client>().inner().clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = server().ensure_running(&client).await {
                tracing::warn!("Built-in search: could not start at launch: {error}");
            }
        });
    }
}

#[tauri::command]
pub fn builtin_search_status() -> BuiltinSearchStatus {
    server().status()
}

#[tauri::command]
pub async fn install_builtin_search(
    app: tauri::AppHandle,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<BuiltinSearchStatus, String> {
    let progress = emitter(&app);
    server()
        .install(client.inner(), &progress)
        .await
        .inspect_err(|error| tracing::warn!("Built-in search: setup failed: {error}"))?;
    Ok(server().status())
}

#[tauri::command]
pub async fn check_builtin_search_update(
    client: tauri::State<'_, reqwest::Client>,
) -> Result<UpdateCheck, String> {
    server().check_update(client.inner()).await
}

#[tauri::command]
pub async fn update_builtin_search(
    app: tauri::AppHandle,
    client: tauri::State<'_, reqwest::Client>,
) -> Result<BuiltinSearchStatus, String> {
    let progress = emitter(&app);
    server()
        .update(client.inner(), &progress)
        .await
        .inspect_err(|error| tracing::warn!("Built-in search: update failed: {error}"))?;
    Ok(server().status())
}

#[tauri::command]
pub async fn remove_builtin_search() -> Result<BuiltinSearchStatus, String> {
    server().remove().await?;
    Ok(server().status())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_listen_on_this_mac_only_with_json_on() {
        let yaml = settings_yaml(8890, "abc123");
        assert!(yaml.contains("bind_address: \"127.0.0.1\""));
        assert!(yaml.contains("port: 8890"));
        assert!(yaml.contains("secret_key: \"abc123\""));
        assert!(yaml.contains("limiter: false"));
        assert!(yaml.contains("formats: [html, json]"));
        assert!(yaml
            .contains("keep_only: [google, yahoo, startpage, wikipedia, bing news, google news]"));
        assert!(yaml.contains("  - name: google news\n    disabled: false\n"));
        assert!(!yaml.contains("duckduckgo"));
    }

    #[test]
    fn checksum_files_are_read_in_both_forms() {
        let hash = "a".repeat(64);
        assert_eq!(
            parse_sha256_file(&format!("{hash}  uv-aarch64-apple-darwin.tar.gz\n")),
            Some(hash.clone())
        );
        assert_eq!(parse_sha256_file(&hash.to_uppercase()), Some(hash));
        assert_eq!(parse_sha256_file("not a hash"), None);
        assert_eq!(parse_sha256_file(""), None);
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn github_commit_json_gives_the_commit_and_its_date() {
        let value = serde_json::json!({
            "sha": "0123456789abcdef0123456789abcdef01234567",
            "commit": { "committer": { "date": "2026-09-28T10:00:00Z" } }
        });
        assert_eq!(
            parse_commit(&value),
            Some((
                "0123456789abcdef0123456789abcdef01234567".to_string(),
                "2026-09-28T10:00:00Z".to_string()
            ))
        );
        assert_eq!(
            parse_commit(&serde_json::json!({ "message": "rate limited" })),
            None
        );
    }

    #[test]
    fn install_folders_are_named_by_the_short_commit() {
        let root = if cfg!(target_os = "windows") {
            r"C:\Temp\search"
        } else {
            "/tmp/search"
        };
        let paths = SearchPaths::new(PathBuf::from(root));
        let commit = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(
            paths.install_dir(commit),
            PathBuf::from(root).join("searxng-0123456789ab")
        );
        if cfg!(target_os = "windows") {
            assert_eq!(
                paths.venv_python(commit),
                PathBuf::from(root)
                    .join("searxng-0123456789ab")
                    .join(".venv")
                    .join("Scripts")
                    .join("python.exe")
            );
            assert_eq!(paths.uv(), PathBuf::from(root).join("bin").join("uv.exe"));
        } else {
            assert_eq!(
                paths.venv_python(commit),
                PathBuf::from("/tmp/search/searxng-0123456789ab/.venv/bin/python")
            );
            assert_eq!(paths.uv(), PathBuf::from("/tmp/search/bin/uv"));
        }
    }

    fn write_test_zip(zip_path: &Path, entries: &[(&str, &[u8])]) {
        let file = std::fs::File::create(zip_path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, body) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(body).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn uv_zip_extraction_unpacks_uv_exe_and_skips_other_files() {
        let root = std::env::temp_dir().join(format!("oratype-uv-zip-{}", uuid::Uuid::new_v4()));
        let bin = root.join("bin");
        let root_zip = root.join("uv-root.zip");
        std::fs::create_dir_all(&root).unwrap();

        write_test_zip(
            &root_zip,
            &[("uv.exe", b"MZ-uv-root"), ("uvx.exe", b"MZ-uvx-ignored")],
        );
        let archive = std::fs::read(&root_zip).unwrap();
        let out = extract_uv_zip(&archive, &bin).unwrap();
        assert_eq!(out, bin.join("uv.exe"));
        assert_eq!(std::fs::read(&out).unwrap(), b"MZ-uv-root");
        assert!(!bin.join("uvx.exe").exists());

        // Also handle archives that nest binaries under a release directory.
        let nested_zip = root.join("uv-nested.zip");
        let nested_bin = root.join("bin-nested");
        write_test_zip(
            &nested_zip,
            &[
                ("uv-x86_64-pc-windows-msvc/uvx.exe", b"ignore"),
                ("uv-x86_64-pc-windows-msvc/UV.EXE", b"MZ-uv-nested"),
            ],
        );
        let nested_archive = std::fs::read(&nested_zip).unwrap();
        let nested_out = extract_uv_zip(&nested_archive, &nested_bin).unwrap();
        assert_eq!(std::fs::read(nested_out).unwrap(), b"MZ-uv-nested");

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn uv_zip_extraction_rejects_zip_slip_traversal_entries() {
        let root =
            std::env::temp_dir().join(format!("oratype-uv-zipslip-{}", uuid::Uuid::new_v4()));
        let bin = root.join("bin");
        std::fs::create_dir_all(&root).unwrap();

        for (label, evil_name) in [
            ("dotdot", "../evil.exe"),
            ("backslash_dotdot", r"sub\..\..\evil.dll"),
            ("drive_prefix", r"C:\evil.exe"),
            ("leading_slash", "/evil.exe"),
        ] {
            assert!(!is_safe_archive_entry_path(evil_name), "{label}");
            let zip_path = root.join(format!("{label}.zip"));
            write_test_zip(
                &zip_path,
                &[("uv.exe", b"MZ-valid"), (evil_name, b"MZ-evil")],
            );
            let archive = std::fs::read(&zip_path).unwrap();
            let err = extract_uv_zip(&archive, &bin).unwrap_err();
            assert!(
                err.contains("unsafe path"),
                "{label}: expected unsafe path error, got: {err}"
            );
            assert!(
                !bin.join("uv.exe").exists(),
                "{label}: uv.exe must not be written when archive fails validation"
            );
            assert!(!root.join("evil.exe").exists(), "{label}: escaped file");
            assert!(!root.join("evil.dll").exists(), "{label}: escaped file");
        }

        std::fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn searxng_python_process_path_matches_only_private_search_root() {
        let root = Path::new(r"C:\Users\Joe\AppData\Roaming\com.oratype.app\search");
        assert!(is_searxng_python_process_path(
            r"\\?\C:\Users\Joe\AppData\Roaming\com.oratype.app\search\searxng-0123456789ab\.venv\Scripts\python.exe",
            root
        ));
        assert!(is_searxng_python_process_path(
            r"C:\Users\Joe\AppData\Roaming\com.oratype.app\search\python\cpython-3.12.1\python.exe",
            root
        ));
        assert!(!is_searxng_python_process_path(
            r"C:\Users\Joe\AppData\Roaming\com.oratype.app\search2\python.exe",
            root
        ));
        assert!(!is_searxng_python_process_path(
            r"C:\Python312\python.exe",
            root
        ));
        assert!(!is_searxng_python_process_path(
            r"C:\Users\Joe\AppData\Roaming\com.oratype.app\search\bin\uv.exe",
            root
        ));
    }

    #[test]
    fn installed_needs_its_python_to_count() {
        let root = std::env::temp_dir().join(format!("oratype-search-{}", uuid::Uuid::new_v4()));
        let paths = SearchPaths::new(root.clone());
        std::fs::create_dir_all(&root).unwrap();
        let installed = Installed {
            commit: "0123456789abcdef".into(),
            commit_date: "2026-09-28T10:00:00Z".into(),
            installed_at: 1,
            secret_key: "s".into(),
        };
        write_installed(&paths, &installed).unwrap();
        assert_eq!(read_installed(&paths), None, "no Python yet: not set up");
        let python = paths.venv_python(&installed.commit);
        std::fs::create_dir_all(python.parent().unwrap()).unwrap();
        std::fs::write(&python, "").unwrap();
        assert_eq!(read_installed(&paths), Some(installed));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_busy_port_gives_another_free_one() {
        let taken = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = taken.local_addr().unwrap().port();
        let other = free_port(port);
        assert_ne!(other, port);
        assert!(std::net::TcpListener::bind(("127.0.0.1", other)).is_ok());
    }

    #[test]
    fn remove_old_installs_keeps_only_the_active_one() {
        let root = std::env::temp_dir().join(format!("oratype-search-{}", uuid::Uuid::new_v4()));
        let paths = SearchPaths::new(root.clone());
        for commit in ["aaaaaaaaaaaaaaaa", "bbbbbbbbbbbbbbbb"] {
            std::fs::create_dir_all(paths.install_dir(commit)).unwrap();
        }
        std::fs::create_dir_all(root.join("bin")).unwrap();
        remove_old_installs(&paths, "bbbbbbbbbbbbbbbb", None);
        assert!(!paths.install_dir("aaaaaaaaaaaaaaaa").exists());
        assert!(paths.install_dir("bbbbbbbbbbbbbbbb").exists());
        assert!(root.join("bin").exists(), "uv stays");
        std::fs::remove_dir_all(root).unwrap();
    }

    /// A real setup: downloads uv, Python and SearXNG (about 200 MB), starts it, searches once,
    /// updates (a no-op when nothing changed), then removes everything. Needs the internet.
    #[tokio::test(flavor = "multi_thread")]
    #[ignore = "downloads SearXNG; run by hand"]
    async fn real_setup_search_update_and_remove() {
        let root = std::env::temp_dir().join(format!("oratype-search-{}", uuid::Uuid::new_v4()));
        let server = SearchServer {
            paths: Mutex::new(Some(SearchPaths::new(root.clone()))),
            running: Mutex::new(None),
            busy: AtomicBool::new(false),
            last_progress: Mutex::new(None),
            start_lock: tokio::sync::Mutex::new(()),
        };
        let client = reqwest::Client::new();
        let steps = Mutex::new(Vec::new());
        let progress = |p: SetupProgress| {
            let mut steps = steps.lock().unwrap();
            if steps.last() != Some(&p.step) {
                println!("step {:?}", p.step);
                steps.push(p.step);
            }
        };
        let started = Instant::now();
        let installed = server.install(&client, &progress).await.expect("setup");
        println!("set up {} in {:.0?}", installed.commit, started.elapsed());
        let du = std::process::Command::new("du")
            .args(["-sm"])
            .arg(&root)
            .output()
            .unwrap();
        println!("on disk: {}", String::from_utf8_lossy(&du.stdout).trim());
        assert!(server.status().running);
        let base = server.ensure_running(&client).await.unwrap();
        let config = crate::web_search::WebSearchConfig {
            provider: crate::web_search::SearchProviderKind::Searxng,
            base_url: base.clone(),
        };
        let (count, took) = crate::web_search::test_provider(&client, &config, "")
            .await
            .expect("a search through the built-in server");
        println!("{base}: {count} results in {took:.0?}");
        let check = server.check_update(&client).await.unwrap();
        assert!(!check.update_available, "{check:?}");
        server.update(&client, &progress).await.expect("update");
        server.remove().await.unwrap();
        assert!(!root.exists());
        assert!(!server.status().running);
    }

    #[test]
    fn unpack_searxng_tar_gz_extracts_files_and_skips_templates_symlinks_and_colons() {
        let mut gz_bytes = Vec::new();
        {
            let encoder =
                flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
            let mut tar = tar::Builder::new(encoder);

            let add_file = |tar: &mut tar::Builder<_>, path: &str, contents: &[u8]| {
                let mut header = tar::Header::new_gnu();
                header.set_size(contents.len() as u64);
                header.set_mode(0o644);
                header.set_entry_type(tar::EntryType::Regular);
                tar.append_data(&mut header, path, contents).unwrap();
            };

            add_file(&mut tar, "searxng-abc123/searx/webapp.py", b"app = 'ok'");
            add_file(
                &mut tar,
                "searxng-abc123/utils/templates/etc/apache2/sites-available/searxng.conf",
                b"apache",
            );
            add_file(
                &mut tar,
                "searxng-abc123/docs/windows:bad.txt",
                b"should-skip",
            );

            let mut symlink = tar::Header::new_gnu();
            symlink.set_size(0);
            symlink.set_mode(0o777);
            symlink.set_entry_type(tar::EntryType::Symlink);
            tar.append_link(&mut symlink, "searxng-abc123/searx/link.py", "webapp.py")
                .unwrap();

            tar.into_inner().unwrap().finish().unwrap();
        }

        let dest = std::env::temp_dir().join(format!("oratype-searxng-{}", uuid::Uuid::new_v4()));
        unpack_searxng_tar_gz(&gz_bytes, &dest).expect("unpack cleanly");

        assert_eq!(
            std::fs::read_to_string(dest.join("searx").join("webapp.py")).unwrap(),
            "app = 'ok'"
        );
        assert!(!dest.join("utils").join("templates").exists());
        assert!(!dest.join("docs").join("windows:bad.txt").exists());
        assert!(!dest.join("searx").join("link.py").exists());

        std::fs::remove_dir_all(dest).unwrap();
    }

    #[test]
    fn unpack_searxng_tar_gz_rejects_parent_dir_traversal() {
        let mut gz_bytes = Vec::new();
        {
            let encoder =
                flate2::write::GzEncoder::new(&mut gz_bytes, flate2::Compression::default());
            let mut tar = tar::Builder::new(encoder);

            let mut header = tar::Header::new_gnu();
            let raw_name = b"searxng-abc123/../../evil.txt";
            header.as_old_mut().name[..raw_name.len()].copy_from_slice(raw_name);
            header.set_size(4);
            header.set_mode(0o644);
            header.set_entry_type(tar::EntryType::Regular);
            header.set_cksum();
            tar.append(&header, &b"evil"[..]).unwrap();

            tar.into_inner().unwrap().finish().unwrap();
        }

        let parent =
            std::env::temp_dir().join(format!("oratype-searxng-trap-{}", uuid::Uuid::new_v4()));
        let dest = parent.join("install");
        std::fs::create_dir_all(&dest).unwrap();

        let err = unpack_searxng_tar_gz(&gz_bytes, &dest).unwrap_err();
        assert!(err.contains("unsafe path"), "unexpected err: {err}");
        assert!(!parent.join("evil.txt").exists());

        std::fs::remove_dir_all(parent).unwrap();
    }
}
