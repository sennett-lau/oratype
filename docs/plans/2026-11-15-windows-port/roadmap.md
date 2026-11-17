# Rollout, risks and Windows verification

Phased rollout plan, Windows-specific engineering risks, and physical machine verification items
for [index.md](index.md).

## Phased rollout

### Phase 1 — Core Dictation and Overlay MVP (foundation landed in this draft)

- Keep `x86_64-pc-windows-gnu` and MSVC cross-compilation green (`build.rs`, `Cargo.toml`,
  `platform.rs`).
- Dictation, Ask, and Translate via `Ctrl+/` defaults (`GlobalShortcut`) plus `WH_KEYBOARD_LL`
  native key hook (`End`, side modifiers, `Escape` cancel).
- Non-focus-stealing `WS_EX_NOACTIVATE | HWND_TOPMOST` pill/Ask overlays, `GetForegroundWindow` +
  `.exe` app detection, `SendInput` `Ctrl+V` paste and `Ctrl+C` selection copy with
  `GetClipboardSequenceNumber()`, Windows Credential Manager secrets, `%APPDATA%`/`%LOCALAPPDATA%`
  paths, Microphone `ConsentStore` check, CPU `whisper-rs` STT, tray, and PC key/Ask UI labels.

### Phase 2 — Rich Windows Desktop Integration

- Enable `NativeHotkeyRecorder` in `ShortcutBindingList.tsx` on Windows and hide macOS-only
  `Show in Dock` / Accessibility onboarding copy.
- Implement COM `IUIAutomation` in `output/focus.rs`, `selection.rs`, `cursor_context.rs`, and
  `app_detector/platform/windows.rs` (focus editability, non-clipboard selection, `cursor_context`
  pre-caret text, Chromium/Edge URL address bar, UWP `ApplicationFrameHost.exe` unwrapping).
- Implement COM Core Audio (`IMMDeviceEnumerator` + `IAudioEndpointVolume`) output mute during
  recording in `audio/output_mute.rs`.
- Complete Win32 multi-format clipboard snapshot/restore (`CF_DIB`, `CF_HDROP`, `HTML Format`),
  `CanIncludeInClipboardHistory = 0` (`Win+V` exclusion), mixed-DPI `MonitorFromPoint`/`rcWork`
  overlay coordinate scaling, prior foreground `HWND` restoration on Ask `set_key`, `AltGr`
  synthetic `VK_LCONTROL` filtering, and `MapVirtualKeyExW` OEM key mapping.
- Add Windows `SwitchLanguage` + `translate_stop` chords in `hotkey.rs` and evaluate
  `Windows.Data.Text.WordsSegmenter` in `speed_stats.rs`.

### Phase 3 — Local GPU AI, Search, Installer, and Updater

- Build `llama-server.exe` (Vulkan / CUDA / CPU) for Windows in CI, add `whisper-rs` `vulkan`/
  `cuda` options, add DXGI/VRAM hardware detection in `stt/hardware.rs`, bind child servers to
  Win32 Job Objects (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`), and extract `uv` `.zip` archives in
  `search_server.rs`.
- Configure NSIS/MSI bundling in `tauri.conf.json`, Evergreen WebView2 installation strategy,
  Windows titlebar/Mica appearance, Authenticode code signing, `latest.json` `windows-x86_64` in
  `scripts/updater-manifest.mjs` and `updates.rs`, and a `windows-latest` CI workflow.

## Risks and unknowns

- **UIPI boundary on elevated apps:** A standard-privilege OraType process cannot receive
  `WH_KEYBOARD_LL` events or deliver `SendInput` keystrokes when an Administrator window (Task
  Manager, elevated PowerShell) has focus unless OraType is Authenticode-signed and installed in
  a trusted folder with `uiAccess="true"`. Without it, paste must detect failure and fall back to
  copying text with a manual-paste notice.
- **European `AltGr` and non-US keyboard layouts:** ISO keyboards generate synthetic `VK_LCONTROL`
  before `VK_RMENU`, and OEM virtual keys (`VK_OEM_2` etc.) vary by layout; wrong handling can
  either misfire `Ctrl+RightAlt` or mislabel keys.
- **Mixed-DPI multi-monitor coordinates and fullscreen apps:** Dragging or positioning the capsule
  across 100% + 150%/200% monitors can drift or clip without per-monitor `rcWork` math, and
  exclusive D3D/Vulkan fullscreen games will always render above desktop `HWND_TOPMOST` windows.
- **Local inference latency and thermal load on CPU-only laptops:** Built-in `whisper-rs` and
  hands-free wake checks run without Metal acceleration until Vulkan/CUDA backends ship and are
  benchmarked on low-power x64 ultrabooks.
- **Windows Defender Firewall and SmartScreen:** Binding `llama-server.exe` or Python SearXNG on
  anything other than `127.0.0.1` triggers first-run firewall block rules (`CLAUDE.md`); unsigned
  NSIS/MSI installers trigger Windows Defender SmartScreen warnings on clean machines.
- **WASAPI shared-mode mic locking and acoustic echo:** Keeping hands-free capture open alongside
  calls or communication apps must handle sample-rate changes and exclusive-mode conflicts cleanly.

## Items requiring a real Windows 10/11 machine to verify

- End-to-end MinGW vs MSVC build and launch under real WebView2 Runtime (including Windows 10
  machines needing Evergreen WebView2 installation).
- `WH_KEYBOARD_LL` latency, `End` key swallowing in VS Code / Edge / Word, `AltGr` typing on a
  European ISO layout, and `Escape` cancellation mid-dictation.
- Pill and Ask panel focus preservation (`WS_EX_NOACTIVATE`), text selection + `Ctrl+C` inside Ask
  panel (`set_key`), multi-monitor mixed-DPI (100% + 175%) placement over taskbar and borderless
  fullscreen windows.
- `SendInput` `Ctrl+V` and clipboard restore timing across Electron, UWP, Win32, WSL terminals, and
  UIPI behavior against an elevated Administrator console.
- Microphone privacy toggle in Windows 10/11 Settings → Privacy → Microphone (`ConsentStore` HKLM
  and HKCU `NonPackaged` registry values plus `ms-settings:privacy-microphone` URI activation).
- `GlobalMemoryStatusEx` + registry `ProcessorNameString` output, `whisper-rs` CPU/Vulkan
  transcription latency, loopback firewall prompt absence for `llama-server.exe` and SearXNG on
  `127.0.0.1`, tray icon rendering, `tauri-plugin-autostart` Registry startup, and NSIS upgrade +
  `tauri-plugin-updater` signature flow.
