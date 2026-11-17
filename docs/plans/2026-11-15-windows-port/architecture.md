# Architecture

Detailed Win32, COM, and packaging designs behind the Windows 10/11 x64 port in
[index.md](index.md).

## Global hotkeys and `WH_KEYBOARD_LL`

- **Default shortcuts:** `storage::mod` already assigns Windows defaults (`Ctrl+/` hold for
  Dictate, `Ctrl+.` for Ask, `Ctrl+Shift+/` for Translate) so new installs register via
  `tauri-plugin-global-shortcut` without needing `Fn`.
- **Low-level hook (`native_hotkey.rs` + `native_keys.rs`):** On Windows, a dedicated thread calls
  `PeekMessageW` (to allocate a thread message queue), installs `SetWindowsHookExW(WH_KEYBOARD_LL,
low_level_keyboard_proc, GetModuleHandleW(null), 0)`, and runs `GetMessageW` until `Drop` posts
  `WM_QUIT` via `PostThreadMessageW`.
- **Swallowing keys (`End`, `F13`–`F20`):** Returning `1` from `LowLevelKeyboardProc` suppresses key
  propagation to the foreground window, preventing `End` from jumping to line end while preserving
  all non-matching keys via `CallNextHookEx`. Auto-repeat suppression uses a `BTreeSet<u16>` of
  currently held `VK_*` codes.
- **Cancel on `Escape`:** `platform_cancel_keycode()` maps `WIN_VK_ESCAPE` (`0x1B`) on Windows so
  pressing `Escape` during a recording cancels the run without stealing focus.
- **Remaining hook gaps:** European `AltGr` emits a synthetic `VK_LCONTROL` (`LLKHF_ALTDOWN` /
  extended scan code) immediately before `VK_RMENU` and must be filtered on ISO layouts; OEM keys
  currently assume US ANSI scan/VK mappings instead of `MapVirtualKeyExW`; `ShortcutBindingList.tsx`
  needs `isMacPlatform()` relaxed once Windows native capture UX is ready.

## Output, clipboard, and modifier guard

- **Existing foundation:** `output/windows_sendinput.rs` provides chunked UTF-16 `SendInput` typing
  (`WindowsSendInputOutput`), and `output/windows_modifier_guard.rs` polls `GetAsyncKeyState` for
  `VK_SHIFT`, `VK_CONTROL`, `VK_MENU`, `VK_LWIN`, and `VK_RWIN` (50 × 10 ms cycles) before typing.
- **Atomic paste (`output/clipboard.rs` + `send_vk_chord`):** Dictation paste waits for modifier
  release, acquires `OwnKeystroke::begin()`, and dispatches an atomic multi-event `SendInput` array
  for `Ctrl+V` (`VK_LCONTROL` + `'V'`), `Ctrl+Shift+V`, or `Shift+Insert`.
- **Sequence & snapshot (`output/pasteboard.rs`):** `change_count()` calls
  `GetClipboardSequenceNumber()` so delayed clipboard restore skips overwriting user copies. Full
  Windows fidelity still needs `OpenClipboard`/`EnumClipboardFormats` snapshot/restore of images
  (`CF_DIB`), files (`CF_HDROP`), and HTML (`HTML Format`), plus setting
  `ExcludeClipboardContentFromMonitorProcessing` and `CanIncludeInClipboardHistory = 0` so Win+V
  history is not polluted.

## Foreground app detection

- **Window & executable (`app_detector/platform/windows.rs`):** `collect()` reads
  `GetForegroundWindow`, `GetWindowTextW`, and `GetWindowThreadProcessId`, then opens the process
  with `PROCESS_QUERY_LIMITED_INFORMATION` (`OpenProcess` + `QueryFullProcessImageNameW`) to derive
  the lowercase `.exe` name for `native_identity` and `process_alias`, and restores focus with
  `ShowWindow(SW_RESTORE)` + `SetForegroundWindow`.
- **App profiles (`app_detector/profiles.rs` & `user_mappings.rs`):** Built-in profiles already
  list Windows `.exe`s (`outlook.exe`, `ms-teams.exe`, `slack.exe`, `wechat.exe`, `code.exe`,
  `cursor.exe`, `winword.exe`, `wps.exe`, `idea64.exe`, etc.); `user_mappings.rs` already stores
  Windows rules as `NativeExecutable`. Windows-native terminals (`windowsterminal.exe`, `pwsh.exe`)
  and UWP child `HWND` unwrapping (`ApplicationFrameHost.exe` via `EnumChildWindows`) remain to add.
- **Browser host URL:** Windows browsers (`chrome.exe`, `msedge.exe`, `brave.exe`, `arc.exe`) are
  flagged as `is_supported_browser`, but `browser_host` stays `None` until COM `IUIAutomation` reads
  the address-bar `Edit` control (`ValuePattern.CurrentValue`) and normalizes URLs without
  requiring an Apple-Events permission dialog.

## Pill and Ask overlay windows

- **Window styles (`overlay_window.rs`):** `make_overlay` runs on the window's GUI thread
  (`GetWindowThreadProcessId == GetCurrentThreadId`), adds `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`
  via `SetWindowLongPtrW(GWL_EXSTYLE)`, and pins z-order with `SetWindowPos(..., HWND_TOPMOST, ...,
SWP_FRAMECHANGED)` so overlays appear without taskbar buttons or stealing keyboard focus.
- **Frame updates and Ask key focus:** `set_frame` uses atomic `SetWindowPos` (`SWP_NOZORDER |
SWP_NOACTIVATE`). `set_key` strips `WS_EX_NOACTIVATE` and calls `SetForegroundWindow` when users
  select text in Ask, then restores `WS_EX_NOACTIVATE` when done; saving/restoring the prior
  foreground `HWND` and handling per-monitor DPI (`MonitorFromPoint` + `GetMonitorInfoW` `rcWork`)
  for mixed-scale displays remain in Phase 2.
- **Fullscreen limitation:** `HWND_TOPMOST` floats above borderless fullscreen windows and the
  Windows taskbar, whereas exclusive Direct3D/Vulkan fullscreen games own the swapchain and sit
  above topmost desktop overlays.

## Permissions, audio mute, and UI Automation

- **Permissions (`commands/permissions.rs`):** Windows has no Accessibility or Automation gate;
  UIPI is documented instead when the foreground target runs elevated. Microphone readiness reads
  `HKLM\SOFTWARE\...\ConsentStore\microphone` and `HKCU\...\ConsentStore\microphone\NonPackaged`
  (`RegGetValueW`) and launches `ms-settings:privacy-microphone` via `ShellExecuteW`, plus needs a
  real WASAPI `E_ACCESSDENIED` capture probe check on hardware.
- **Audio output mute (`audio/output_mute.rs`):** Crash recovery flag handling in `%APPDATA%`
  runs at startup; live mute currently uses `NoOutput` pending Windows Core Audio COM
  (`IMMDeviceEnumerator::GetDefaultAudioEndpoint(eRender, eMultimedia)` + `IAudioEndpointVolume`).
- **UI Automation (`output/focus.rs`, `selection.rs`, `cursor_context.rs`):** Selection currently
  relies on `Ctrl+C` clipboard fallback via `SendInput`; reading focus editability, text length,
  selected ranges, and 300 UTF-16 units before the caret directly needs COM
  `IUIAutomation::GetFocusedElement`, `IUIAutomationValuePattern`, and
  `IUIAutomationTextPattern2::GetCaretRange`.

## Local STT/AI, search, packaging, and UI

- **Credentials & storage:** `keyring` uses Windows Credential Manager (`windows-native`),
  config/SQLite/models use `%APPDATA%\dev.oratype.mac`, and logs write to
  `%LOCALAPPDATA%\OraType\Logs\oratype.log`.
- **Built-in STT & AI (`stt/builtin.rs`, `stt/hardware.rs`, `llm/builtin.rs`):** `whisper-rs`
  runs on CPU today while `stt/hardware.rs` reads `GlobalMemoryStatusEx` RAM and CPU brand string
  from the registry. Enabling Built-in AI on Windows requires shipping `llama-server.exe` (Vulkan
  and/or CUDA + CPU fallback), DXGI/VRAM hardware rules, `TerminateProcess` / Win32 Job Object
  lifecycle (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), and verifying `127.0.0.1` binds cleanly without
  Windows Defender Firewall prompts (following the firewall note in `CLAUDE.md` — non-loopback
  binds auto-create inbound block rules).
- **Tray, autostart, installer, updater, and UI:** `tray.rs` and `tauri-plugin-autostart` already
  support Windows icons and `HKCU\...\Run`. Production distribution requires NSIS/MSI bundle rules
  in `tauri.conf.json`, WebView2 Evergreen bootstrapper handling, Authenticode code signing,
  extending `scripts/updater-manifest.mjs` and `updates.rs` for `windows-x86_64`, adapting
  `Main`/`Settings` glass & `Show in Dock` UI, and leveraging the completed PC key labels
  (`src/lib/keyLabels.ts`) and `Ctrl+C`/`Ctrl+A` Ask shortcuts (`src/components/AskPanel/selection.ts`).
