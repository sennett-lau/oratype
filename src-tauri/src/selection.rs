//! Reading the user's highlight for Ask, Translate and (optionally) Dictate.
//!
//! Plan `clipboard-safety`: Accessibility first. Most apps report the focused element's selected
//! text (`AXSelectedText`), or that nothing is selected, and the clipboard is not involved at
//! all. Only an app that does not tell gets a ⌘C. Then the clipboard is copied as a whole first,
//! its change count tells whether the app copied anything, and everything that was there is put
//! back. If the app copied nothing, the clipboard was never written. An app that answers the ⌘C
//! after OraType stopped waiting would overwrite the clipboard, so it is watched a little longer
//! and put back.

#[cfg(not(target_os = "macos"))]
use enigo::{Direction, Enigo, Key, Keyboard, Settings as EnigoSettings};

use crate::output::focus::{self, SelectedText};
use crate::output::pasteboard;

/// How long to wait for the app to put the selection on the clipboard after ⌘C. Browsers can
/// take well over 100 ms, so the clipboard is polled instead of read once.
const CLIPBOARD_COPY_TIMEOUT_MS: u64 = 500;
const CLIPBOARD_POLL_MS: u64 = 25;
/// After a ⌘C that copied nothing in time: how long a late copy is still put back.
const LATE_COPY_WATCH_MS: u64 = 1500;
/// How long to wait for the shortcut's own modifier keys (Control, Shift, ...) to be released,
/// so the app sees a plain ⌘C and not, say, ⌃⌘C.
const MODIFIER_RELEASE_TIMEOUT_MS: u64 = 400;

/// Plan `translate-selection-panel`: whether a run reads the highlight. Ask (answer, translate,
/// edit the highlight) and Translate (translate the highlight in place) always do; it is what
/// they are for. Dictate does only when the user turned on "Use selected text as polish
/// context" (`selected_text_enabled`), since it sends the highlight to the AI on every dictation.
pub fn should_capture_selection(
    mode: crate::voice_intent::VoiceMode,
    selected_text_enabled: bool,
) -> bool {
    match mode {
        crate::voice_intent::VoiceMode::Ask | crate::voice_intent::VoiceMode::Translate => true,
        crate::voice_intent::VoiceMode::Dictate => selected_text_enabled,
    }
}

/// Polls `read` (the clipboard's change count) every `poll` until it differs from `before` (the
/// app copied) or `timeout` passes. True when it changed.
fn wait_for_clipboard_change(
    mut read: impl FnMut() -> Option<i64>,
    before: i64,
    timeout: std::time::Duration,
    poll: std::time::Duration,
) -> bool {
    let deadline = std::time::Instant::now() + timeout;
    loop {
        if read().is_some_and(|count| count != before) {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        std::thread::sleep(poll);
    }
}

/// Whether osascript's error output says macOS refused the Apple Event to System Events
/// (Automation permission, error -1743) or the keystroke (Accessibility, error 1002).
fn is_permission_error(stderr: &str) -> bool {
    stderr.contains("-1743") || stderr.contains("1002") || stderr.contains("not allowed")
}

#[cfg(target_os = "macos")]
mod modifiers {
    // CGEventSourceFlagsState reads the modifier keys held right now (no permission needed).
    #[link(name = "CoreGraphics", kind = "framework")]
    extern "C" {
        fn CGEventSourceFlagsState(state_id: i32) -> u64;
    }
    /// kCGEventSourceStateHIDSystemState: the hardware state.
    const HID_SYSTEM_STATE: i32 = 1;
    /// Shift, Control, Option and Command. Fn is left out: in hold mode it stays down for the
    /// whole recording and does not change ⌘C.
    const MASK: u64 = 0x0002_0000 | 0x0004_0000 | 0x0008_0000 | 0x0010_0000;

    pub fn any_held() -> bool {
        // SAFETY: a plain query with a valid state id.
        unsafe { CGEventSourceFlagsState(HID_SYSTEM_STATE) & MASK != 0 }
    }
}

#[cfg(target_os = "macos")]
fn wait_for_modifier_release() {
    let deadline =
        std::time::Instant::now() + std::time::Duration::from_millis(MODIFIER_RELEASE_TIMEOUT_MS);
    while modifiers::any_held() {
        if std::time::Instant::now() >= deadline {
            tracing::debug!("Selected text: modifier keys still held; copying anyway");
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(CLIPBOARD_POLL_MS));
    }
}

#[cfg(not(target_os = "macos"))]
fn wait_for_modifier_release() {}

#[cfg(target_os = "macos")]
fn copy_selected_text_to_clipboard() -> bool {
    // The same System Events route as the paste (⌘V) in `output/clipboard.rs`, so it needs no
    // permission the paste does not already need: Accessibility, and Automation of System
    // Events (asked once, on first use). Marked as OraType's own, so its ⌘ cannot fire a
    // shortcut on a bare Command key (plan `own-keystrokes`).
    let _own = crate::native_hotkey::OwnKeystroke::begin();
    match std::process::Command::new("/usr/bin/osascript")
        .args([
            "-e",
            r#"tell application "System Events" to keystroke "c" using command down"#,
        ])
        .output()
    {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if is_permission_error(&stderr) {
                tracing::warn!(
                    "Selected text: macOS refused the copy keystroke. Allow OraType in System \
                     Settings → Privacy & Security → Accessibility and → Automation → System Events"
                );
            } else {
                tracing::warn!(
                    "macOS selected-text copy failed with exit code: {:?}",
                    output.status.code()
                );
            }
            false
        }
        Err(e) => {
            tracing::warn!("Failed to run osascript for selected-text copy: {}", e);
            false
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn copy_selected_text_to_clipboard() -> bool {
    let Ok(mut enigo) = Enigo::new(&EnigoSettings::default()) else {
        return false;
    };

    let pressed = enigo.key(Key::Control, Direction::Press).is_ok();
    if pressed {
        let _ = enigo.key(Key::Unicode('c'), Direction::Click);
        let _ = enigo.key(Key::Control, Direction::Release);
    }
    pressed
}

/// The user's highlight, or `None` when nothing is selected or it cannot be read.
pub fn capture_selected_text() -> Option<String> {
    wait_for_modifier_release();
    let started = std::time::Instant::now();
    match focus::focused_selected_text() {
        SelectedText::Text(text) => {
            tracing::info!(
                "Selected text: {} chars through Accessibility in {} ms",
                text.chars().count(),
                started.elapsed().as_millis()
            );
            return (!text.trim().is_empty()).then_some(text);
        }
        SelectedText::Nothing => {
            tracing::info!(
                "Selected text: none (Accessibility) in {} ms",
                started.elapsed().as_millis()
            );
            return None;
        }
        SelectedText::Unknown => {}
    }
    capture_with_copy(started)
}

/// ⌘C for an app that does not report its selection through Accessibility.
fn capture_with_copy(started: std::time::Instant) -> Option<String> {
    let before = pasteboard::change_count()?;
    let saved = pasteboard::snapshot();
    if !copy_selected_text_to_clipboard() {
        tracing::debug!("Selected text copy shortcut could not be sent");
        return None;
    }
    let copied = wait_for_clipboard_change(
        pasteboard::change_count,
        before,
        std::time::Duration::from_millis(CLIPBOARD_COPY_TIMEOUT_MS),
        std::time::Duration::from_millis(CLIPBOARD_POLL_MS),
    );
    if !copied {
        // Nothing was copied, so the clipboard is untouched.
        tracing::info!(
            "Selected text: none (⌘C copied nothing in {} ms)",
            started.elapsed().as_millis()
        );
        if let Some(saved) = saved {
            put_back_a_late_copy(before, saved);
        }
        return None;
    }
    let selected = pasteboard::read_text();
    let restored = saved.as_ref().is_some_and(pasteboard::restore);
    tracing::info!(
        "Selected text: {} chars through ⌘C in {} ms, clipboard {}",
        selected
            .as_deref()
            .map(|text| text.chars().count())
            .unwrap_or(0),
        started.elapsed().as_millis(),
        if restored { "restored" } else { "not restored" }
    );
    selected.filter(|text| !text.trim().is_empty())
}

/// An app that answers the ⌘C after OraType stopped waiting would leave the selection on the
/// clipboard. For a short while, a change is taken as that late copy and the clipboard is put
/// back once.
fn put_back_a_late_copy(before: i64, saved: pasteboard::Snapshot) {
    std::thread::spawn(move || {
        if wait_for_clipboard_change(
            pasteboard::change_count,
            before,
            std::time::Duration::from_millis(LATE_COPY_WATCH_MS),
            std::time::Duration::from_millis(CLIPBOARD_POLL_MS),
        ) {
            let restored = pasteboard::restore(&saved);
            tracing::info!(
                "Selected text: an app copied after the wait; clipboard {}",
                if restored { "restored" } else { "not restored" }
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::voice_intent::VoiceMode;

    #[test]
    fn ask_and_translate_always_read_the_highlight_dictate_only_when_enabled() {
        for enabled in [false, true] {
            assert!(should_capture_selection(VoiceMode::Ask, enabled));
            assert!(should_capture_selection(VoiceMode::Translate, enabled));
        }
        assert!(!should_capture_selection(VoiceMode::Dictate, false));
        assert!(should_capture_selection(VoiceMode::Dictate, true));
    }

    #[test]
    fn the_change_count_is_polled_until_the_app_copies() {
        let mut reads = vec![7, 7, 8].into_iter();
        assert!(wait_for_clipboard_change(
            || reads.next(),
            7,
            std::time::Duration::from_secs(1),
            std::time::Duration::from_millis(1),
        ));
    }

    #[test]
    fn polling_gives_up_when_nothing_was_copied() {
        assert!(!wait_for_clipboard_change(
            || Some(7),
            7,
            std::time::Duration::from_millis(20),
            std::time::Duration::from_millis(5),
        ));
        // An unreadable count is not a change.
        assert!(!wait_for_clipboard_change(
            || None,
            7,
            std::time::Duration::from_millis(20),
            std::time::Duration::from_millis(5),
        ));
    }

    #[test]
    fn permission_refusals_are_recognised() {
        assert!(is_permission_error(
            "execution error: Not authorized to send Apple events to System Events. (-1743)"
        ));
        assert!(is_permission_error(
            "System Events got an error: osascript is not allowed to send keystrokes. (1002)"
        ));
        assert!(!is_permission_error("syntax error"));
    }
}
