//! Plan `rename-oratype`: Typelite became OraType in 1.2.0, bundle identifier included
//! (`dev.typelite.mac` → `dev.oratype.mac`). macOS keys an app's folders and login item by its
//! identifier and name, so on the first launch after the update this moves them across, before
//! anything reads them:
//!
//! - `~/Library/Application Support/<old id>` → `<new id>`: settings, the database (renamed
//!   `oratype.db`) and the downloaded models. A move on the same disk, so gigabytes of models
//!   take no time.
//! - `~/Library/WebKit/<old id>` → `<new id>`: the windows' local storage.
//! - `~/Library/LaunchAgents/<old name>.plist` is removed. "Launch at login" registers OraType
//!   again from the setting (`sync_auto_start_preference`).
//! - An app bundle still called `Typelite.app` is renamed `OraType.app` and started again from
//!   there: an in-place update keeps the old folder name.
//!
//! API keys move on first use (`credentials.rs`). macOS permissions (Accessibility, Microphone,
//! Automation) belong to the identifier and cannot be moved: they are granted again once.
//! Everything here is a no-op when there is nothing to move, so it runs on every launch.

#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

use std::path::{Path, PathBuf};

#[cfg(not(feature = "dev-build"))]
mod names {
    pub const OLD_ID: &str = "dev.typelite.mac";
    pub const NEW_ID: &str = "dev.oratype.mac";
    pub const OLD_NAME: &str = "Typelite";
    pub const NEW_NAME: &str = "OraType";
}
#[cfg(feature = "dev-build")]
mod names {
    pub const OLD_ID: &str = "dev.typelite.mac.dev";
    pub const NEW_ID: &str = "dev.oratype.mac.dev";
    pub const OLD_NAME: &str = "Typelite Dev";
    pub const NEW_NAME: &str = "OraType Dev";
}
use names::{NEW_ID, NEW_NAME, OLD_ID, OLD_NAME};

/// The database Typelite kept in its data folder, and its SQLite side files.
const OLD_DATABASE: &str = "typelite.db";
const NEW_DATABASE: &str = "oratype.db";
const DATABASE_SIDE_FILES: [&str; 3] = ["", "-wal", "-shm"];

/// What the caller does next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// Carry on starting.
    Continue,
    /// The bundle was renamed and the app started again from its new place: exit now.
    Exit,
}

/// Runs the migration (macOS only). Call first thing at startup, after logging is set up.
pub fn run() -> Next {
    #[cfg(target_os = "macos")]
    {
        let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
            return Next::Continue;
        };
        move_folders(&home);
        if let Ok(exe) = std::env::current_exe() {
            if let Some((from, to)) = bundle_rename(&exe) {
                return relaunch_renamed(&from, &to);
            }
        }
    }
    Next::Continue
}

/// Moves Typelite's folders to OraType's under `home`, and removes its login item.
fn move_folders(home: &Path) {
    let library = home.join("Library");
    for parent in ["Application Support", "WebKit"] {
        let parent = library.join(parent);
        move_folder(&parent.join(OLD_ID), &parent.join(NEW_ID));
    }
    rename_database(&library.join("Application Support").join(NEW_ID));
    let agent = library
        .join("LaunchAgents")
        .join(format!("{OLD_NAME}.plist"));
    if agent.exists() {
        match std::fs::remove_file(&agent) {
            Ok(()) => tracing::info!("Rename: removed Typelite's login item"),
            Err(error) => tracing::warn!("Rename: could not remove Typelite's login item: {error}"),
        }
    }
}

/// Moves `from` to `to` when only `from` exists. When both exist OraType has run before, so its
/// own folder wins and Typelite's is left alone.
fn move_folder(from: &Path, to: &Path) {
    if !from.is_dir() {
        return;
    }
    if to.exists() {
        tracing::info!(
            "Rename: {} and {} both exist; keeping OraType's",
            from.display(),
            to.display()
        );
        return;
    }
    match std::fs::rename(from, to) {
        Ok(()) => tracing::info!("Rename: moved {} to {}", from.display(), to.display()),
        Err(error) => tracing::warn!("Rename: could not move {}: {error}", from.display()),
    }
}

/// Renames `typelite.db` (and its `-wal` / `-shm` files) to `oratype.db` in `dir`, unless
/// OraType's database already exists.
fn rename_database(dir: &Path) {
    if !dir.join(OLD_DATABASE).exists() || dir.join(NEW_DATABASE).exists() {
        return;
    }
    for suffix in DATABASE_SIDE_FILES {
        let from = dir.join(format!("{OLD_DATABASE}{suffix}"));
        if from.exists() {
            if let Err(error) = std::fs::rename(&from, dir.join(format!("{NEW_DATABASE}{suffix}")))
            {
                tracing::warn!("Rename: could not rename {}: {error}", from.display());
            }
        }
    }
    tracing::info!("Rename: renamed the database to {NEW_DATABASE}");
}

/// The bundle to rename and its new path, when the running app sits in a bundle still called
/// `Typelite.app` (an update replaces the bundle's contents in place) and nothing is in the way.
/// A bundle macOS runs from a randomised read-only copy (App Translocation, for a downloaded app
/// opened in place) is left alone.
fn bundle_rename(exe: &Path) -> Option<(PathBuf, PathBuf)> {
    let bundle = exe
        .ancestors()
        .find(|path| path.extension().is_some_and(|ext| ext == "app"))?;
    if bundle.file_name()? != std::ffi::OsStr::new(&format!("{OLD_NAME}.app")) {
        return None;
    }
    if bundle
        .components()
        .any(|part| part.as_os_str() == "AppTranslocation")
    {
        return None;
    }
    let target = bundle.parent()?.join(format!("{NEW_NAME}.app"));
    (!target.exists()).then(|| (bundle.to_path_buf(), target))
}

/// Renames the bundle and opens it from its new path. On failure the app just keeps running
/// from the old one.
#[cfg(target_os = "macos")]
fn relaunch_renamed(from: &Path, to: &Path) -> Next {
    if let Err(error) = std::fs::rename(from, to) {
        tracing::warn!("Rename: could not rename {}: {error}", from.display());
        return Next::Continue;
    }
    tracing::info!("Rename: renamed {} to {}", from.display(), to.display());
    // `open -n` starts a new instance from the new path; this one exits before it sets up
    // anything (single instance, tray, shortcuts).
    match std::process::Command::new("/usr/bin/open")
        .arg("-n")
        .arg(to)
        .spawn()
    {
        Ok(_) => Next::Exit,
        Err(error) => {
            tracing::warn!("Rename: could not start {}: {error}", to.display());
            Next::Continue
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_home(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("oratype-rename-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(path: &Path, text: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, text).unwrap();
    }

    #[test]
    fn typelites_folders_move_across_with_the_database_renamed() {
        let home = temp_home("move");
        let old = home.join("Library/Application Support").join(OLD_ID);
        write(&old.join("settings.json"), "{}");
        write(&old.join("models/ggml-base-q5_1.bin"), "model");
        write(&old.join(OLD_DATABASE), "db");
        write(&old.join(format!("{OLD_DATABASE}-wal")), "wal");
        write(&home.join("Library/WebKit").join(OLD_ID).join("data"), "x");
        let agent = home
            .join("Library/LaunchAgents")
            .join(format!("{OLD_NAME}.plist"));
        write(&agent, "plist");

        move_folders(&home);

        let new = home.join("Library/Application Support").join(NEW_ID);
        assert!(!old.exists());
        assert_eq!(
            std::fs::read_to_string(new.join("settings.json")).unwrap(),
            "{}"
        );
        assert!(new.join("models/ggml-base-q5_1.bin").exists());
        assert_eq!(
            std::fs::read_to_string(new.join(NEW_DATABASE)).unwrap(),
            "db"
        );
        assert!(new.join(format!("{NEW_DATABASE}-wal")).exists());
        assert!(!new.join(OLD_DATABASE).exists());
        assert!(home
            .join("Library/WebKit")
            .join(NEW_ID)
            .join("data")
            .exists());
        assert!(!agent.exists());
        // A second run has nothing to do.
        move_folders(&home);
        assert!(new.join(NEW_DATABASE).exists());
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn an_existing_oratype_folder_wins() {
        let home = temp_home("both");
        let support = home.join("Library/Application Support");
        write(&support.join(OLD_ID).join("settings.json"), "old");
        write(&support.join(NEW_ID).join("settings.json"), "new");
        move_folders(&home);
        assert_eq!(
            std::fs::read_to_string(support.join(NEW_ID).join("settings.json")).unwrap(),
            "new"
        );
        assert!(support.join(OLD_ID).exists());
        let _ = std::fs::remove_dir_all(&home);
    }

    #[test]
    fn only_a_bundle_still_called_typelite_is_renamed() {
        let apps = temp_home("bundle");
        let exe = |bundle: &str| apps.join(bundle).join("Contents/MacOS/oratype");
        assert_eq!(
            bundle_rename(&exe(&format!("{OLD_NAME}.app"))),
            Some((
                apps.join(format!("{OLD_NAME}.app")),
                apps.join(format!("{NEW_NAME}.app"))
            ))
        );
        // Already renamed, renamed by the user, or not in a bundle (a development run).
        assert_eq!(bundle_rename(&exe(&format!("{NEW_NAME}.app"))), None);
        assert_eq!(bundle_rename(&exe("My Dictation.app")), None);
        assert_eq!(bundle_rename(Path::new("/repo/target/debug/oratype")), None);
        // Something already called OraType.app is in the way.
        std::fs::create_dir_all(apps.join(format!("{NEW_NAME}.app"))).unwrap();
        assert_eq!(bundle_rename(&exe(&format!("{OLD_NAME}.app"))), None);
        // A translocated copy is read-only.
        let translocated = PathBuf::from(format!(
            "/private/var/folders/x/AppTranslocation/ABC/d/{OLD_NAME}.app/Contents/MacOS/oratype"
        ));
        assert_eq!(bundle_rename(&translocated), None);
        let _ = std::fs::remove_dir_all(&apps);
    }
}
