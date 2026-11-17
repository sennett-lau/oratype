//! Plan `ai-polish-setup`: Built-in AI. OraType runs llama.cpp's `llama-server` as a separate
//! program on this Mac and sends it the same OpenAI-compatible chat requests as any other AI
//! preset.
//!
//! - The program ships inside the app bundle next to the main executable (a Tauri external
//!   binary, built by `scripts/build-llama-server.sh`). Without it Built-in AI reports
//!   [`ServerError::BinaryMissing`]; the rest of the app works.
//! - It listens on 127.0.0.1 on a free port, with every layer on the GPU, an 8192-token context,
//!   one request slot, thinking off (`--reasoning off`) and no web UI. Each start gets a new
//!   random API key (passed in the environment, not on the command line), so other programs
//!   and web pages cannot use it.
//! - [`ServerManager::ensure`] starts it on demand and waits for `/health`; a server that died
//!   is started again on the next request. It is stopped when OraType quits, when another AI
//!   engine is picked, and when its model is deleted.
//! - Its log goes to the OraType log filtered to errors, load and timing lines (see
//!   [`log_level_for`]); llama-server never prints the prompt or the answer at its default
//!   verbosity.
//! - Plan `warm-builtin-ai`: with "Fast language switching" on, it gets one request slot per
//!   language (up to [`MAX_LANGUAGE_SLOTS`]) and each dictation goes to its language's slot, so
//!   the server keeps every language's instructions read. After a start in the background it is
//!   warmed up: each slot reads its language's instructions once, before the first dictation.
//!
//! The first start after install or update compiles llama.cpp's Metal shaders, which takes one
//! to two minutes; macOS caches the result, and later starts take a few seconds.

use std::io::BufRead;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::error::AppError;
use crate::storage::{AiPreset, AppConfig};

use super::language_library::store::LibraryStore;
use super::LlmConfig;

/// The program's name inside `OraType.app/Contents/MacOS/`.
pub const SERVER_BINARY_NAME: &str = "llama-server";
/// Context size in tokens. The prompt and the answer share it: the polish prompt with a language
/// preset is about 3,300 tokens, so 4096 left too little room and cut long answers short
/// (1.1.3). 8192 holds a long dictation and its answer; it costs about 0.6 GB more memory.
pub const CONTEXT_TOKENS: u32 = 8192;
/// Plan `warm-builtin-ai`: at most this many request slots, one per language. Each slot has its
/// own [`CONTEXT_TOKENS`], which costs about 1.1 GB (Qwen 3.5 4B) to 2.2 GB (Qwen3 4B) of memory.
pub const MAX_LANGUAGE_SLOTS: u32 = 3;
/// Long enough for the first start, which compiles the Metal shaders.
pub const STARTUP_TIMEOUT: Duration = Duration::from_secs(240);
const HEALTH_POLL: Duration = Duration::from_millis(250);

/// Why the built-in server is not available.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServerError {
    /// This copy of OraType has no `llama-server` program.
    BinaryMissing,
    /// The model file is not downloaded (or the models folder is unknown).
    ModelMissing,
    /// The program could not be started.
    Spawn(String),
    /// The program stopped before it was ready (for example, the model did not load).
    Exited(String),
    /// It did not answer `/health` within [`STARTUP_TIMEOUT`].
    Timeout,
    /// Built-in AI was switched off while the server was starting.
    Stopped,
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::BinaryMissing => write!(
                f,
                "The built-in AI server (llama-server) is missing from this copy of OraType."
            ),
            ServerError::ModelMissing => write!(f, "The built-in AI model is not downloaded."),
            ServerError::Spawn(reason) => {
                write!(f, "Could not start the built-in AI server: {reason}")
            }
            ServerError::Exited(reason) => {
                write!(
                    f,
                    "The built-in AI server stopped while loading the model: {reason}"
                )
            }
            ServerError::Timeout => write!(
                f,
                "The built-in AI server did not start within {} minutes.",
                STARTUP_TIMEOUT.as_secs() / 60
            ),
            ServerError::Stopped => write!(f, "Built-in AI was switched off."),
        }
    }
}

/// Where to send requests while the server runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Endpoint {
    /// `http://127.0.0.1:<port>/v1`
    pub base_url: String,
    /// Sent as `Authorization: Bearer <key>`.
    pub api_key: String,
}

/// The program, its arguments and extra environment for one start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaunchSpec {
    pub binary: PathBuf,
    pub args: Vec<String>,
    pub envs: Vec<(String, String)>,
}

/// What a server runs: the AI polish model, or (plan `language-model-suggestions`) a speech model
/// such as Qwen3-ASR with its audio projector. Each has its own [`ServerManager`], so both can run
/// at once.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerRole {
    Polish,
    Speech,
}

impl ServerRole {
    /// How the log names the server.
    pub fn label(self) -> &'static str {
        match self {
            ServerRole::Polish => "Built-in AI server",
            ServerRole::Speech => "Built-in speech server",
        }
    }
}

/// Context size of the speech server: the audio's tokens and the transcript. Qwen3-ASR's encoder
/// gives about 13 tokens per second of audio, so a ten-minute dictation (the default recording
/// limit) needs about 7,800 for the audio and up to about 3,000 for its transcript. 8192 would cut
/// it short; each 4096 tokens cost about 0.45 GB.
pub const SPEECH_CONTEXT_TOKENS: u32 = 12288;

/// The arguments of one AI server start (see the module comment).
pub fn launch_spec(
    binary: &Path,
    model_path: &Path,
    model_alias: &str,
    port: u16,
    api_key: &str,
) -> LaunchSpec {
    launch_spec_for(
        ServerRole::Polish,
        binary,
        model_path,
        None,
        model_alias,
        port,
        api_key,
        1,
    )
}

/// The arguments of one server start for `role`. A speech server gets the audio projector and a
/// larger context, and no `--reasoning` switch (a speech model has no thinking mode). `slots`
/// request slots each get the full context (plan `warm-builtin-ai`).
#[allow(clippy::too_many_arguments)]
pub fn launch_spec_for(
    role: ServerRole,
    binary: &Path,
    model_path: &Path,
    projector_path: Option<&Path>,
    model_alias: &str,
    port: u16,
    api_key: &str,
    slots: u32,
) -> LaunchSpec {
    let slots = slots.max(1);
    let context = match role {
        ServerRole::Polish => CONTEXT_TOKENS,
        ServerRole::Speech => SPEECH_CONTEXT_TOKENS,
    } * slots;
    let mut args: Vec<String> = [
        "--model",
        &model_path.to_string_lossy(),
        "--alias",
        model_alias,
        "--host",
        "127.0.0.1",
        "--port",
        &port.to_string(),
        "--n-gpu-layers",
        "999",
        "--ctx-size",
        &context.to_string(),
        "--parallel",
        &slots.to_string(),
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    if let Some(projector) = projector_path {
        args.push("--mmproj".to_string());
        args.push(projector.to_string_lossy().to_string());
    }
    if role == ServerRole::Polish {
        args.extend(["--reasoning", "off"].map(String::from));
    }
    args.extend(["--no-webui", "--offline", "--cors-origins", "localhost"].map(String::from));
    LaunchSpec {
        binary: binary.to_path_buf(),
        args,
        envs: vec![("LLAMA_API_KEY".to_string(), api_key.to_string())],
    }
}

/// Applies the persisted Built-in backend preference (`"auto"`, `"cpu"`, or an explicit device
/// such as `"Vulkan0"`) to `llama-server` CLI arguments.
pub fn apply_backend_choice_to_args(
    args: &mut Vec<String>,
    has_projector: bool,
    backend_choice: &str,
) {
    let choice = crate::storage::normalize_builtin_backend(backend_choice);
    match choice.as_str() {
        "auto" => {}
        "cpu" => {
            if let Some(pos) = args.iter().position(|arg| arg == "--n-gpu-layers") {
                if let Some(layers) = args.get_mut(pos + 1) {
                    *layers = "0".to_string();
                }
            }
            args.extend(["--device", "none"].map(String::from));
            if has_projector {
                args.extend(["--no-mmproj-offload", "--mmproj-device", "none"].map(String::from));
            }
        }
        device => {
            args.extend(["--device", device, "--split-mode", "none"].map(String::from));
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn launch_spec_for_with_backend(
    role: ServerRole,
    binary: &Path,
    model_path: &Path,
    projector_path: Option<&Path>,
    model_alias: &str,
    port: u16,
    api_key: &str,
    slots: u32,
    backend_choice: &str,
) -> LaunchSpec {
    let mut spec = launch_spec_for(
        role,
        binary,
        model_path,
        projector_path,
        model_alias,
        port,
        api_key,
        slots,
    );
    apply_backend_choice_to_args(&mut spec.args, projector_path.is_some(), backend_choice);
    spec
}

/// A started server program.
pub trait ServerChild: Send {
    fn pid(&self) -> u32;
    /// `Some(description)` once the program has exited.
    fn exit_status(&mut self) -> Option<String>;
    fn kill(&mut self);
}

/// Starts server programs and asks them whether they are ready. The real one runs
/// `llama-server`; tests use a fake.
#[async_trait]
pub trait Launcher: Send + Sync {
    fn find_binary(&self) -> Option<PathBuf>;
    fn spawn(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn ServerChild>>;
    /// True when `GET {origin}/health` answers 200 (the model is loaded).
    async fn healthy(&self, origin: &str) -> bool;
}

struct Running {
    child: Box<dyn ServerChild>,
    model_path: PathBuf,
    projector_path: Option<PathBuf>,
    slots: u32,
    backend_choice: String,
    endpoint: Endpoint,
}

/// Owns one `llama-server` process: the AI server ([`server`]) or the speech server
/// ([`speech_server`]).
pub struct ServerManager {
    role: ServerRole,
    launcher: Box<dyn Launcher>,
    /// One start at a time; a second caller waits and then reuses the started server.
    gate: tokio::sync::Mutex<()>,
    running: Mutex<Option<Running>>,
    /// Bumped by `stop`, so a start that was under way when Built-in AI was switched off does
    /// not leave a server behind.
    generation: AtomicU64,
    models_dir: Mutex<Option<PathBuf>>,
    pid_file: Mutex<Option<PathBuf>>,
    /// Request slots for the next start (plan `warm-builtin-ai`); set from the config by
    /// [`sync_with_config`], so a start on demand uses the same number.
    slots: Mutex<u32>,
    backend_choice: Mutex<String>,
    startup_timeout: Duration,
    poll: Duration,
}

impl ServerManager {
    pub fn new(launcher: Box<dyn Launcher>) -> Self {
        Self::for_role(ServerRole::Polish, launcher)
    }

    pub fn for_role(role: ServerRole, launcher: Box<dyn Launcher>) -> Self {
        Self {
            role,
            launcher,
            gate: tokio::sync::Mutex::new(()),
            running: Mutex::new(None),
            generation: AtomicU64::new(0),
            models_dir: Mutex::new(None),
            pid_file: Mutex::new(None),
            slots: Mutex::new(1),
            backend_choice: Mutex::new("auto".to_string()),
            startup_timeout: STARTUP_TIMEOUT,
            poll: HEALTH_POLL,
        }
    }

    #[cfg(test)]
    fn with_timing(mut self, startup_timeout: Duration, poll: Duration) -> Self {
        self.startup_timeout = startup_timeout;
        self.poll = poll;
        self
    }

    /// Sets the models folder and the file that remembers the server's process id. A server
    /// left running by a OraType that crashed is stopped here.
    pub fn set_paths(&self, models_dir: PathBuf, pid_file: PathBuf) {
        stop_stale_server(&pid_file);
        *lock(&self.models_dir) = Some(models_dir);
        *lock(&self.pid_file) = Some(pid_file);
    }

    pub fn binary_available(&self) -> bool {
        self.launcher.find_binary().is_some()
    }

    /// Sets how many request slots the next start gets (at least 1). A running server with a
    /// different number is started again by the next [`ensure`](Self::ensure).
    pub fn set_slots(&self, slots: u32) {
        *lock(&self.slots) = slots.max(1);
    }

    /// Sets the backend preference (`"auto"`, `"cpu"`, or a specific device id like `"Vulkan0"`).
    /// A running server with a different backend is restarted on the next [`ensure`](Self::ensure).
    pub fn set_backend_choice(&self, choice: &str) {
        *lock(&self.backend_choice) = crate::storage::normalize_builtin_backend(choice);
    }

    pub fn backend_choice(&self) -> String {
        lock(&self.backend_choice).clone()
    }

    /// True if a healthy server is already serving `model_file`, `slots`, and `backend_choice`.
    pub fn running_matches(&self, model_file: &str, slots: u32, backend_choice: &str) -> bool {
        let mut running = lock(&self.running);
        let Some(current) = running.as_mut() else {
            return false;
        };
        if current.child.exit_status().is_some() {
            return false;
        }
        let wanted_backend = crate::storage::normalize_builtin_backend(backend_choice);
        current
            .model_path
            .file_name()
            .is_some_and(|name| name.to_string_lossy() == model_file)
            && current.slots == slots.max(1)
            && current.backend_choice == wanted_backend
    }

    /// The model file the running server serves, if one runs.
    pub fn running_model_file(&self) -> Option<String> {
        self.running_model_and_slots().map(|(file, _)| file)
    }

    /// The running server's model file and number of request slots, if one runs.
    pub fn running_model_and_slots(&self) -> Option<(String, u32)> {
        let mut running = lock(&self.running);
        let current = running.as_mut()?;
        if current.child.exit_status().is_some() {
            return None;
        }
        let file = current
            .model_path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())?;
        Some((file, current.slots))
    }

    /// Returns the address of a server running `model_file`, starting it (or starting it
    /// again) first when needed.
    pub async fn ensure(&self, model_file: &str, alias: &str) -> Result<Endpoint, ServerError> {
        self.ensure_with(model_file, None, alias).await
    }

    /// Like [`ensure`](Self::ensure), with an audio projector file (`--mmproj`) from the same
    /// models folder, for a speech model.
    pub async fn ensure_with(
        &self,
        model_file: &str,
        projector_file: Option<&str>,
        alias: &str,
    ) -> Result<Endpoint, ServerError> {
        let label = self.role.label();
        let _gate = self.gate.lock().await;
        let dir = lock(&self.models_dir)
            .clone()
            .ok_or(ServerError::ModelMissing)?;
        let model_path = dir.join(model_file);
        if model_file.trim().is_empty() || !model_path.is_file() {
            return Err(ServerError::ModelMissing);
        }
        let projector_path = match projector_file {
            Some(file) if file.trim().is_empty() || !dir.join(file).is_file() => {
                return Err(ServerError::ModelMissing)
            }
            Some(file) => Some(dir.join(file)),
            None => None,
        };
        let slots = *lock(&self.slots);
        let backend_choice = lock(&self.backend_choice).clone();

        {
            let mut running = lock(&self.running);
            if let Some(current) = running.as_mut() {
                match current.child.exit_status() {
                    None if current.model_path == model_path
                        && current.projector_path == projector_path
                        && current.slots == slots
                        && current.backend_choice == backend_choice =>
                    {
                        return Ok(current.endpoint.clone())
                    }
                    None => {}
                    Some(status) => {
                        tracing::warn!("{label} had stopped ({status}); starting it again")
                    }
                }
            }
        }
        self.stop();
        let generation = self.generation.load(Ordering::SeqCst);

        let binary = self
            .launcher
            .find_binary()
            .ok_or(ServerError::BinaryMissing)?;
        let port = free_port().map_err(|error| ServerError::Spawn(error.to_string()))?;
        let api_key = uuid::Uuid::new_v4().simple().to_string();
        let spec = launch_spec_for_with_backend(
            self.role,
            &binary,
            &model_path,
            projector_path.as_deref(),
            alias,
            port,
            &api_key,
            slots,
            &backend_choice,
        );
        let started = Instant::now();
        let mut child = self
            .launcher
            .spawn(&spec)
            .map_err(|error| ServerError::Spawn(error.to_string()))?;
        self.write_pid(Some(child.pid()));
        tracing::info!(
            "{label} starting: {} on port {port}, {slots} slot{}",
            model_path
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_default(),
            if slots == 1 { "" } else { "s" }
        );

        let origin = format!("http://127.0.0.1:{port}");
        loop {
            if self.launcher.healthy(&origin).await {
                break;
            }
            if let Some(status) = child.exit_status() {
                self.write_pid(None);
                tracing::warn!("{label} exited while starting: {status}");
                return Err(ServerError::Exited(status));
            }
            if started.elapsed() >= self.startup_timeout {
                child.kill();
                self.write_pid(None);
                tracing::warn!("{label} did not become ready; stopped it");
                return Err(ServerError::Timeout);
            }
            tokio::time::sleep(self.poll).await;
        }
        if self.generation.load(Ordering::SeqCst) != generation {
            child.kill();
            self.write_pid(None);
            return Err(ServerError::Stopped);
        }
        tracing::info!("{label} ready in {:.1} s", started.elapsed().as_secs_f64());
        let endpoint = Endpoint {
            base_url: format!("{origin}/v1"),
            api_key,
        };
        *lock(&self.running) = Some(Running {
            child,
            model_path,
            projector_path,
            slots,
            backend_choice,
            endpoint: endpoint.clone(),
        });
        Ok(endpoint)
    }

    /// Stops the server if one runs (and any start under way). Safe to call at any time,
    /// including from the app's exit handler.
    pub fn stop(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
        let previous = lock(&self.running).take();
        if let Some(mut current) = previous {
            current.child.kill();
            self.write_pid(None);
            tracing::info!("{} stopped", self.role.label());
        }
    }

    fn write_pid(&self, pid: Option<u32>) {
        let Some(path) = lock(&self.pid_file).clone() else {
            return;
        };
        let result = match pid {
            Some(pid) => std::fs::write(&path, pid.to_string()),
            None => match std::fs::remove_file(&path) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                other => other,
            },
        };
        if let Err(error) = result {
            tracing::warn!("Could not update {}: {error}", path.display());
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn free_port() -> std::io::Result<u16> {
    // TODO(windows): binding a local HTTP port may still prompt Windows Defender Firewall if any code later binds non-loopback interfaces; verify `127.0.0.1` passes cleanly on Windows 10/11.
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0))?;
    Ok(listener.local_addr()?.port())
}

/// Whether a process image path or command name belongs to `llama-server` (or a
/// triple-suffixed build of it).
fn is_server_process_name(process_path_or_name: &str) -> bool {
    let file_name = process_path_or_name
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(process_path_or_name)
        .trim();
    let lower = file_name.to_ascii_lowercase();
    let stem = lower.strip_suffix(".exe").unwrap_or(&lower);
    stem == SERVER_BINARY_NAME || stem.starts_with("llama-server-")
}

/// Whether an environment variable is a `LLAMA_*` setting read by `llama-server` (case-insensitive
/// for Windows).
fn is_llama_env_var(name: &str) -> bool {
    name.len() >= 6 && name[..6].eq_ignore_ascii_case("LLAMA_")
}

/// Stops a `llama-server` a crashed OraType left behind (its id is in `pid_file`). Only a
/// process that really is `llama-server` is stopped.
fn stop_stale_server(pid_file: &Path) {
    let Ok(text) = std::fs::read_to_string(pid_file) else {
        return;
    };
    let _ = std::fs::remove_file(pid_file);
    let Ok(pid) = text.trim().parse::<u32>() else {
        return;
    };
    if pid == 0 {
        return;
    }
    #[cfg(unix)]
    {
        let name = std::process::Command::new("ps")
            .args(["-p", &pid.to_string(), "-o", "comm="])
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
            .unwrap_or_default();
        if is_server_process_name(&name) {
            tracing::info!("Stopping a built-in AI server left from an earlier run (pid {pid})");
            let _ = std::process::Command::new("kill")
                .arg(pid.to_string())
                .status();
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
        // rights, inspect its full image path, terminate strictly when it matches `llama-server`,
        // and unconditionally close the handle.
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
                if is_server_process_name(&image_path) {
                    tracing::info!(
                        "Stopping a built-in AI server left from an earlier run (pid {pid})"
                    );
                    let _ = TerminateProcess(handle, 1);
                }
            }
            let _ = CloseHandle(handle);
        }
    }
}

/// Which level a line of llama-server's log gets in the OraType log, or `None` to drop it.
/// Lines look like `0.02.681.426 I srv  llama_server: model loaded`; the letter is the level.
/// Errors, the load and listen lines and each request's timings are kept: prompt tokens read and
/// how fast ("prompt eval time"), tokens written and how fast ("eval time"), the total, and the
/// slot's size at the end ("stop processing: n_tokens"), which shows how much came from the cache.
pub fn log_level_for(line: &str) -> Option<tracing::Level> {
    let mut parts = line.split_whitespace();
    let _time = parts.next()?;
    match parts.next()? {
        "E" => Some(tracing::Level::WARN),
        "W" if !line.contains("n_past was set to") => Some(tracing::Level::WARN),
        "I" if line.contains("model loaded")
            || line.contains("listening on")
            || line.contains("eval time")
            || line.contains("total time")
            || line.contains("stop processing")
            || line.contains("load_tensors")
            || line.contains("offload")
            || line.contains("Vulkan")
            || line.contains("encode")
            || line.contains("mtmd")
            || line.contains("clip") =>
        {
            Some(tracing::Level::INFO)
        }
        _ => None,
    }
}

/// Where the `llama-server` program is: next to the main executable (the app bundle's
/// `Contents/MacOS/`, where Tauri puts external binaries), or with the target triple in its
/// name. `ORATYPE_LLAMA_SERVER` overrides it; debug builds also look in
/// `src-tauri/binaries/`, where the build script puts it.
pub fn find_server_binary() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("ORATYPE_LLAMA_SERVER") {
        let path = PathBuf::from(path);
        return path.is_file().then_some(path);
    }
    let exe_name = format!("{SERVER_BINARY_NAME}{}", std::env::consts::EXE_SUFFIX);
    let triple_name = format!(
        "{SERVER_BINARY_NAME}-{}{}",
        env!("ORATYPE_TARGET_TRIPLE"),
        std::env::consts::EXE_SUFFIX
    );
    let mut candidates = Vec::new();
    if let Some(dir) = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(Path::to_path_buf))
    {
        candidates.push(dir.join(&exe_name));
        candidates.push(dir.join(&triple_name));
    }
    if cfg!(debug_assertions) {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(&triple_name),
        );
    }
    candidates.into_iter().find(|path| path.is_file())
}

/// Runs the real `llama-server`.
pub struct ProcessLauncher {
    client: reqwest::Client,
}

impl ProcessLauncher {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_secs(2))
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for ProcessLauncher {
    fn default() -> Self {
        Self::new()
    }
}

/// One GPU entry printed by `llama-server --list-devices`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LlamaDevice {
    pub id: String,
    pub description: String,
    pub total_mib: u64,
    pub free_mib: u64,
}

fn is_virtual_or_software_vk_description(description: &str) -> bool {
    let lower = description.to_ascii_lowercase();
    [
        "llvmpipe",
        "lavapipe",
        "basic render",
        "swiftshader",
        "parsec",
        "sudomaker",
        "virtual display",
        "virtual desktop",
        "virtual monitor",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

fn is_integrated_vk_description(description: &str) -> bool {
    let lower = description.to_ascii_lowercase();
    [
        "radeon(tm) graphics",
        "amd radeon graphics",
        "uhd graphics",
        "iris(r) xe",
        "iris xe",
        "hd graphics",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
}

/// Parses `llama-server --list-devices` output lines such as
/// `  Vulkan0: NVIDIA GeForce RTX 4090 (24142 MiB, 23400 MiB free)`.
pub fn parse_llama_list_devices(output: &str) -> Vec<LlamaDevice> {
    let mut devices = Vec::new();
    for line in output.lines() {
        let trimmed = line.trim();
        let Some((raw_id, rest)) = trimmed.split_once(": ") else {
            continue;
        };
        if raw_id.is_empty() || raw_id.chars().any(char::is_whitespace) {
            continue;
        }
        let Some(open_paren) = rest.rfind('(') else {
            continue;
        };
        let Some(inner) = rest[open_paren + 1..].strip_suffix(')') else {
            continue;
        };
        let mut parts = inner.split(',');
        let Some(total_part) = parts.next() else {
            continue;
        };
        let Some(free_part) = parts.next() else {
            continue;
        };
        if parts.next().is_some() {
            continue;
        }
        let Ok(total_mib) = total_part
            .trim()
            .strip_suffix("MiB")
            .unwrap_or_default()
            .trim()
            .parse::<u64>()
        else {
            continue;
        };
        let Ok(free_mib) = free_part
            .trim()
            .strip_suffix("free")
            .unwrap_or_default()
            .trim()
            .strip_suffix("MiB")
            .unwrap_or_default()
            .trim()
            .parse::<u64>()
        else {
            continue;
        };
        let description = rest[..open_paren].trim().to_string();
        if !description.is_empty() {
            devices.push(LlamaDevice {
                id: raw_id.to_string(),
                description,
                total_mib,
                free_mib,
            });
        }
    }
    devices
}

/// Picks the best Vulkan device id (`Vulkan0`, `Vulkan1`, ...) for `llama-server`.
/// Matches `preferred_gpu_name` from DXGI when present (e.g. `NVIDIA GeForce RTX 4090`), and
/// otherwise prefers discrete GPUs with the largest VRAM over integrated GPUs whose Vulkan heaps
/// include shared host memory.
pub fn select_best_llama_device(
    devices: &[LlamaDevice],
    preferred_gpu_name: Option<&str>,
) -> Option<String> {
    let hardware: Vec<&LlamaDevice> = devices
        .iter()
        .filter(|d| !d.id.eq_ignore_ascii_case("CPU"))
        .filter(|d| !is_virtual_or_software_vk_description(&d.description))
        .collect();
    if let Some(pref) = preferred_gpu_name.map(str::trim).filter(|s| !s.is_empty()) {
        let pref_lower = pref.to_ascii_lowercase();
        if let Some(matched) = hardware
            .iter()
            .copied()
            .filter(|d| {
                let desc_lower = d.description.to_ascii_lowercase();
                desc_lower.contains(&pref_lower) || pref_lower.contains(&desc_lower)
            })
            .max_by_key(|d| (d.total_mib, d.free_mib))
        {
            return Some(matched.id.clone());
        }
    }
    let discrete_max = hardware
        .iter()
        .copied()
        .filter(|d| !is_integrated_vk_description(&d.description))
        .max_by_key(|d| (d.total_mib, d.free_mib));
    let picked = discrete_max.or_else(|| {
        hardware
            .into_iter()
            .max_by_key(|d| (d.total_mib, d.free_mib))
    });
    picked.map(|d| d.id.clone())
}

#[cfg(target_os = "windows")]
pub fn windows_vulkan_devices_for_binary(binary: &Path) -> &'static [LlamaDevice] {
    static DEVICES: std::sync::OnceLock<Vec<LlamaDevice>> = std::sync::OnceLock::new();
    DEVICES
        .get_or_init(|| {
            let mut command = std::process::Command::new(binary);
            crate::platform::configure_background_command(&mut command);
            command
                .arg("--list-devices")
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());
            for (name, _) in std::env::vars_os() {
                if is_llama_env_var(&name.to_string_lossy()) {
                    command.env_remove(name);
                }
            }
            let Ok(output) = command.output() else {
                return Vec::new();
            };
            let mut combined = String::from_utf8_lossy(&output.stdout).into_owned();
            combined.push('\n');
            combined.push_str(&String::from_utf8_lossy(&output.stderr));
            parse_llama_list_devices(&combined)
                .into_iter()
                .filter(|d| !d.id.eq_ignore_ascii_case("CPU"))
                .filter(|d| !is_virtual_or_software_vk_description(&d.description))
                .collect()
        })
        .as_slice()
}

pub fn list_windows_vulkan_devices() -> Vec<LlamaDevice> {
    #[cfg(target_os = "windows")]
    {
        find_server_binary()
            .map(|binary| windows_vulkan_devices_for_binary(&binary).to_vec())
            .unwrap_or_default()
    }
    #[cfg(not(target_os = "windows"))]
    {
        Vec::new()
    }
}

#[cfg(target_os = "windows")]
pub fn preferred_windows_vulkan_device(binary: &Path) -> Option<String> {
    static DEVICE: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    DEVICE
        .get_or_init(|| {
            let devices = windows_vulkan_devices_for_binary(binary);
            let dxgi_pref = crate::stt::hardware::preferred_windows_gpu_name();
            let chosen = select_best_llama_device(devices, dxgi_pref.as_deref());
            tracing::info!(
                "Built-in Vulkan devices detected: {:?}; dxgi_pref={:?}; selected={:?}",
                devices,
                dxgi_pref,
                chosen
            );
            chosen
        })
        .clone()
}

#[cfg(target_os = "windows")]
pub fn preferred_windows_vulkan_device_index() -> Option<i32> {
    let binary = find_server_binary()?;
    let device = preferred_windows_vulkan_device(&binary)?;
    let lower = device.to_ascii_lowercase();
    lower
        .strip_prefix("vulkan")
        .and_then(|rest| rest.trim().parse::<i32>().ok())
}

#[async_trait]
impl Launcher for ProcessLauncher {
    fn find_binary(&self) -> Option<PathBuf> {
        find_server_binary()
    }

    fn spawn(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn ServerChild>> {
        let mut command = std::process::Command::new(&spec.binary);
        crate::platform::configure_background_command(&mut command);
        command
            .args(&spec.args)
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        #[cfg(target_os = "windows")]
        if !spec
            .args
            .iter()
            .any(|arg| arg == "--device" || arg == "-dev")
        {
            if let Some(device) = preferred_windows_vulkan_device(&spec.binary) {
                command.args(["--device", &device, "--split-mode", "none"]);
            }
        }
        // llama-server reads LLAMA_ARG_* settings from the environment; only ours count.
        for (name, _) in std::env::vars_os() {
            if is_llama_env_var(&name.to_string_lossy()) {
                command.env_remove(name);
            }
        }
        for (name, value) in &spec.envs {
            command.env(name, value);
        }
        let mut child = command.spawn()?;
        let last_error = Arc::new(Mutex::new(String::new()));
        if let Some(stdout) = child.stdout.take() {
            forward_log(stdout, last_error.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            forward_log(stderr, last_error.clone());
        }
        Ok(Box::new(ProcessChild { child, last_error }))
    }

    async fn healthy(&self, origin: &str) -> bool {
        matches!(
            self.client.get(format!("{origin}/health")).send().await,
            Ok(response) if response.status().is_success()
        )
    }
}

/// Copies the kept lines of the server's output into the OraType log (on its own thread).
fn forward_log(stream: impl std::io::Read + Send + 'static, last_error: Arc<Mutex<String>>) {
    std::thread::spawn(move || {
        for line in std::io::BufReader::new(stream).lines() {
            let Ok(line) = line else { break };
            let line: String = line.chars().take(300).collect();
            match log_level_for(&line) {
                Some(tracing::Level::WARN) => {
                    tracing::warn!("[llama-server] {line}");
                    *lock(&last_error) = line;
                }
                Some(_) => tracing::info!("[llama-server] {line}"),
                None => {}
            }
        }
    });
}

struct ProcessChild {
    child: std::process::Child,
    last_error: Arc<Mutex<String>>,
}

impl ServerChild for ProcessChild {
    fn pid(&self) -> u32 {
        self.child.id()
    }

    fn exit_status(&mut self) -> Option<String> {
        match self.child.try_wait() {
            Ok(Some(status)) => {
                let last = lock(&self.last_error).clone();
                Some(if last.is_empty() {
                    status.to_string()
                } else {
                    format!("{status}; last error: {last}")
                })
            }
            Ok(None) => None,
            Err(error) => Some(error.to_string()),
        }
    }

    fn kill(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// The app's AI server manager.
pub fn server() -> &'static ServerManager {
    static SERVER: std::sync::OnceLock<ServerManager> = std::sync::OnceLock::new();
    SERVER.get_or_init(|| ServerManager::new(Box::new(ProcessLauncher::new())))
}

/// Plan `language-model-suggestions`: the app's speech server manager, for built-in speech models
/// that run in `llama-server` (Qwen3-ASR). Separate from the AI server, so both models stay
/// loaded.
pub fn speech_server() -> &'static ServerManager {
    static SERVER: std::sync::OnceLock<ServerManager> = std::sync::OnceLock::new();
    SERVER.get_or_init(|| {
        ServerManager::for_role(ServerRole::Speech, Box::new(ProcessLauncher::new()))
    })
}

/// The address and key to use for `preset`: its own for a server preset, the running built-in
/// server's (started when needed) for the Built-in preset.
pub async fn endpoint_for(preset: &AiPreset, api_key: String) -> Result<Endpoint, ServerError> {
    if !preset.is_builtin_llama() {
        return Ok(Endpoint {
            base_url: preset.base_url.clone(),
            api_key,
        });
    }
    server().ensure(&preset.model_file, &preset.model).await
}

/// The preset with its runtime address filled in, and the key to send.
pub async fn resolve_preset(
    preset: &AiPreset,
    api_key: String,
) -> Result<(AiPreset, String), ServerError> {
    let endpoint = endpoint_for(preset, api_key).await?;
    let mut resolved = preset.clone();
    resolved.base_url = endpoint.base_url;
    Ok((resolved, endpoint.api_key))
}

/// The chat settings for the active AI preset (plan `ai-polish-setup`: the polish pipeline itself
/// is unchanged; only the address and key come from the running built-in server).
pub async fn llm_config(preset: &AiPreset, api_key: String) -> Result<LlmConfig, AppError> {
    let (resolved, api_key) = resolve_preset(preset, api_key)
        .await
        .map_err(|error| AppError::Config(error.to_string()))?;
    Ok(LlmConfig::from_preset(&resolved, api_key))
}

/// The automatic test after setup: one short clean-up request, like a real polish. Returns the
/// round-trip time in milliseconds; an empty answer counts as a failure (a thinking model that
/// used its whole budget would give one).
pub async fn test_request(endpoint: &Endpoint, model: &str) -> Result<u32, String> {
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .map_err(|error| error.to_string())?;
    let body = super::protocol::build_chat_body(
        model,
        vec![
            serde_json::json!({"role": "system", "content":
                "Clean up the dictated text: remove filler words and fix the punctuation. \
                 Reply with the text only."}),
            serde_json::json!({"role": "user", "content":
                "um so this is a quick test of the built in model"}),
        ],
        48,
        0.3,
        false,
        &serde_json::Map::new(),
    );
    let url = super::protocol::chat_endpoint(&endpoint.base_url)?;
    let request = super::protocol::apply_auth_headers(client.post(url), &endpoint.api_key)
        .json(&body)
        .timeout(super::protocol::REQUEST_TIMEOUT);
    let started = Instant::now();
    let response = request.send().await.map_err(|error| error.to_string())?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("HTTP {}", status.as_u16()));
    }
    let body: serde_json::Value = response.json().await.map_err(|error| error.to_string())?;
    let elapsed = started.elapsed().as_millis() as u32;
    if super::protocol::response_text(&body).trim().is_empty() {
        return Err("the model returned an empty answer".to_string());
    }
    Ok(elapsed)
}

/// What the built-in server should do for this config: run the Built-in preset's model when it
/// is tested and is the AI polish preset (which also translates, plan
/// `language-prompt-library`); otherwise nothing.
pub fn wanted_model(config: &AppConfig) -> Option<(String, String)> {
    Some(config.active_ai_preset())
        .filter(|preset| {
            preset.is_builtin_llama()
                && !preset.model_file.is_empty()
                && preset.verified_at.is_some()
        })
        .map(|preset| (preset.model_file.clone(), preset.model.clone()))
}

/// Plan `warm-builtin-ai`: the language codes that get their own request slot, in slot order:
/// the router's languages it can recognise (see
/// [`super::language_router::LanguageProfile::recognisable`]), at most [`MAX_LANGUAGE_SLOTS`].
/// A language without a preset or hints is never routed, so a slot for it would sit unused.
/// Empty when "Fast language switching" is off. `store` is where downloaded presets live.
pub fn slot_languages(config: &AppConfig, store: Option<&LibraryStore>) -> Vec<String> {
    if !config.builtin_ai_language_slots {
        return Vec::new();
    }
    super::language_router::language_profiles(config, store)
        .into_iter()
        .filter(|profile| profile.recognisable())
        .map(|profile| profile.code)
        .take(MAX_LANGUAGE_SLOTS as usize)
        .collect()
}

/// How many request slots the built-in server runs with for `config`: one per language with
/// "Fast language switching" on (at least 1), else 1.
pub fn wanted_slots(config: &AppConfig, store: Option<&LibraryStore>) -> u32 {
    (slot_languages(config, store).len() as u32).max(1)
}

/// The slot a dictation in `language` goes to, when it has one of its own. `None` lets the server
/// pick (one slot, or a language without its own).
pub fn slot_for_language(
    config: &AppConfig,
    store: Option<&LibraryStore>,
    language: &str,
) -> Option<u32> {
    let languages = slot_languages(config, store);
    if languages.len() < 2 {
        return None;
    }
    languages
        .iter()
        .position(|code| code == language)
        .map(|index| index as u32)
}

/// Bumped when a recording starts: a warm-up that began under an earlier value stops, so it
/// never competes with a dictation for the GPU (plan `warm-builtin-ai`).
static WARM_UP_RUN: AtomicU64 = AtomicU64::new(0);

/// The current warm-up run; a warm-up keeps it and stops once it changes.
pub fn warm_up_run() -> u64 {
    WARM_UP_RUN.load(Ordering::SeqCst)
}

/// Stops any warm-up under way (a recording started).
pub fn stop_warm_up() {
    WARM_UP_RUN.fetch_add(1, Ordering::SeqCst);
}

type WarmUp = Box<dyn Fn() + Send + Sync>;

/// What runs after the built-in server started in the background (set once at app start; the
/// pipeline sends each language's instructions, plan `warm-builtin-ai`).
static WARM_UP: std::sync::OnceLock<WarmUp> = std::sync::OnceLock::new();

pub fn set_warm_up(warm_up: impl Fn() + Send + Sync + 'static) {
    let _ = WARM_UP.set(Box::new(warm_up));
}

/// Runs the warm-up, if one is set. Called after a background start and after Built-in AI
/// passed its setup test.
pub fn warm_up() {
    if let Some(warm_up) = WARM_UP.get() {
        warm_up();
    }
}

/// Starts or stops the built-in server to match `config` (at app start and after settings
/// changed). Starting runs in the background and ends with the warm-up. A server with another
/// number of slots (languages or "Fast language switching" changed) is started again.
pub fn sync_with_config(config: &AppConfig, store: Option<&LibraryStore>) {
    server().set_backend_choice(&config.builtin_ai_backend);
    match wanted_model(config) {
        Some((model_file, alias)) => {
            let slots = wanted_slots(config, store);
            server().set_slots(slots);
            if server().running_matches(&model_file, slots, &config.builtin_ai_backend) {
                return;
            }
            tauri::async_runtime::spawn(async move {
                match server().ensure(&model_file, &alias).await {
                    Ok(_) => warm_up(),
                    Err(error) => tracing::warn!("Built-in AI server did not start: {error}"),
                }
            });
        }
        None => server().stop(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    /// A fake launcher: `healthy` answers true after `ready_after` checks; `exit_after` makes
    /// the child report an exit after that many status checks.
    struct Fake {
        binary: Option<PathBuf>,
        ready_after: usize,
        exit_after: Option<usize>,
        health_checks: Arc<AtomicUsize>,
        spawned: Arc<Mutex<Vec<LaunchSpec>>>,
        children: Arc<Mutex<Vec<Arc<FakeState>>>>,
    }

    #[derive(Default)]
    struct FakeState {
        killed: std::sync::atomic::AtomicBool,
        status_checks: AtomicUsize,
        exit_after: Option<usize>,
    }

    struct FakeChild(Arc<FakeState>);

    impl ServerChild for FakeChild {
        fn pid(&self) -> u32 {
            4242
        }
        fn exit_status(&mut self) -> Option<String> {
            if self.0.killed.load(Ordering::SeqCst) {
                return Some("killed".into());
            }
            let checks = self.0.status_checks.fetch_add(1, Ordering::SeqCst) + 1;
            match self.0.exit_after {
                Some(limit) if checks >= limit => Some("exit status: 1".into()),
                _ => None,
            }
        }
        fn kill(&mut self) {
            self.0.killed.store(true, Ordering::SeqCst);
        }
    }

    #[async_trait]
    impl Launcher for Fake {
        fn find_binary(&self) -> Option<PathBuf> {
            self.binary.clone()
        }
        fn spawn(&self, spec: &LaunchSpec) -> std::io::Result<Box<dyn ServerChild>> {
            lock(&self.spawned).push(spec.clone());
            let state = Arc::new(FakeState {
                exit_after: self.exit_after,
                ..FakeState::default()
            });
            lock(&self.children).push(state.clone());
            self.health_checks.store(0, Ordering::SeqCst);
            Ok(Box::new(FakeChild(state)))
        }
        async fn healthy(&self, _origin: &str) -> bool {
            self.health_checks.fetch_add(1, Ordering::SeqCst) + 1 >= self.ready_after
        }
    }

    struct Setup {
        manager: ServerManager,
        spawned: Arc<Mutex<Vec<LaunchSpec>>>,
        children: Arc<Mutex<Vec<Arc<FakeState>>>>,
        dir: PathBuf,
    }

    fn setup(name: &str, binary: bool, ready_after: usize, exit_after: Option<usize>) -> Setup {
        let dir = std::env::temp_dir().join(format!(
            "oratype-llama-{name}-{}-{}",
            std::process::id(),
            crate::storage::now_unix_ms()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a.gguf"), b"model a").unwrap();
        std::fs::write(dir.join("b.gguf"), b"model b").unwrap();
        let spawned = Arc::new(Mutex::new(Vec::new()));
        let children = Arc::new(Mutex::new(Vec::new()));
        let fake = Fake {
            binary: binary.then(|| PathBuf::from("/fake/llama-server")),
            ready_after,
            exit_after,
            health_checks: Arc::new(AtomicUsize::new(0)),
            spawned: spawned.clone(),
            children: children.clone(),
        };
        let manager = ServerManager::new(Box::new(fake))
            .with_timing(Duration::from_millis(200), Duration::from_millis(1));
        manager.set_paths(dir.clone(), dir.join("llama-server.pid"));
        Setup {
            manager,
            spawned,
            children,
            dir,
        }
    }

    #[test]
    fn the_launch_spec_has_the_plan_settings_and_keeps_the_key_off_the_command_line() {
        let spec = launch_spec(
            Path::new("/app/llama-server"),
            Path::new("/models/m.gguf"),
            "qwen3-4b",
            5555,
            "secret",
        );
        let args = spec.args.join(" ");
        for part in [
            "--model /models/m.gguf",
            "--host 127.0.0.1",
            "--port 5555",
            "--n-gpu-layers 999",
            "--ctx-size 8192",
            "--reasoning off",
            "--no-webui",
            "--offline",
            "--alias qwen3-4b",
        ] {
            assert!(args.contains(part), "missing {part} in {args}");
        }
        assert!(!args.contains("secret"));
        assert_eq!(
            spec.envs,
            vec![("LLAMA_API_KEY".to_string(), "secret".to_string())]
        );
    }

    #[test]
    fn a_speech_server_gets_the_projector_a_larger_context_and_no_reasoning_switch() {
        let spec = launch_spec_for(
            ServerRole::Speech,
            Path::new("/app/llama-server"),
            Path::new("/models/asr.gguf"),
            Some(Path::new("/models/mmproj-asr.gguf")),
            "qwen3-asr-1.7b",
            5556,
            "secret",
            1,
        );
        let args = spec.args.join(" ");
        for part in [
            "--model /models/asr.gguf",
            "--mmproj /models/mmproj-asr.gguf",
            "--ctx-size 12288",
            "--alias qwen3-asr-1.7b",
            "--no-webui",
            "--offline",
        ] {
            assert!(args.contains(part), "missing {part} in {args}");
        }
        assert!(!args.contains("--reasoning"));
        assert!(!args.contains("secret"));
        // The AI server's arguments did not change.
        let polish = launch_spec(
            Path::new("/app/llama-server"),
            Path::new("/models/m.gguf"),
            "qwen3-4b",
            5555,
            "secret",
        );
        assert!(!polish.args.join(" ").contains("--mmproj"));
    }

    #[test]
    fn each_language_slot_gets_the_full_context() {
        let spec = launch_spec_for(
            ServerRole::Polish,
            Path::new("/app/llama-server"),
            Path::new("/models/m.gguf"),
            None,
            "qwen3.5-4b",
            5555,
            "secret",
            3,
        );
        let args = spec.args.join(" ");
        assert!(args.contains("--parallel 3"), "{args}");
        assert!(args.contains("--ctx-size 24576"), "{args}");
        let one = launch_spec(
            "/app/llama-server".as_ref(),
            "/m.gguf".as_ref(),
            "m",
            1,
            "k",
        );
        assert!(one.args.join(" ").contains("--parallel 1"));
    }

    #[tokio::test]
    async fn another_number_of_slots_restarts_the_server() {
        let s = setup("slots", true, 1, None);
        s.manager.ensure("a.gguf", "m").await.unwrap();
        s.manager.ensure("a.gguf", "m").await.unwrap();
        assert_eq!(lock(&s.spawned).len(), 1);
        s.manager.set_slots(2);
        s.manager.ensure("a.gguf", "m").await.unwrap();
        let spawned = lock(&s.spawned);
        assert_eq!(spawned.len(), 2);
        assert!(spawned[1].args.join(" ").contains("--parallel 2"));
        drop(spawned);
        assert_eq!(
            s.manager.running_model_and_slots(),
            Some(("a.gguf".to_string(), 2))
        );
    }

    /// `codes` as the router's languages, each with a user hint so it can be recognised
    /// without a preset.
    fn languages_with_hints(config: &mut AppConfig, codes: &[&str]) {
        config.your_languages = Some(codes.iter().map(|code| code.to_string()).collect());
        for code in codes {
            config
                .translation
                .languages
                .entry(code.to_string())
                .or_default()
                .user_hints
                .push(format!("hint-{code}"));
        }
    }

    #[test]
    fn languages_get_their_own_slots_only_with_fast_switching_on() {
        let mut config = AppConfig::default();
        languages_with_hints(&mut config, &["zh-Hant-HK", "en", "ja"]);
        assert!(config.builtin_ai_language_slots, "on by default");
        assert_eq!(wanted_slots(&config, None), 3);
        assert_eq!(slot_for_language(&config, None, "zh-Hant-HK"), Some(0));
        assert_eq!(slot_for_language(&config, None, "ja"), Some(2));
        assert_eq!(slot_for_language(&config, None, "fr"), None);

        config.builtin_ai_language_slots = false;
        assert_eq!(wanted_slots(&config, None), 1);
        assert_eq!(slot_for_language(&config, None, "en"), None);

        // One language needs no pinning.
        config.builtin_ai_language_slots = true;
        languages_with_hints(&mut config, &["en"]);
        assert_eq!(wanted_slots(&config, None), 1);
        assert_eq!(slot_for_language(&config, None, "en"), None);
    }

    /// Plan `warm-builtin-ai`: a language the router cannot recognise (no preset, no hints) never
    /// gets a dictation, so it gets no slot either, and slot numbers skip it.
    #[test]
    fn only_languages_the_router_can_recognise_get_a_slot() {
        // Plan `languages-in-general`: every listed language has speech codes, preset or not.
        let mut config = AppConfig::default();
        config.your_languages = Some(["en", "zh-Hant-HK", "ja"].map(String::from).to_vec());
        assert_eq!(slot_languages(&config, None), ["en", "zh-Hant-HK", "ja"]);
        assert_eq!(slot_for_language(&config, None, "zh-Hant-HK"), Some(1));

        // A language switched off is never routed: no slot, and the numbers skip it.
        config
            .translation
            .languages
            .entry("en".to_string())
            .or_default()
            .enabled = false;
        assert_eq!(slot_languages(&config, None), ["zh-Hant-HK", "ja"]);
        assert_eq!(slot_for_language(&config, None, "zh-Hant-HK"), Some(0));
        assert_eq!(slot_for_language(&config, None, "en"), None);

        for code in ["zh-Hant-HK", "ja"] {
            config
                .translation
                .languages
                .entry(code.to_string())
                .or_default()
                .enabled = false;
        }
        assert_eq!(wanted_slots(&config, None), 1);
    }

    #[test]
    fn slots_stop_at_the_limit() {
        let mut config = AppConfig::default();
        languages_with_hints(&mut config, &["en", "zh-Hant-HK", "ja", "fr"]);
        assert_eq!(
            slot_languages(&config, None).len(),
            MAX_LANGUAGE_SLOTS as usize
        );
        assert_eq!(wanted_slots(&config, None), MAX_LANGUAGE_SLOTS);
    }

    #[tokio::test]
    async fn a_projector_must_exist_and_a_new_one_restarts_the_server() {
        let s = setup("projector", true, 1, None);
        assert_eq!(
            s.manager
                .ensure_with("a.gguf", Some("missing.gguf"), "m")
                .await,
            Err(ServerError::ModelMissing)
        );
        s.manager
            .ensure_with("a.gguf", Some("b.gguf"), "m")
            .await
            .unwrap();
        s.manager
            .ensure_with("a.gguf", Some("b.gguf"), "m")
            .await
            .unwrap();
        assert_eq!(lock(&s.spawned).len(), 1);
        assert!(lock(&s.spawned)[0]
            .args
            .join(" ")
            .contains(&format!("--mmproj {}", s.dir.join("b.gguf").display())));
        // Without the projector it is another server.
        s.manager.ensure("a.gguf", "m").await.unwrap();
        assert_eq!(lock(&s.spawned).len(), 2);
        assert!(lock(&s.children)[0].killed.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn ensure_starts_once_waits_for_health_and_reuses_the_server() {
        let s = setup("reuse", true, 3, None);
        let first = s.manager.ensure("a.gguf", "m").await.unwrap();
        assert!(first.base_url.starts_with("http://127.0.0.1:"));
        assert!(first.base_url.ends_with("/v1"));
        assert_eq!(first.api_key.len(), 32);
        assert_eq!(
            std::fs::read_to_string(s.dir.join("llama-server.pid")).unwrap(),
            "4242"
        );
        let again = s.manager.ensure("a.gguf", "m").await.unwrap();
        assert_eq!(first, again);
        assert_eq!(lock(&s.spawned).len(), 1);
        assert_eq!(s.manager.running_model_file().as_deref(), Some("a.gguf"));

        // Another model restarts it with a new key.
        let other = s.manager.ensure("b.gguf", "m").await.unwrap();
        assert_ne!(other.api_key, first.api_key);
        assert_eq!(lock(&s.spawned).len(), 2);
        assert!(lock(&s.children)[0].killed.load(Ordering::SeqCst));

        s.manager.stop();
        assert!(lock(&s.children)[1].killed.load(Ordering::SeqCst));
        assert!(s.manager.running_model_file().is_none());
        assert!(!s.dir.join("llama-server.pid").exists());
    }

    #[tokio::test]
    async fn a_server_that_died_is_started_again_on_the_next_request() {
        let s = setup("restart", true, 1, None);
        s.manager.ensure("a.gguf", "m").await.unwrap();
        lock(&s.children)[0].killed.store(true, Ordering::SeqCst);
        assert!(s.manager.running_model_file().is_none());
        s.manager.ensure("a.gguf", "m").await.unwrap();
        assert_eq!(lock(&s.spawned).len(), 2);
    }

    #[tokio::test]
    async fn missing_binary_model_exit_and_timeout_are_reported() {
        let s = setup("errors", false, 1, None);
        assert_eq!(
            s.manager.ensure("a.gguf", "m").await,
            Err(ServerError::BinaryMissing)
        );
        assert!(!s.manager.binary_available());
        assert_eq!(
            s.manager.ensure("missing.gguf", "m").await,
            Err(ServerError::ModelMissing)
        );
        assert_eq!(
            s.manager.ensure("", "m").await,
            Err(ServerError::ModelMissing)
        );

        let s = setup("exit", true, usize::MAX, Some(2));
        assert_eq!(
            s.manager.ensure("a.gguf", "m").await,
            Err(ServerError::Exited("exit status: 1".into()))
        );

        let s = setup("timeout", true, usize::MAX, None);
        assert_eq!(
            s.manager.ensure("a.gguf", "m").await,
            Err(ServerError::Timeout)
        );
        assert!(lock(&s.children)[0].killed.load(Ordering::SeqCst));
        assert!(s.manager.running_model_file().is_none());
    }

    #[test]
    fn only_errors_loads_and_timings_reach_the_log() {
        let keep = [
            "0.02.681.426 I srv  llama_server: model loaded",
            "0.02.681.430 I srv  llama_server: listening on http://127.0.0.1:18431",
            "0.13.023.925 I slot print_timing: id  0 | task 24 |       total time =     312.28 ms /    12 tokens",
            "0.00.100.000 E srv  load_model: failed to load model",
            "2.50.074.900 I slot print_timing: id  0 | task 17 | prompt eval time =    8500.12 ms /  3269 tokens (    2.60 ms per token,   384.58 tokens per second)",
            "2.50.074.950 I slot print_timing: id  0 | task 17 |        eval time =     253.11 ms /    60 tokens (    4.22 ms per token,   237.05 tokens per second)",
            "2.50.074.800 I slot  release: id  0 | task 17 | stop processing: n_tokens = 3329, truncated = 0",
        ];
        for line in keep {
            assert!(log_level_for(line).is_some(), "{line}");
        }
        assert_eq!(log_level_for(keep[3]), Some(tracing::Level::WARN));
        for line in [
            "0.12.711.639 W slot   operator(): id  0 | task 24 | n_past was set to 47",
            "0.00.066.332 I srv  llama_server: initializing ...",
            "2.41.500.000 I slot update_slots: id  0 | task 17 | prompt processing, n_tokens =   2048, progress = 0.62, t =   5.10 s / 401.57 tokens per second",
            "ggml_metal_init: found device",
            "",
        ] {
            assert_eq!(log_level_for(line), None, "{line}");
        }
    }

    #[test]
    fn the_server_runs_only_for_an_active_tested_built_in_preset() {
        let mut config = AppConfig::default();
        assert_eq!(wanted_model(&config), None);
        let preset = config.install_builtin_llama("qwen3-4b", "Qwen3-4B-Instruct-2507-Q4_K_M.gguf");
        assert_eq!(wanted_model(&config), None);
        config.mark_ai_verified(&preset, 5);
        assert_eq!(
            wanted_model(&config),
            Some((
                "Qwen3-4B-Instruct-2507-Q4_K_M.gguf".to_string(),
                "qwen3-4b".to_string()
            ))
        );
        config.ai_presets.push(AiPreset::server(
            "mine",
            "Mine",
            "http://192.0.2.3:11434/v1",
            "m",
        ));
        config.active_ai_preset_id = "mine".to_string();
        assert_eq!(wanted_model(&config), None);

        // Plan `language-prompt-library`: translation languages no longer pick a model, so
        // a translation language cannot keep the server running.
        config.translation.targets = vec!["en".to_string(), "zh-Hant-HK".to_string()];
        assert_eq!(wanted_model(&config), None);
    }

    #[tokio::test]
    async fn server_presets_keep_their_own_address_and_key() {
        let preset = AiPreset::server("mine", "Mine", "http://192.0.2.3:11434/v1", "m");
        let (resolved, key) = resolve_preset(&preset, "sk-1".into()).await.unwrap();
        assert_eq!(resolved, preset);
        assert_eq!(key, "sk-1");
    }

    #[test]
    fn stale_process_matcher_accepts_only_llama_server_executables() {
        for expected in [
            "llama-server",
            "llama-server.exe",
            r"C:\OraType\llama-server.exe",
            r"C:\OraType\LLAMA-SERVER.EXE",
            r"C:\OraType\llama-server-x86_64-pc-windows-gnu.exe",
            "/Applications/OraType.app/Contents/MacOS/llama-server",
            "llama-server-aarch64-apple-darwin",
            "llama-server-x8",
        ] {
            assert!(is_server_process_name(expected), "{expected}");
        }
        for rejected in [
            "",
            "svchost.exe",
            r"C:\Windows\System32\svchost.exe",
            r"C:\OraType\oratype.exe",
            r"C:\Tools\ollama-server.exe",
            "not-llama-server.exe",
            "llama-server_other.exe",
        ] {
            assert!(!is_server_process_name(rejected), "{rejected}");
        }
    }

    #[test]
    fn llama_env_var_filter_is_ascii_case_insensitive() {
        assert!(is_llama_env_var("LLAMA_ARG_N_GPU_LAYERS"));
        assert!(is_llama_env_var("llama_api_key"));
        assert!(is_llama_env_var("Llama_Host"));
        assert!(!is_llama_env_var("LLAMA"));
        assert!(!is_llama_env_var("PATH"));
    }

    #[test]
    fn list_devices_parser_reads_vulkan_lines_with_nested_parentheses() {
        let output = "\
0.00.001.000 I srv  llama_server: initializing ...\n\
Available devices:\r\n\
  Vulkan0: AMD Radeon 8060S Graphics (RADV STRIX_HALO) (127488 MiB, 106934 MiB free)\r\n\
  Vulkan1: NVIDIA GeForce RTX 4090 (24142 MiB, 23400 MiB free)\n";
        let devices = parse_llama_list_devices(output);
        assert_eq!(
            devices,
            vec![
                LlamaDevice {
                    id: "Vulkan0".into(),
                    description: "AMD Radeon 8060S Graphics (RADV STRIX_HALO)".into(),
                    total_mib: 127488,
                    free_mib: 106934,
                },
                LlamaDevice {
                    id: "Vulkan1".into(),
                    description: "NVIDIA GeForce RTX 4090".into(),
                    total_mib: 24142,
                    free_mib: 23400,
                },
            ]
        );
    }

    #[test]
    fn vulkan_device_selector_picks_rtx_4090_over_amd_igpu_with_95gb_shared_ram() {
        // Ryzen 9 7950X3D (95 GB RAM -> ~48 GB shared Vulkan heap on iGPU) + RTX 4090 (24 GB).
        let devices = vec![
            LlamaDevice {
                id: "Vulkan0".into(),
                description: "AMD Radeon(TM) Graphics".into(),
                total_mib: 48640,
                free_mib: 46000,
            },
            LlamaDevice {
                id: "Vulkan1".into(),
                description: "NVIDIA GeForce RTX 4090".into(),
                total_mib: 24142,
                free_mib: 23400,
            },
        ];
        // Even though Vulkan0 (iGPU) reports higher total MiB from shared RAM, RTX 4090 wins
        // both with DXGI preference and with heuristic fallback.
        assert_eq!(
            select_best_llama_device(&devices, Some("NVIDIA GeForce RTX 4090")).as_deref(),
            Some("Vulkan1")
        );
        assert_eq!(
            select_best_llama_device(&devices, None).as_deref(),
            Some("Vulkan1")
        );

        // Two discrete GPUs: picks the larger VRAM discrete GPU.
        let dual = vec![
            LlamaDevice {
                id: "Vulkan0".into(),
                description: "NVIDIA GeForce RTX 3060".into(),
                total_mib: 12288,
                free_mib: 11800,
            },
            LlamaDevice {
                id: "Vulkan1".into(),
                description: "NVIDIA GeForce RTX 4090".into(),
                total_mib: 24142,
                free_mib: 23400,
            },
        ];
        assert_eq!(
            select_best_llama_device(&dual, None).as_deref(),
            Some("Vulkan1")
        );
    }

    #[test]
    fn apply_backend_choice_to_args_supports_auto_cpu_and_explicit_vulkan_device() {
        let base = launch_spec_for(
            ServerRole::Speech,
            Path::new("/app/llama-server"),
            Path::new("/models/asr.gguf"),
            Some(Path::new("/models/mmproj.gguf")),
            "qwen3-asr",
            5556,
            "k",
            1,
        );

        let mut auto_args = base.args.clone();
        apply_backend_choice_to_args(&mut auto_args, true, "auto");
        assert_eq!(auto_args, base.args);

        let mut cpu_args = base.args.clone();
        apply_backend_choice_to_args(&mut cpu_args, true, "cpu");
        let joined_cpu = cpu_args.join(" ");
        assert!(joined_cpu.contains("--n-gpu-layers 0"), "{joined_cpu}");
        assert!(joined_cpu.contains("--device none"), "{joined_cpu}");
        assert!(
            joined_cpu.contains("--no-mmproj-offload --mmproj-device none"),
            "{joined_cpu}"
        );

        let mut vk_args = base.args;
        apply_backend_choice_to_args(&mut vk_args, true, "Vulkan1");
        let joined_vk = vk_args.join(" ");
        assert!(joined_vk.contains("--n-gpu-layers 999"), "{joined_vk}");
        assert!(
            joined_vk.contains("--device Vulkan1 --split-mode none"),
            "{joined_vk}"
        );
    }

    #[tokio::test]
    async fn switching_backend_choice_restarts_running_server() {
        let s = setup("backend-restart", true, 1, None);
        s.manager.set_backend_choice("auto");
        let ep1 = s.manager.ensure("a.gguf", "m").await.unwrap();
        let ep2 = s.manager.ensure("a.gguf", "m").await.unwrap();
        assert_eq!(ep1, ep2);
        assert_eq!(lock(&s.spawned).len(), 1);

        s.manager.set_backend_choice("cpu");
        assert!(!s.manager.running_matches("a.gguf", 1, "cpu"));
        let _ep3 = s.manager.ensure("a.gguf", "m").await.unwrap();
        let spawned = lock(&s.spawned);
        assert_eq!(spawned.len(), 2);
        assert!(spawned[1].args.join(" ").contains("--device none"));
        assert!(lock(&s.children)[0].killed.load(Ordering::SeqCst));
    }

    #[test]
    fn log_level_for_forwards_warnings_and_multimodal_vulkan_diagnostics() {
        assert_eq!(log_level_for("01:02:03 W warn"), Some(tracing::Level::WARN));
        assert_eq!(
            log_level_for("01:02:03 I ggml_vulkan: Vulkan0 ready"),
            Some(tracing::Level::INFO)
        );
        assert_eq!(
            log_level_for("01:02:03 I clip_model_load: encode time = 123 ms"),
            Some(tracing::Level::INFO)
        );
        assert_eq!(
            log_level_for("01:02:03 I mtmd_context_init_from_file: done"),
            Some(tracing::Level::INFO)
        );
    }
}
