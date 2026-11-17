//! Windows foreground-window tracking and focus restoration around recording/paste.
//!
//! Showing the capsule overlay or racing with background UI transitions can move Win32
//! `GetForegroundWindow()` away from the text field the user was focused on when dictation
//! started. This module records the non-overlay foreground window before `Preparing` updates
//! the UI and restores it (with `AttachThreadInput` fallback) right before text output.

use std::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundWindowInfo {
    pub hwnd: isize,
    pub pid: u32,
    pub thread_id: u32,
    pub exe_name: String,
}

static LAST_NON_OVERLAY_FOREGROUND: Mutex<Option<ForegroundWindowInfo>> = Mutex::new(None);
#[cfg(target_os = "windows")]
static ACTIVE_RECORDING_TARGET: Mutex<Option<ForegroundWindowInfo>> = Mutex::new(None);

/// Pure decision helper for unit testing: returns `true` when we have a valid recorded
/// target window and the current foreground `HWND` is different (or missing).
pub fn should_attempt_focus_restore(
    recorded: Option<&ForegroundWindowInfo>,
    current: Option<&ForegroundWindowInfo>,
) -> bool {
    let Some(target) = recorded else {
        return false;
    };
    if target.hwnd == 0 {
        return false;
    }
    current.map(|fg| fg.hwnd) != Some(target.hwnd)
}

/// A window as the log names it: its handle and process id. The program's name is left out on
/// purpose: the log is a file users send with a bug report, and it must not record which apps
/// they dictate into (the macOS log does not name the app either).
pub fn format_foreground_summary(info: Option<&ForegroundWindowInfo>) -> String {
    match info {
        Some(fg) => format!("hwnd=0x{:x}, pid={}", fg.hwnd as usize, fg.pid),
        None => "hwnd=0x0, pid=0".to_string(),
    }
}

pub fn last_non_overlay_hwnd() -> Option<isize> {
    LAST_NON_OVERLAY_FOREGROUND
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .as_ref()
        .map(|info| info.hwnd)
}

#[cfg(target_os = "windows")]
pub fn remember_non_overlay_hwnd(hwnd: isize) {
    if hwnd == 0 || crate::overlay_window::is_own_overlay_hwnd(hwnd) {
        return;
    }
    if let Some(info) = unsafe { win::inspect_hwnd(hwnd as _) } {
        *LAST_NON_OVERLAY_FOREGROUND
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(info);
    }
}

#[cfg(not(target_os = "windows"))]
pub fn remember_non_overlay_hwnd(_hwnd: isize) {}

/// Captures the target window at recording start BEFORE showing `Preparing` capsule UI.
#[cfg(target_os = "windows")]
pub fn capture_recording_start_foreground() -> Option<ForegroundWindowInfo> {
    let fg = unsafe { win::find_non_overlay_foreground() }.or_else(|| {
        LAST_NON_OVERLAY_FOREGROUND
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .filter(|info| unsafe { win::is_valid_target_hwnd(info.hwnd) })
    });
    if let Some(info) = &fg {
        *LAST_NON_OVERLAY_FOREGROUND
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(info.clone());
    }
    *ACTIVE_RECORDING_TARGET
        .lock()
        .unwrap_or_else(|e| e.into_inner()) = fg.clone();
    tracing::info!(
        "Windows foreground at recording start: {}",
        format_foreground_summary(fg.as_ref())
    );
    fg
}

#[cfg(not(target_os = "windows"))]
pub fn capture_recording_start_foreground() -> Option<ForegroundWindowInfo> {
    None
}

/// Logs the current foreground window right before output/paste, and restores focus back
/// to the recording-start target if another window (including an overlay) took foreground.
#[cfg(target_os = "windows")]
pub fn ensure_recording_target_foreground_before_paste() -> Option<ForegroundWindowInfo> {
    let recorded = ACTIVE_RECORDING_TARGET
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    let current = unsafe { win::current_foreground_info() };
    tracing::info!(
        "Windows foreground before paste: current=({}) recorded=({})",
        format_foreground_summary(current.as_ref()),
        format_foreground_summary(recorded.as_ref())
    );

    if !should_attempt_focus_restore(recorded.as_ref(), current.as_ref()) {
        return current;
    }

    let Some(target) = recorded else {
        return current;
    };
    if !unsafe { win::is_valid_target_hwnd(target.hwnd) } {
        tracing::info!(
            "Windows focus restore skipped: recorded target hwnd=0x{:x} no longer exists",
            target.hwnd as usize
        );
        return current;
    }

    let restored = unsafe { win::restore_foreground_hwnd(&target) };
    std::thread::sleep(std::time::Duration::from_millis(30));
    let final_fg = unsafe { win::current_foreground_info() };
    tracing::info!(
        "Windows focus restore result: target=({}) restored={} final=({})",
        format_foreground_summary(Some(&target)),
        restored,
        format_foreground_summary(final_fg.as_ref())
    );
    final_fg
}

#[cfg(not(target_os = "windows"))]
pub fn ensure_recording_target_foreground_before_paste() -> Option<ForegroundWindowInfo> {
    None
}

#[cfg(target_os = "windows")]
mod win {
    use super::ForegroundWindowInfo;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::{CloseHandle, HWND};
    use windows_sys::Win32::System::Threading::{
        AttachThreadInput, GetCurrentThreadId, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        BringWindowToTop, GetForegroundWindow, GetWindow, GetWindowThreadProcessId, IsWindow,
        IsWindowVisible, SetForegroundWindow, GW_HWNDNEXT,
    };

    pub(super) unsafe fn is_valid_target_hwnd(hwnd: isize) -> bool {
        hwnd != 0 && IsWindow(hwnd as HWND) != 0
    }

    pub(super) unsafe fn current_foreground_info() -> Option<ForegroundWindowInfo> {
        let hwnd = GetForegroundWindow();
        inspect_hwnd(hwnd)
    }

    pub(super) unsafe fn find_non_overlay_foreground() -> Option<ForegroundWindowInfo> {
        let fg = GetForegroundWindow();
        if !fg.is_null() && !crate::overlay_window::is_own_overlay_hwnd(fg as isize) {
            return inspect_hwnd(fg);
        }
        // If one of our own overlays somehow holds foreground, walk behind it in Z-order.
        let mut candidate = fg;
        for _ in 0..64 {
            candidate = if candidate.is_null() {
                break;
            } else {
                GetWindow(candidate, GW_HWNDNEXT)
            };
            if candidate.is_null() {
                break;
            }
            if IsWindowVisible(candidate) == 0 {
                continue;
            }
            if crate::overlay_window::is_own_overlay_hwnd(candidate as isize) {
                continue;
            }
            if let Some(info) = inspect_hwnd(candidate) {
                return Some(info);
            }
        }
        None
    }

    pub(super) unsafe fn inspect_hwnd(hwnd: HWND) -> Option<ForegroundWindowInfo> {
        if hwnd.is_null() || IsWindow(hwnd) == 0 {
            return None;
        }
        let mut pid = 0u32;
        let thread_id = GetWindowThreadProcessId(hwnd, &mut pid);
        let exe_name = process_exe_name(pid).unwrap_or_else(|| "<unknown>".to_string());
        Some(ForegroundWindowInfo {
            hwnd: hwnd as isize,
            pid,
            thread_id,
            exe_name,
        })
    }

    unsafe fn process_exe_name(pid: u32) -> Option<String> {
        if pid == 0 {
            return None;
        }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return None;
        }
        let mut buf = [0u16; 260];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buf.as_mut_ptr(), &mut len);
        let _ = CloseHandle(handle);
        if ok == 0 || len == 0 {
            return None;
        }
        let full = OsString::from_wide(&buf[..len as usize])
            .to_string_lossy()
            .to_string();
        full.rsplit(['\\', '/']).next().map(str::to_string)
    }

    pub(super) unsafe fn restore_foreground_hwnd(target: &ForegroundWindowInfo) -> bool {
        let target_hwnd = target.hwnd as HWND;
        if target_hwnd.is_null() || IsWindow(target_hwnd) == 0 {
            return false;
        }
        if SetForegroundWindow(target_hwnd) != 0 && GetForegroundWindow() == target_hwnd {
            return true;
        }

        // Windows blocks `SetForegroundWindow` from background threads unless input queues
        // are temporarily attached to the current foreground / target threads.
        let cur_tid = GetCurrentThreadId();
        let fg_hwnd = GetForegroundWindow();
        let fg_tid = if !fg_hwnd.is_null() {
            GetWindowThreadProcessId(fg_hwnd, ptr::null_mut())
        } else {
            0
        };
        let target_tid = target.thread_id;

        let attached_cur = if target_tid != 0 && cur_tid != target_tid {
            AttachThreadInput(cur_tid, target_tid, 1) != 0
        } else {
            false
        };
        let attached_fg = if fg_tid != 0 && fg_tid != target_tid && fg_tid != cur_tid {
            AttachThreadInput(fg_tid, target_tid, 1) != 0
        } else {
            false
        };

        let ok_fg = SetForegroundWindow(target_hwnd) != 0;
        let _ = BringWindowToTop(target_hwnd);

        if attached_fg {
            let _ = AttachThreadInput(fg_tid, target_tid, 0);
        }
        if attached_cur {
            let _ = AttachThreadInput(cur_tid, target_tid, 0);
        }

        ok_fg || GetForegroundWindow() == target_hwnd
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(hwnd: isize, exe: &str) -> ForegroundWindowInfo {
        ForegroundWindowInfo {
            hwnd,
            pid: 42,
            thread_id: 100,
            exe_name: exe.to_string(),
        }
    }

    #[test]
    fn the_log_summary_never_names_the_program() {
        let summary = format_foreground_summary(Some(&info(0x1234, "notepad.exe")));
        assert_eq!(summary, "hwnd=0x1234, pid=42");
        assert_eq!(format_foreground_summary(None), "hwnd=0x0, pid=0");
    }

    #[test]
    fn focus_restore_skipped_when_current_matches_recorded_hwnd() {
        let recorded = info(0x1234, "notepad.exe");
        let current = info(0x1234, "notepad.exe");
        assert!(!should_attempt_focus_restore(
            Some(&recorded),
            Some(&current)
        ));
    }

    #[test]
    fn focus_restore_triggered_when_foreground_drifted_or_missing() {
        let recorded = info(0x1234, "notepad.exe");
        let overlay = info(0x9999, "oratype.exe");
        assert!(should_attempt_focus_restore(
            Some(&recorded),
            Some(&overlay)
        ));
        assert!(should_attempt_focus_restore(Some(&recorded), None));
    }

    #[test]
    fn focus_restore_skipped_when_recorded_is_missing_or_null() {
        let current = info(0x9999, "oratype.exe");
        assert!(!should_attempt_focus_restore(None, Some(&current)));
        assert!(!should_attempt_focus_restore(
            Some(&info(0, "")),
            Some(&current)
        ));
    }
}
