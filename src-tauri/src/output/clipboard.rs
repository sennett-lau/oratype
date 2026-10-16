use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use crate::error::AppError;

use super::pasteboard::{self, Snapshot};
#[cfg(test)]
use super::InsertStatus;
use super::{InsertResult, InsertionStrategy, OutputMode, TextOutput};

/// Delay after writing to clipboard before simulating paste.
const CLIPBOARD_SETTLE_MS: u64 = 20;
const CLIPBOARD_RESTORE_DELAY_MS: u64 = 750;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasteShortcut {
    CtrlV,
    CtrlShiftV,
    ShiftInsert,
}

impl PasteShortcut {
    pub fn from_config_value(value: &str) -> Self {
        match value {
            "ctrlShiftV" => Self::CtrlShiftV,
            "shiftInsert" => Self::ShiftInsert,
            _ => Self::CtrlV,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClipboardOutputOptions {
    pub restore_after_paste: bool,
    pub paste_shortcut: PasteShortcut,
    pub auto_paste: bool,
}

impl Default for ClipboardOutputOptions {
    fn default() -> Self {
        Self {
            restore_after_paste: true,
            paste_shortcut: PasteShortcut::CtrlV,
            auto_paste: true,
        }
    }
}

pub struct ClipboardOutput {
    options: ClipboardOutputOptions,
}

impl Default for ClipboardOutput {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardOutput {
    pub fn new() -> Self {
        Self::with_options(ClipboardOutputOptions::default())
    }

    pub fn with_options(options: ClipboardOutputOptions) -> Self {
        Self { options }
    }
}

#[cfg(any(target_os = "linux", test))]
fn should_auto_paste_after_clipboard(session_type: &str) -> bool {
    !session_type.eq_ignore_ascii_case("wayland")
}

/// Plan `clipboard-safety`: whether to put the user's clipboard back after a paste. Only when the
/// paste went through, restoring is on, there is something to restore, and the clipboard still
/// holds what Typelite wrote (no app or person copied something since).
fn should_restore_clipboard(
    restore_enabled: bool,
    paste_succeeded: bool,
    have_previous: bool,
    still_ours: bool,
) -> bool {
    restore_enabled && paste_succeeded && have_previous && still_ours
}

fn clipboard_insert_result(paste_succeeded: bool, inserted_text: &str) -> InsertResult {
    if paste_succeeded {
        InsertResult::inserted(
            InsertionStrategy::ClipboardPaste,
            inserted_text.chars().count(),
        )
    } else {
        InsertResult::copied_fallback(
            InsertionStrategy::ClipboardPaste,
            inserted_text.chars().count(),
        )
    }
}

/// The restore waiting after a paste. Pastes in quick succession share one: the clipboard from
/// before the first is the one that comes back, after the last.
#[derive(Debug)]
struct PendingClipboardRestore {
    latest_restore_id: u64,
    original: Option<Snapshot>,
}

static NEXT_CLIPBOARD_RESTORE_ID: AtomicU64 = AtomicU64::new(1);
static PENDING_CLIPBOARD_RESTORE: Mutex<Option<PendingClipboardRestore>> = Mutex::new(None);

/// Puts `previous` back after [`CLIPBOARD_RESTORE_DELAY_MS`], if the clipboard still holds the
/// pasted text then (its change count is still `written`).
fn schedule_clipboard_restore(previous: Option<Snapshot>, written: i64) {
    let (restore_id, original) = remember_pending_clipboard_restore(previous);
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_RESTORE_DELAY_MS));
        restore_clipboard_if_safe(original, written, restore_id);
    });
}

fn remember_pending_clipboard_restore(previous: Option<Snapshot>) -> (u64, Option<Snapshot>) {
    let restore_id = NEXT_CLIPBOARD_RESTORE_ID.fetch_add(1, Ordering::SeqCst);
    let mut pending = PENDING_CLIPBOARD_RESTORE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let original = match pending.take() {
        Some(batch) => batch.original,
        None => previous,
    };
    *pending = Some(PendingClipboardRestore {
        latest_restore_id: restore_id,
        original: original.clone(),
    });
    (restore_id, original)
}

fn is_latest_clipboard_restore(restore_id: u64) -> bool {
    matches!(
        PENDING_CLIPBOARD_RESTORE
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref(),
        Some(batch) if batch.latest_restore_id == restore_id
    )
}

fn clear_pending_clipboard_restore(restore_id: u64) {
    let mut pending = PENDING_CLIPBOARD_RESTORE
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if matches!(pending.as_ref(), Some(batch) if batch.latest_restore_id == restore_id) {
        pending.take();
    }
}

fn restore_clipboard_if_safe(original: Option<Snapshot>, written: i64, restore_id: u64) {
    if !is_latest_clipboard_restore(restore_id) {
        return;
    }
    let still_ours = pasteboard::change_count() == Some(written);
    if should_restore_clipboard(true, true, original.is_some(), still_ours) {
        if let Some(original) = &original {
            if !pasteboard::restore(original) {
                tracing::warn!("Failed to restore the previous clipboard");
            }
        }
    } else if !still_ours {
        tracing::debug!("Clipboard changed after the paste; not restoring it");
    }
    clear_pending_clipboard_restore(restore_id);
}

#[cfg(target_os = "macos")]
fn simulate_paste(_shortcut: PasteShortcut) -> Result<(), AppError> {
    let status = std::process::Command::new("osascript")
        .args([
            "-e",
            r#"tell application "System Events" to keystroke "v" using command down"#,
        ])
        .status()
        .map_err(|e| AppError::Output(format!("osascript error: {}", e)))?;
    if !status.success() {
        return Err(AppError::Output(format!(
            "osascript paste failed with exit code: {:?}",
            status.code()
        )));
    }
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn paste_keys(shortcut: PasteShortcut) -> (Vec<enigo::Key>, enigo::Key) {
    use enigo::Key;
    match shortcut {
        PasteShortcut::CtrlV => (vec![Key::Control], Key::Unicode('v')),
        PasteShortcut::CtrlShiftV => (vec![Key::Control, Key::Shift], Key::Unicode('v')),
        PasteShortcut::ShiftInsert => (vec![Key::Shift], Key::Insert),
    }
}

#[cfg(not(target_os = "macos"))]
fn simulate_paste(shortcut: PasteShortcut) -> Result<(), AppError> {
    super::windows_modifier_guard::wait_for_modifier_release()?;

    use enigo::{Direction, Enigo, Keyboard, Settings};
    let (modifiers, primary) = paste_keys(shortcut);
    let mut enigo = Enigo::new(&Settings::default())
        .map_err(|e| AppError::Output(format!("Failed to create Enigo: {:?}", e)))?;

    let mut pressed = 0usize;
    let mut first_error: Option<String> = None;

    for modifier in &modifiers {
        if let Err(error) = enigo.key(*modifier, Direction::Press) {
            first_error = Some(format!("Key press error: {:?}", error));
            break;
        }
        pressed += 1;
    }

    if first_error.is_none() {
        if let Err(error) = enigo.key(primary, Direction::Click) {
            first_error = Some(format!("Key click error: {:?}", error));
        }
    }

    for modifier in modifiers[..pressed].iter().rev() {
        if let Err(error) = enigo.key(*modifier, Direction::Release) {
            if first_error.is_none() {
                first_error = Some(format!("Key release error: {:?}", error));
            }
        }
    }

    match first_error {
        Some(error) => Err(AppError::Output(error)),
        None => Ok(()),
    }
}

#[async_trait]
impl TextOutput for ClipboardOutput {
    async fn type_text(&self, text: &str) -> Result<InsertResult, AppError> {
        let text = text.to_string();
        let options = self.options;
        tokio::task::spawn_blocking(move || -> Result<InsertResult, AppError> {
            if !options.auto_paste {
                // The user asked for the text (Copy): a normal copy that stays.
                pasteboard::write_text(&text)
                    .then_some(())
                    .ok_or_else(|| AppError::Output("Failed to set clipboard".to_string()))?;
                return Ok(InsertResult::copied_fallback(
                    InsertionStrategy::ClipboardCopyOnly,
                    text.chars().count(),
                ));
            }

            // Plan `clipboard-safety`: keep everything that was there (images, files, formatted
            // text), and mark the pasted text as transient so clipboard managers skip it. With
            // restoring off the text is meant to stay, so it is a normal copy.
            let previous = options
                .restore_after_paste
                .then(pasteboard::snapshot)
                .flatten();
            let written = if options.restore_after_paste {
                pasteboard::write_transient_text(&text)
            } else {
                pasteboard::write_text(&text)
                    .then(pasteboard::change_count)
                    .flatten()
            }
            .ok_or_else(|| AppError::Output("Failed to set clipboard".to_string()))?;

            std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_SETTLE_MS));

            #[cfg(target_os = "linux")]
            if !should_auto_paste_after_clipboard(&crate::platform::current_session_type()) {
                return Ok(InsertResult::copied_fallback(
                    InsertionStrategy::ClipboardCopyOnly,
                    text.chars().count(),
                ));
            }

            if let Err(error) = simulate_paste(options.paste_shortcut) {
                // The text must not be lost: leave it on the clipboard as a normal copy.
                tracing::warn!("Clipboard paste failed; leaving output text on clipboard: {error}");
                pasteboard::write_text(&text);
                return Ok(clipboard_insert_result(false, &text));
            }

            if options.restore_after_paste {
                schedule_clipboard_restore(previous, written);
            }

            Ok(clipboard_insert_result(true, &text))
        })
        .await
        .map_err(|e| AppError::Output(format!("Spawn blocking error: {}", e)))?
    }

    fn mode(&self) -> OutputMode {
        OutputMode::Clipboard
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_clipboard_output_is_copy_only() {
        assert!(!should_auto_paste_after_clipboard("wayland"));
        assert!(!should_auto_paste_after_clipboard("WAYLAND"));
    }

    #[test]
    fn x11_clipboard_output_keeps_auto_paste() {
        assert!(should_auto_paste_after_clipboard("x11"));
        assert!(should_auto_paste_after_clipboard("unknown"));
    }

    #[test]
    fn the_clipboard_comes_back_only_after_a_successful_paste_it_still_holds() {
        assert!(should_restore_clipboard(true, true, true, true));
        assert!(
            !should_restore_clipboard(true, false, true, true),
            "paste failed"
        );
        assert!(
            !should_restore_clipboard(true, true, true, false),
            "something was copied since"
        );
        assert!(
            !should_restore_clipboard(false, true, true, true),
            "restoring is off"
        );
        assert!(
            !should_restore_clipboard(true, true, false, true),
            "nothing to restore"
        );
    }

    #[test]
    fn quick_pastes_restore_the_clipboard_from_before_the_first() {
        let before = pasteboard::text_snapshot("what the user copied");
        let (first_id, first) = remember_pending_clipboard_restore(Some(before.clone()));
        assert_eq!(first.as_ref(), Some(&before));
        // A second paste before the restore: its "previous" is Typelite's own first text.
        let (second_id, second) =
            remember_pending_clipboard_restore(Some(pasteboard::text_snapshot("first dictation")));
        assert_eq!(second.as_ref(), Some(&before));
        assert!(!is_latest_clipboard_restore(first_id));
        assert!(is_latest_clipboard_restore(second_id));
        clear_pending_clipboard_restore(second_id);
        assert!(!is_latest_clipboard_restore(second_id));
    }

    #[test]
    fn paste_shortcut_defaults_unknown_values_to_ctrl_v() {
        assert_eq!(PasteShortcut::from_config_value(""), PasteShortcut::CtrlV);
        assert_eq!(
            PasteShortcut::from_config_value("something-else"),
            PasteShortcut::CtrlV
        );
    }

    #[test]
    fn paste_shortcut_parses_terminal_friendly_shortcuts() {
        assert_eq!(
            PasteShortcut::from_config_value("ctrlShiftV"),
            PasteShortcut::CtrlShiftV
        );
        assert_eq!(
            PasteShortcut::from_config_value("shiftInsert"),
            PasteShortcut::ShiftInsert
        );
    }

    #[test]
    fn clipboard_result_marks_paste_failure_as_copied_fallback() {
        let result = clipboard_insert_result(false, "typed text");

        assert_eq!(result.status, InsertStatus::CopiedFallback);
        assert_eq!(result.strategy_used, InsertionStrategy::ClipboardPaste);
        assert_eq!(result.chars_inserted, 0);
        assert_eq!(result.chars_copied, 10);
    }
}
