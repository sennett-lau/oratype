use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub os: String,
    pub session_type: String,
    pub global_hotkey_reliable: bool,
    pub keyboard_output_reliable: bool,
    pub clipboard_auto_paste_reliable: bool,
}

pub fn current_os() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(target_os = "windows")]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        "unknown"
    }
}

pub fn current_session_type() -> String {
    #[cfg(target_os = "linux")]
    {
        let session = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
        normalize_session_type(&session).to_string()
    }

    #[cfg(not(target_os = "linux"))]
    {
        "unknown".to_string()
    }
}

pub fn is_wayland_session() -> bool {
    current_session_type() == "wayland"
}

pub fn capabilities() -> PlatformCapabilities {
    capabilities_for(current_os(), &current_session_type())
}

#[cfg(target_os = "windows")]
pub(crate) fn read_reg_sz(
    root: windows_sys::Win32::System::Registry::HKEY,
    subkey: &str,
    value_name: &str,
) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr;
    use windows_sys::Win32::Foundation::ERROR_SUCCESS;
    use windows_sys::Win32::System::Registry::{RegGetValueW, RRF_RT_REG_SZ};

    let to_wide = |s: &str| {
        OsStr::new(s)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>()
    };
    let subkey_w = to_wide(subkey);
    let value_w = to_wide(value_name);
    let mut byte_len: u32 = 0;

    let status = unsafe {
        RegGetValueW(
            root,
            subkey_w.as_ptr(),
            value_w.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            ptr::null_mut(),
            &mut byte_len,
        )
    };
    if status != ERROR_SUCCESS || byte_len == 0 {
        return None;
    }

    let unit_len = (byte_len as usize).div_ceil(2);
    let mut buffer = vec![0u16; unit_len];
    let status = unsafe {
        RegGetValueW(
            root,
            subkey_w.as_ptr(),
            value_w.as_ptr(),
            RRF_RT_REG_SZ,
            ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut byte_len,
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let trimmed = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    let text = String::from_utf16_lossy(&buffer[..trimmed])
        .trim()
        .to_string();
    (!text.is_empty()).then_some(text)
}

/// Prevents spawned background CLI helpers (`llama-server`, `uv`, Python SearXNG) from
/// flashing an unwanted console window on Windows (`CREATE_NO_WINDOW`).
pub(crate) fn configure_background_command(command: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(target_os = "windows"))]
    let _ = command;
}

fn normalize_session_type(session_type: &str) -> &'static str {
    if session_type.eq_ignore_ascii_case("wayland") {
        "wayland"
    } else if session_type.eq_ignore_ascii_case("x11") {
        "x11"
    } else {
        "unknown"
    }
}

fn capabilities_for(os: &str, session_type: &str) -> PlatformCapabilities {
    let normalized_session = normalize_session_type(session_type);
    let is_linux_wayland = os == "linux" && normalized_session == "wayland";

    PlatformCapabilities {
        os: os.to_string(),
        session_type: normalized_session.to_string(),
        global_hotkey_reliable: !is_linux_wayland,
        keyboard_output_reliable: !is_linux_wayland,
        clipboard_auto_paste_reliable: !is_linux_wayland,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayland_linux_marks_input_automation_unreliable() {
        let caps = capabilities_for("linux", "wayland");

        assert_eq!(caps.session_type, "wayland");
        assert!(!caps.global_hotkey_reliable);
        assert!(!caps.keyboard_output_reliable);
        assert!(!caps.clipboard_auto_paste_reliable);
    }

    #[test]
    fn x11_linux_keeps_input_automation_reliable() {
        let caps = capabilities_for("linux", "x11");

        assert_eq!(caps.session_type, "x11");
        assert!(caps.global_hotkey_reliable);
        assert!(caps.keyboard_output_reliable);
        assert!(caps.clipboard_auto_paste_reliable);
    }

    #[test]
    fn unknown_session_normalizes_to_unknown() {
        let caps = capabilities_for("linux", "mir");

        assert_eq!(caps.session_type, "unknown");
        assert!(caps.global_hotkey_reliable);
    }
}
