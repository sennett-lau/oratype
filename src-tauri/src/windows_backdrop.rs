//! Windows backdrop configuration for the main application window.
//!
//! `tauri.conf.json` configures `transparent: true` and macOS sidebar vibrancy so the macOS
//! build keeps its glass appearance untouched. Left alone on Windows, a transparent WebView2
//! window paired with translucent CSS leaves the main settings window permanently see-through
//! against the desktop. This module fixes ONLY the `"main"` window on Windows (`"capsule"` and
//! `"ask"` stay transparent overlays):
//!
//! - Windows 11 (`build >= 22000`): applies native DWM Mica (`DWMSBT_MAINWINDOW`) and clears any
//!   blur-behind / transparent cutout so the desktop wallpaper softly tints behind opaque text.
//! - Windows 10 (or if DWM Mica fails, e.g. Wine/RDP): resets DWM margins and forces an opaque
//!   solid window/WebView2 background color + notifies the frontend (`"solid"`).

use serde::Serialize;
use tauri::AppHandle;
#[cfg(target_os = "windows")]
use tauri::Manager;

#[allow(dead_code)]
pub const WIN11_MICA_MIN_BUILD: u32 = 22_000;
#[allow(dead_code)]
pub const WIN11_SYSTEMBACKDROP_MIN_BUILD: u32 = 22_523;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowsBackdropKind {
    Mica,
    SolidOpaque,
}

impl WindowsBackdropKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Mica => "mica",
            Self::SolidOpaque => "solid",
        }
    }
}

/// Pure policy helper: selects `"mica"` only when running on Windows 11 (`build >= 22000`) and
/// the DWM Mica call succeeded; otherwise falls back to `"solid"`.
pub fn choose_windows_backdrop(build_number: u32, mica_succeeded: bool) -> WindowsBackdropKind {
    if build_number >= WIN11_MICA_MIN_BUILD && mica_succeeded {
        WindowsBackdropKind::Mica
    } else {
        WindowsBackdropKind::SolidOpaque
    }
}

pub fn apply_to_main_window(app: &AppHandle, dark: bool) -> WindowsBackdropKind {
    #[cfg(target_os = "windows")]
    {
        let Some(window) = app.get_webview_window("main") else {
            return WindowsBackdropKind::SolidOpaque;
        };
        win::apply(&window, dark)
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = (app, dark);
        WindowsBackdropKind::SolidOpaque
    }
}

#[tauri::command]
pub fn sync_main_window_backdrop(app: AppHandle, dark: bool) -> String {
    apply_to_main_window(&app, dark).as_str().to_string()
}

#[cfg(target_os = "windows")]
mod win {
    use super::{
        choose_windows_backdrop, WindowsBackdropKind, WIN11_MICA_MIN_BUILD,
        WIN11_SYSTEMBACKDROP_MIN_BUILD,
    };
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use std::ptr;
    use tauri::window::Color;
    use tauri::WebviewWindow;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DwmEnableBlurBehindWindow, DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
        DWMSBT_MAINWINDOW, DWMSBT_NONE, DWMWA_SYSTEMBACKDROP_TYPE, DWMWA_USE_IMMERSIVE_DARK_MODE,
        DWM_BB_ENABLE, DWM_BLURBEHIND,
    };
    use windows_sys::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryA};
    use windows_sys::Win32::System::SystemInformation::OSVERSIONINFOW;
    use windows_sys::Win32::UI::Controls::MARGINS;

    const DWMWA_MICA_EFFECT: u32 = 1029;
    const S_OK: i32 = 0;

    fn windows_build_number() -> u32 {
        type RtlGetVersionFn = unsafe extern "system" fn(*mut OSVERSIONINFOW) -> i32;
        unsafe {
            let ntdll = LoadLibraryA(c"ntdll.dll".as_ptr().cast());
            if !ntdll.is_null() {
                if let Some(proc) = GetProcAddress(ntdll, c"RtlGetVersion".as_ptr().cast()) {
                    let rtl_get_version: RtlGetVersionFn = std::mem::transmute(proc);
                    let mut info: OSVERSIONINFOW = zeroed();
                    info.dwOSVersionInfoSize = size_of::<OSVERSIONINFOW>() as u32;
                    if rtl_get_version(&mut info) == 0 && info.dwBuildNumber > 0 {
                        return info.dwBuildNumber;
                    }
                }
            }
        }
        0
    }

    unsafe fn try_apply_mica(hwnd: HWND, build_number: u32) -> bool {
        if build_number < WIN11_MICA_MIN_BUILD {
            return false;
        }
        let full_frame = MARGINS {
            cxLeftWidth: -1,
            cxRightWidth: -1,
            cyTopHeight: -1,
            cyBottomHeight: -1,
        };
        if DwmExtendFrameIntoClientArea(hwnd, &full_frame) != S_OK {
            return false;
        }
        if build_number >= WIN11_SYSTEMBACKDROP_MIN_BUILD {
            let backdrop: i32 = DWMSBT_MAINWINDOW;
            return DwmSetWindowAttribute(
                hwnd,
                DWMWA_SYSTEMBACKDROP_TYPE as u32,
                (&backdrop as *const i32).cast::<c_void>(),
                size_of::<i32>() as u32,
            ) == S_OK;
        }
        let enable: i32 = 1;
        DwmSetWindowAttribute(
            hwnd,
            DWMWA_MICA_EFFECT,
            (&enable as *const i32).cast::<c_void>(),
            size_of::<i32>() as u32,
        ) == S_OK
    }

    pub(super) fn apply(window: &WebviewWindow, dark: bool) -> WindowsBackdropKind {
        let Ok(raw_hwnd) = window.hwnd() else {
            return apply_solid_fallback(window, ptr::null_mut(), dark);
        };
        let hwnd = raw_hwnd.0 as HWND;
        if hwnd.is_null() {
            return apply_solid_fallback(window, ptr::null_mut(), dark);
        }

        let build = windows_build_number();
        unsafe {
            // Clear any blur-behind region Tauri may have configured for `transparent: true` +
            // `windowEffects`, so it never turns the entire client area see-through.
            let bb = DWM_BLURBEHIND {
                dwFlags: DWM_BB_ENABLE,
                fEnable: 0,
                hRgnBlur: ptr::null_mut(),
                fTransitionOnMaximized: 0,
            };
            let _ = DwmEnableBlurBehindWindow(hwnd, &bb);

            let dark_flag: i32 = i32::from(dark);
            let _ = DwmSetWindowAttribute(
                hwnd,
                DWMWA_USE_IMMERSIVE_DARK_MODE as u32,
                (&dark_flag as *const i32).cast::<c_void>(),
                size_of::<i32>() as u32,
            );
        }

        let mica_ok = unsafe { try_apply_mica(hwnd, build) };
        let kind = choose_windows_backdrop(build, mica_ok);
        if kind == WindowsBackdropKind::Mica {
            tracing::debug!("Main window backdrop on Windows build {build}: Mica (dark={dark})");
            return kind;
        }

        apply_solid_fallback(window, hwnd, dark)
    }

    fn apply_solid_fallback(window: &WebviewWindow, hwnd: HWND, dark: bool) -> WindowsBackdropKind {
        if !hwnd.is_null() {
            unsafe {
                let no_margins = MARGINS {
                    cxLeftWidth: 0,
                    cxRightWidth: 0,
                    cyTopHeight: 0,
                    cyBottomHeight: 0,
                };
                let _ = DwmExtendFrameIntoClientArea(hwnd, &no_margins);
                let none: i32 = DWMSBT_NONE;
                let _ = DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_SYSTEMBACKDROP_TYPE as u32,
                    (&none as *const i32).cast::<c_void>(),
                    size_of::<i32>() as u32,
                );
            }
        }
        let color = if dark {
            Color(20, 26, 38, 255)
        } else {
            Color(246, 248, 252, 255)
        };
        let _ = window.set_background_color(Some(color));
        tracing::debug!("Main window backdrop on Windows: SolidOpaque fallback (dark={dark})");
        WindowsBackdropKind::SolidOpaque
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn windows_10_uses_solid_opaque_even_if_flag_set() {
        assert_eq!(
            choose_windows_backdrop(19_045, true),
            WindowsBackdropKind::SolidOpaque
        );
        assert_eq!(
            choose_windows_backdrop(19_045, false),
            WindowsBackdropKind::SolidOpaque
        );
    }

    #[test]
    fn windows_11_uses_mica_when_supported_and_solid_on_failure() {
        assert_eq!(
            choose_windows_backdrop(22_631, true),
            WindowsBackdropKind::Mica
        );
        assert_eq!(
            choose_windows_backdrop(22_000, true),
            WindowsBackdropKind::Mica
        );
        assert_eq!(
            choose_windows_backdrop(22_631, false),
            WindowsBackdropKind::SolidOpaque
        );
    }
}
