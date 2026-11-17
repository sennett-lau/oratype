# Windows port (Windows 10/11 x64)

Exploratory draft of the macOS-to-Windows 10/11 x64 port for OraType on branch `draft/windows-port`
while upstream `CLAUDE.md` keeps Windows as a non-goal for now. It audits every
`cfg(target_os = "macos")` and macOS-only runtime path across Rust, Tauri configuration, release
scripts, and the React frontend, lands cross-compilable Win32 implementations via `windows-sys`
where practical (`x86_64-pc-windows-gnu`), and marks remaining COM, GPU, and installer work with
non-panicking `// TODO(windows): ...` stubs.

Status: draft - 2026-11-15.

## Goals and non-goals

- Compile cleanly for `x86_64-pc-windows-gnu` without altering macOS or Linux behavior.
- Implement core dictation primitives in Rust using documented Win32 APIs via `windows-sys`:
  `WH_KEYBOARD_LL` native hotkey hook and `Escape` cancel, atomic `SendInput` paste and selection
  copy, `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` + `HWND_TOPMOST` floating pill/Ask overlays,
  microphone `ConsentStore` registry checks, and `GlobalMemoryStatusEx` / registry hardware reads.
- Replace complex COM, Core Audio, GPU, and packaging paths with explicit fallback implementations and
  searchable `// TODO(windows): ...` markers rather than panicking or silently failing.
- Keep `CLAUDE.md` and macOS default behavior untouched.
- Not a goal in this draft: shipping an MSVC/NSIS/MSI installer, obtaining an Authenticode EV/OV
  certificate, implementing full COM `IUIAutomation` or Core Audio COM vtables, or validating UI
  rendering on physical Windows 10/11 hardware.

## Key decisions

- **Default Windows global shortcuts remain `Ctrl+/` (hold), `Ctrl+.` (Ask), and `Ctrl+Shift+/`
  (Translate).** Reason: standard PC keyboards do not expose the hardware `Fn` key as a virtual
  key; keeping these defaults on `tauri-plugin-global-shortcut` works out of the box alongside the
  existing `storage::mod` migration rules.
- **Native single keys, swallowed keys (`End`), side-specific modifiers, and active-run `Escape`
  cancel use a dedicated `WH_KEYBOARD_LL` thread.** Reason: `RegisterHotKey` (`global-hotkey`)
  cannot intercept modifier-only triggers (`RightAlt`), swallows keys less predictably, and collides
  with existing bindings; returning `1` from `LowLevelKeyboardProc` drops swallowed presses such as
  `End`, while a dedicated `GetMessageW` loop avoids blocking Tauri's main message pump.
- **Non-activating floating overlays use `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW` and `HWND_TOPMOST`.**
  Reason: Windows does not use `NSPanel`, and preventing focus theft on the pill and Ask panel avoids
  breaking caret target focus in Chromium, Office, and Electron apps; `set_key` strips
  `WS_EX_NOACTIVATE` only while selecting/copying text in the Ask panel.
- **Dictation output uses clipboard + atomic `SendInput` (`Ctrl+V` / `Ctrl+Shift+V` /
  `Shift+Insert`) gated by `windows_modifier_guard` and `OwnKeystroke`.** Reason: Win32 `SendInput`
  unicode typing (`windows_sendinput.rs`) already exists, but atomic `Ctrl+V` paste matches macOS
  speed and formatting across rich-text editors while `GetClipboardSequenceNumber()` detects whether
  the target app changed the clipboard before timed restore.
- **Built-in speech (`whisper-rs`) starts on CPU on Windows (`use_gpu(false)`), with Vulkan/CUDA as
  follow-up options.** Reason: Metal is Apple-only; CPU builds cross-compile cleanly via `cmake` +
  MinGW-w64 today, while Vulkan (`vulkan` feature) supports Intel/AMD/NVIDIA broadly and CUDA
  (`cuda` feature) can accelerate NVIDIA cards once tested on Windows hardware.
- **Secrets use `keyring`'s `windows-native` backend (Windows Credential Manager) and data lives in
  standard Windows paths.** Reason: `Cargo.toml` already selects `windows-native`; Tauri resolves
  `app_data_dir()` to `%APPDATA%\dev.oratype.mac`, and logs write to
  `%LOCALAPPDATA%\OraType\Logs\oratype.log` so configuration and large model files stay separated
  under Roaming vs Local/Logs.
- **Permissions bypass Accessibility/Automation prompts and check Microphone `ConsentStore`
  first.** Reason: Windows grants standard desktop apps input injection to same-integrity windows
  without an Accessibility toggle, whereas Windows Settings → Privacy → Microphone (`ConsentStore`
  HKLM + HKCU `NonPackaged`) can block capture globally or per-app.

## Considered

- **Custom HID filter / kernel driver to synthesize a Windows `Fn` key.** Rejected: laptops trap `Fn`
  in Embedded Controller firmware, and shipping a WHQL-signed driver is incompatible with a lightweight
  MIT desktop app.
- **Using `Enigo` Unicode typing (`KEYEVENTF_UNICODE`) as primary output instead of clipboard +
  `SendInput` `Ctrl+V`.** Rejected: slow for long paragraphs, drops newlines/tabs in some apps, and
  triggers IME glitches on CJK Windows layouts. Keep `SendInput` `Ctrl+V` primary and chunked
  `windows_sendinput.rs` as keyboard fallback.
- **Polling `GetAsyncKeyState` on a timer instead of `WH_KEYBOARD_LL`.** Rejected: cannot swallow
  `End` before it moves the editor cursor, misses rapid taps, and burns CPU when idle.
- **Migrating legacy `Typelite` registry/folder state on Windows (`rename_migration.rs`).** Rejected:
  Typelite never had a Windows release before 1.2.0, so migration stays a no-op returning
  `Next::Continue`.

## Audit of macOS-specific paths

| File                                                                      | Feature                                                | macOS approach                                                                               | Windows approach                                                                                                                                                         | Draft status |
| ------------------------------------------------------------------------- | ------------------------------------------------------ | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------ |
| `src-tauri/build.rs`                                                      | Linker / target env                                    | Links `libclang_rt.osx.a` for Metal `@available`                                             | Gates `/MANIFESTDEPENDENCY` on `target_env == "msvc"`; exports `ORATYPE_TARGET_TRIPLE` for GNU & MSVC                                                                    | done         |
| `src-tauri/Cargo.toml`                                                    | Platform deps & GPU features                           | `apple-native` keyring, `objc2`, `block2`, `coreaudio-sys`, `whisper-rs/metal`, `libc`       | `windows-native` keyring, `windows-sys 0.59` Win32 features, CPU `whisper-rs` (`vulkan`/`cuda` next)                                                                     | done         |
| `src-tauri/src/platform.rs`                                               | OS ID, Registry & child console suppression            | `current_os() -> "macos"`, Unix stdio                                                        | `current_os() -> "windows"`, `read_reg_sz` (`RegGetValueW`), `configure_background_command` (`CREATE_NO_WINDOW`)                                                         | done         |
| `src-tauri/src/native_keys.rs`                                            | Native key table                                       | `CGEvent` keycode table (`Fn`, `End`, `F13`–`F20`, side modifiers)                           | `WINDOWS_KEYS` `VK_*` table (`End`–`F20`, side mods, arrows, ANSI OEMs; no `Fn`); non-US `MapVirtualKeyExW` pending                                                      | done         |
| `src-tauri/src/hotkey.rs`                                                 | Routing, defaults, Switch language & Translate stop    | `Fn` defaults (`Fn`, `Fn+Space`, `Fn+LeftShift`), native `SwitchLanguage` + `translate_stop` | Defaults `Ctrl+/` hold, `Ctrl+.`, `Ctrl+Shift+/`; `windows_native_chord` routes native keys; `SwitchLanguage` & `translate_stop` gated                                   | stub         |
| `src-tauri/src/native_hotkey.rs`                                          | Low-level hook, key swallow & Escape cancel            | `CGEventTap` (`kCGHIDEventTap`) + `CGEventSourceFlagsState`                                  | Dedicated `WH_KEYBOARD_LL` thread (`SetWindowsHookExW` + `GetMessageW`), swallows matched chords, `VK_ESCAPE` cancel, `OwnKeystroke`                                     | done         |
| `src-tauri/src/output/windows_sendinput.rs` & `windows_modifier_guard.rs` | Direct typing & modifier release wait                  | `CGEventKeyboardSetUnicodeString` + `modifiers::any_held`                                    | Chunked UTF-16 `SendInput`, `GetAsyncKeyState` modifier guard, atomic `send_vk_chord` under `OwnKeystroke`                                                               | done         |
| `src-tauri/src/output/clipboard.rs`                                       | Paste shortcut simulation                              | `osascript` `Cmd+V` under `OwnKeystroke`                                                     | Waits `windows_modifier_guard`, sends `Ctrl+V` / `Ctrl+Shift+V` / `Shift+Insert` via `send_vk_chord` (UIPI toast pending)                                                | done         |
| `src-tauri/src/output/pasteboard.rs`                                      | Clipboard sequence, snapshot & history                 | `NSPasteboard` changeCount, multi-UTI snapshot, transient type                               | `GetClipboardSequenceNumber()` wired; snapshot/restore is plain-text `arboard` (`CF_DIB`/`HTML Format` & `Win+V` suppression pending)                                    | stub         |
| `src-tauri/src/output/keyboard.rs`                                        | Secondary keyboard typing output                       | Main-thread `dispatch` timeout (`MACOS_TYPE_*`)                                              | `spawn_blocking` + `wait_for_modifier_release` + `enigo::Enigo` typing                                                                                                   | done         |
| `src-tauri/src/output/focus.rs`                                           | Focused field inspect, length, selection & pre-caret   | Accessibility (`AXFocusedUIElement`, `AXRole`, `AXSelectedTextRange`)                        | Returns `None` / `SelectedText::Unknown` until COM `IUIAutomation` (`ValuePattern`, `TextPattern2::GetCaretRange`)                                                       | stub         |
| `src-tauri/src/selection.rs`                                              | Selection highlight capture                            | `AXSelectedText` then `osascript` `Cmd+C` + late-copy watch                                  | Falls through `focus` to `wait_for_modifier_release` + `send_vk_chord(&[VK_LCONTROL], 'C')` (`IUIAutomationTextPattern` next)                                            | done         |
| `src-tauri/src/cursor_context.rs`                                         | Pre-caret polish context                               | Calls `focus::focused_text_before_cursor(300)` (AX)                                          | Depends on `focus::focused_text_before_cursor`; returns `None` until COM `IUIAutomationTextPattern2::GetCaretRange`                                                      | stub         |
| `src-tauri/src/overlay_window.rs`                                         | Non-focus-stealing pill & Ask panel                    | `NSPanel` KVO swap, `NonActivatingPanel`, `FullScreenAuxiliary`, `NSStatusWindowLevel`       | `WS_EX_NOACTIVATE \| WS_EX_TOOLWINDOW` + `HWND_TOPMOST`, atomic `SetWindowPos`, `set_key` toggle (mixed-DPI `MonitorFromPoint` & prior `HWND` restore next)              | done         |
| `src-tauri/src/app_detector/platform/mod.rs` & `windows.rs`               | Foreground app, guard & browser URL                    | `NSWorkspace.frontmostApplication` + `osascript` URL/Automation                              | `GetForegroundWindow` + `QueryFullProcessImageNameW` `.exe` + `SetForegroundWindow` restore; `browser_host` = `None` (UIA address bar & UWP child `HWND` unwrap pending) | stub         |
| `src-tauri/src/app_detector/profiles.rs` & `user_mappings.rs`             | App profiles & custom matchers                         | Bundle ID (`NativeBundleId`) + process aliases + web hosts                                   | `user_mappings` uses `NativeExecutable` (`.exe`); `profiles.rs` already maps 25+ Windows `.exe`s (add `windowsterminal.exe`, Edge profiles next)                         | done         |
| `src-tauri/src/commands/permissions.rs` & `pipeline.rs`                   | Microphone, Accessibility & Automation                 | `AVCaptureDevice`, `AXIsProcessTrustedWithOptions`, AppleEvents                              | Accessibility/Automation return `Granted` (UIPI admin caveat); Mic checks HKLM/HKCU `ConsentStore\microphone` + `ShellExecuteW("ms-settings:privacy-microphone")`        | done         |
| `src-tauri/src/audio/output_mute.rs`                                      | Mute playback while recording                          | CoreAudio `kAudioHardwarePropertyDefaultOutputDevice` + crash flag                           | Startup crash flag cleared; active mute is no-op `NoOutput` pending COM `IMMDeviceEnumerator` + `IAudioEndpointVolume`                                                   | stub         |
| `src-tauri/src/credentials.rs`                                            | API key credential storage                             | `keyring` `apple-native` (Keychain + Typelite migration)                                     | `keyring` `windows-native` (Windows Credential Manager) already enabled and wired                                                                                        | done         |
| `src-tauri/src/storage/mod.rs` & `lib.rs` (paths/defaults/UI)             | AppData, logs, hotkeys & overlay init                  | `~/Library/Application Support/...`, `~/Library/Logs/...`, `ActivationPolicy`, `Reopen`      | `%APPDATA%\dev.oratype.mac`, `%LOCALAPPDATA%\OraType\Logs\oratype.log`, `Ctrl+/` defaults, startup `apply_overlay_windows`                                               | done         |
| `src-tauri/src/rename_migration.rs`                                       | Pre-1.2.0 `Typelite` upgrade migration                 | Moves `dev.typelite.mac` App Support/WebKit/LaunchAgent + relaunch                           | Immediate `Next::Continue` (no Windows Typelite release existed)                                                                                                         | done         |
| `src-tauri/src/stt/builtin.rs` & `hands_free/wake.rs`                     | Built-in Whisper STT & wake detection                  | `whisper-rs` with Metal (`use_gpu(true)`) + `RunEvent::Exit` unload                          | Compiles & runs `whisper-rs` CPU (`use_gpu(false)`) via cmake (`vulkan`/`cuda` GPU flags & wake AEC check next)                                                          | done         |
| `src-tauri/src/stt/hardware.rs`                                           | Memory/CPU/GPU model offer probe                       | `sysctlbyname` (`hw.optional.arm64`, brand, `hw.memsize`)                                    | `GlobalMemoryStatusEx` + registry `ProcessorNameString` done; returns `ChipKind::Unknown` until DXGI/Vulkan/CUDA VRAM + Windows offer rules                              | stub         |
| `src-tauri/src/llm/builtin.rs` & `scripts/build-llama-server.sh`          | Built-in AI / Qwen3-ASR `llama-server`                 | Static Metal binary via `build-llama-server.sh`, `ps`/`kill` cleanup                         | `.exe` lookup, `127.0.0.1` loopback bind, `CREATE_NO_WINDOW`; `build-llama-server.sh` is macOS-only (Windows binary, Job Object & firewall verify next)                  | stub         |
| `src-tauri/src/search_server.rs`                                          | Built-in SearXNG (`uv` + Python venv)                  | `uv.tar.gz` via `tar`, `.venv/bin/python`, `ps`/`kill` cleanup                               | `.exe` / `.venv\Scripts\python.exe` paths, `pc-windows-msvc.zip` URL, `CREATE_NO_WINDOW`; `.zip` extract & Job Object cleanup pending                                    | stub         |
| `src-tauri/src/speed_stats.rs`                                            | Multilingual word counting                             | `CFStringTokenizer` (`kCFStringTokenizerUnitWord`)                                           | `count_words_fallback` (letter runs + per-CJK character) active; `Windows.Data.Text.WordsSegmenter` pending evaluation                                                   | done         |
| `src-tauri/src/tray.rs` & `lib.rs` (autostart)                            | System tray icon & Launch at login                     | Monochrome template PNG + LaunchAgent                                                        | Uses `app.default_window_icon()` + `tauri-plugin-autostart` (`HKCU\...\Run` Registry entry)                                                                              | done         |
| `src-tauri/src/updates.rs` & `scripts/updater-manifest.mjs`               | Self-updater & `latest.json`                           | `/Contents/MacOS/` writable probe, `.app.tar.gz` + `darwin-aarch64`                          | `self_updatable()` disabled on Windows; `updater-manifest.mjs` lacks `windows-x86_64` NSIS/MSI entry & install-dir check                                                 | todo         |
| `src-tauri/tauri.conf.json` & `.github/workflows/release.yml`             | Installer, signing, WebView2, window chrome & firewall | `.app` + `dmgbuild`, `Typelite Signing`, `sidebar` + `Overlay` titlebar                      | Has `icons/icon.ico`; needs NSIS/MSI targets, WebView2 Evergreen config, Authenticode signing, Windows titlebar/`mica`, Windows CI & firewall guide                      | todo         |
| `src/lib/keyLabels.ts` & `src/components/AskPanel/selection.ts`           | PC key labels & Ask panel copy/select-all              | `⌃ ⌥ ⇧ ⌘ ↩ ⌫ ⌦` & `⌘C`/`⌘A`                                                                  | `isWindowsPlatform()` / `{ windows: true }` maps to `Ctrl`/`Alt`/`Shift`/`Win`/`Enter`/`Backspace`/`Del`/`Esc` & `Ctrl+C`/`Ctrl+A`                                       | done         |
| `src/components/Settings/*`, `Onboarding/*` & `src/i18n/locales/*.json`   | Recorder UI, System Dock toggle & onboarding           | `NativeHotkeyRecorder` gated on `isMacPlatform()`, 3 macOS permissions, `Show in Dock`       | Needs Windows UI adaptation: hide `Show in Dock` & macOS Accessibility steps, enable `NativeHotkeyRecorder` on Windows, add UIPI elevation hint                          | todo         |

## Parts

| File                               | What it covers                                                                                     |
| ---------------------------------- | -------------------------------------------------------------------------------------------------- |
| [architecture.md](architecture.md) | Win32/WinRT/COM API designs for hotkeys, output, overlays, detection, audio, STT/AI, and installer |
| [roadmap.md](roadmap.md)           | Phased delivery plan, Windows-specific risks, and physical Windows verification items              |

## Open questions

- **GPU strategy for Windows Built-in Speech/AI:** Ship dual Vulkan + CPU `llama-server.exe` and
  `whisper-rs` builds for broad AMD/Intel/NVIDIA laptops, or add a separate CUDA variant for
  higher throughput on NVIDIA GPUs at the cost of larger downloads?
- **UIPI and `uiAccess="true"` signing:** Should OraType stay a standard non-elevated user binary
  (documenting that Task Manager / Admin terminals require elevated mode or manual paste toast), or
  target EV/OV Authenticode signing + Program Files installation for `uiAccess="true"`?
- **Windows window chrome (`main` window):** Keep standard Windows caption buttons
  (`decorations: true` without macOS `Overlay` transparency tricks) with WebView2 Acrylic/Mica
  detection by OS build, or build a custom title bar on Windows 10/11?
- **Cross-compile vs native CI:** Keep MSVC (`x86_64-pc-windows-msvc` on GitHub `windows-latest`)
  as the official Windows release toolchain while preserving `x86_64-pc-windows-gnu` for fast
  Linux compile checks?
