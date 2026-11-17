use async_trait::async_trait;

use crate::error::AppError;

use super::{InsertResult, InsertionStrategy, OutputMode, TextOutput};

#[cfg(target_os = "windows")]
const SENDINPUT_CHUNK_CHARS: usize = 16;
#[cfg(target_os = "windows")]
const SENDINPUT_CHUNK_DELAY_MS: u64 = 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowsSendInputNewlineMode {
    Enter,
    ShiftEnter,
    CrLf,
}

impl WindowsSendInputNewlineMode {
    pub fn from_config_value(value: &str) -> Self {
        match value {
            "shiftEnter" => Self::ShiftEnter,
            "crlf" => Self::CrLf,
            _ => Self::Enter,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeError {
    Partial {
        typed_chars: usize,
        source: Box<TypeError>,
    },
    SendInputFailed(String),
}

impl TypeError {
    fn typed_chars(&self) -> usize {
        match self {
            Self::Partial { typed_chars, .. } => *typed_chars,
            _ => 0,
        }
    }
}

impl std::fmt::Display for TypeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TypeError::Partial {
                typed_chars,
                source,
            } => {
                write!(f, "{source} after {typed_chars} chars were sent")
            }
            TypeError::SendInputFailed(message) => write!(f, "Windows SendInput failed: {message}"),
        }
    }
}

impl std::error::Error for TypeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WindowsSendInputOptions {
    pub newline_mode: WindowsSendInputNewlineMode,
}

impl Default for WindowsSendInputOptions {
    fn default() -> Self {
        Self {
            newline_mode: WindowsSendInputNewlineMode::Enter,
        }
    }
}

pub struct WindowsSendInputOutput {
    options: WindowsSendInputOptions,
}

impl WindowsSendInputOutput {
    pub fn new(options: WindowsSendInputOptions) -> Self {
        Self { options }
    }
}

#[async_trait]
impl TextOutput for WindowsSendInputOutput {
    async fn type_text(&self, text: &str) -> Result<InsertResult, AppError> {
        let text = text.to_string();
        let options = self.options;
        tokio::task::spawn_blocking(move || {
            let result = type_unicode_chunk_with_options(&text, options);
            let insert_result = map_sendinput_result(&text, result);
            if insert_result.status == super::InsertStatus::Failed {
                Err(AppError::Output(format!(
                    "Windows SendInput inserted 0/{} chars",
                    expected_sendinput_typed_chars(&text)
                )))
            } else {
                Ok(insert_result)
            }
        })
        .await
        .map_err(|e| AppError::Output(format!("Spawn blocking error: {}", e)))?
    }

    fn mode(&self) -> OutputMode {
        OutputMode::Keyboard
    }
}

pub fn expected_sendinput_typed_chars(text: &str) -> usize {
    text.chars().filter(|ch| *ch != '\r').count()
}

/// Validates that `SendInput` accepted every generated `INPUT` event.
pub fn validate_send_input_count(
    sent: u32,
    expected: usize,
    last_os_error: &str,
) -> Result<u32, TypeError> {
    if sent as usize == expected {
        Ok(sent)
    } else {
        Err(TypeError::SendInputFailed(format!(
            "SendInput injected {sent}/{expected} events ({last_os_error})"
        )))
    }
}

pub fn map_sendinput_result(text: &str, result: Result<usize, TypeError>) -> InsertResult {
    let expected = expected_sendinput_typed_chars(text);
    match result {
        Ok(typed_chars) if typed_chars == expected => {
            InsertResult::inserted(InsertionStrategy::WindowsSendInput, typed_chars)
        }
        Ok(typed_chars) if typed_chars > 0 => {
            InsertResult::partially_inserted(InsertionStrategy::WindowsSendInput, typed_chars)
        }
        Err(error) if error.typed_chars() > 0 => InsertResult::partially_inserted(
            InsertionStrategy::WindowsSendInput,
            error.typed_chars(),
        ),
        Ok(_) | Err(_) => InsertResult::failed(InsertionStrategy::WindowsSendInput),
    }
}

#[cfg(target_os = "windows")]
fn type_unicode_chunk_with_options(
    text: &str,
    options: WindowsSendInputOptions,
) -> Result<usize, TypeError> {
    if text.is_empty() {
        return Ok(0);
    }

    super::windows_modifier_guard::wait_for_modifier_release()
        .map_err(|error| TypeError::SendInputFailed(error.to_string()))?;

    let mut typed_chars = 0usize;
    let mut sent_in_chunk = 0usize;
    let mut chars = text.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\r' {
            continue;
        }

        let result = if ch == '\n' {
            send_newline(options.newline_mode)
        } else if ch == '\t' {
            press_vk(windows_sys::Win32::UI::Input::KeyboardAndMouse::VK_TAB)
        } else {
            send_unicode_char(ch)
        };

        if let Err(error) = result {
            return Err(partial_or_original(typed_chars, error));
        }

        typed_chars += 1;
        sent_in_chunk += 1;

        if sent_in_chunk >= SENDINPUT_CHUNK_CHARS && chars.peek().is_some() {
            std::thread::sleep(std::time::Duration::from_millis(SENDINPUT_CHUNK_DELAY_MS));
            sent_in_chunk = 0;
        }
    }

    Ok(typed_chars)
}

#[cfg(not(target_os = "windows"))]
fn type_unicode_chunk_with_options(
    _text: &str,
    _options: WindowsSendInputOptions,
) -> Result<usize, TypeError> {
    Err(TypeError::SendInputFailed(
        "Windows SendInput is only available on Windows".to_string(),
    ))
}

#[cfg(target_os = "windows")]
fn partial_or_original(typed_chars: usize, source: TypeError) -> TypeError {
    if typed_chars == 0 {
        source
    } else {
        TypeError::Partial {
            typed_chars,
            source: Box::new(source),
        }
    }
}

#[cfg(target_os = "windows")]
fn send_unicode_char(ch: char) -> Result<(), TypeError> {
    let mut buf = [0u16; 2];
    for unit in ch.encode_utf16(&mut buf) {
        send_utf16_unit(*unit, false)?;
        send_utf16_unit(*unit, true)?;
    }
    Ok(())
}

#[cfg(target_os = "windows")]
fn send_newline(mode: WindowsSendInputNewlineMode) -> Result<(), TypeError> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_RETURN, VK_SHIFT};

    match mode {
        WindowsSendInputNewlineMode::Enter => press_vk(VK_RETURN),
        WindowsSendInputNewlineMode::ShiftEnter => {
            send_vk(VK_SHIFT, false)?;
            press_vk(VK_RETURN)?;
            send_vk(VK_SHIFT, true)
        }
        WindowsSendInputNewlineMode::CrLf => {
            send_utf16_unit(0x000D, false)?;
            send_utf16_unit(0x000D, true)?;
            send_utf16_unit(0x000A, false)?;
            send_utf16_unit(0x000A, true)
        }
    }
}

#[cfg(target_os = "windows")]
fn press_vk(vk: u16) -> Result<(), TypeError> {
    send_vk(vk, false)?;
    send_vk(vk, true)
}

#[cfg(target_os = "windows")]
fn send_vk(vk: u16, key_up: bool) -> Result<(), TypeError> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        MAPVK_VK_TO_VSC,
    };

    let scan = unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16 };
    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: scan,
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    validate_send_input_count(sent, 1, &std::io::Error::last_os_error().to_string()).map(|_| ())
}

/// Sends an atomic key chord (modifiers down, primary key down/up, modifiers up in reverse)
/// via Win32 `SendInput`, shielded from the native keyboard hook by `OwnKeystroke`.
/// Returns the number of `INPUT` events injected (`modifiers.len() * 2 + 2`).
#[cfg(target_os = "windows")]
pub(crate) fn send_vk_chord(modifiers: &[u16], key: u16) -> Result<u32, TypeError> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        MapVirtualKeyW, SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP,
        MAPVK_VK_TO_VSC,
    };

    let key_input = |vk: u16, key_up: bool| INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: unsafe { MapVirtualKeyW(vk as u32, MAPVK_VK_TO_VSC) as u16 },
                dwFlags: if key_up { KEYEVENTF_KEYUP } else { 0 },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let _own = crate::native_hotkey::OwnKeystroke::begin();
    let mut events = Vec::with_capacity(modifiers.len() * 2 + 2);
    for &modifier in modifiers {
        events.push(key_input(modifier, false));
    }
    events.push(key_input(key, false));
    events.push(key_input(key, true));
    for &modifier in modifiers.iter().rev() {
        events.push(key_input(modifier, true));
    }

    let sent = unsafe {
        SendInput(
            events.len() as u32,
            events.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        )
    };
    validate_send_input_count(
        sent,
        events.len(),
        &std::io::Error::last_os_error().to_string(),
    )
}

#[cfg(target_os = "windows")]
fn send_utf16_unit(unit: u16, key_up: bool) -> Result<(), TypeError> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, KEYEVENTF_UNICODE,
    };

    let input = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: 0,
                wScan: unit,
                dwFlags: if key_up {
                    KEYEVENTF_UNICODE | KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_UNICODE
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    };

    let sent = unsafe { SendInput(1, &input, std::mem::size_of::<INPUT>() as i32) };
    if sent == 1 {
        Ok(())
    } else {
        Err(TypeError::SendInputFailed(
            std::io::Error::last_os_error().to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expected_typed_chars_skips_carriage_returns_and_counts_unicode_scalars() {
        assert_eq!(expected_sendinput_typed_chars("A\r\n\t😀"), 4);
    }

    #[test]
    fn validate_send_input_count_checks_exact_event_total() {
        assert_eq!(validate_send_input_count(4, 4, ""), Ok(4));
        let err = validate_send_input_count(0, 4, "access denied").unwrap_err();
        assert!(err.to_string().contains("0/4"));
    }

    #[test]
    fn maps_full_partial_and_zero_sendinput_results() {
        assert_eq!(
            map_sendinput_result("abcd", Ok(4)),
            InsertResult::inserted(InsertionStrategy::WindowsSendInput, 4)
        );
        assert_eq!(
            map_sendinput_result("abcd", Ok(2)),
            InsertResult::partially_inserted(InsertionStrategy::WindowsSendInput, 2)
        );
        assert_eq!(
            map_sendinput_result("abcd", Ok(0)),
            InsertResult::failed(InsertionStrategy::WindowsSendInput)
        );
        assert_eq!(
            map_sendinput_result("abcd", Err(TypeError::SendInputFailed("denied".into()))),
            InsertResult::failed(InsertionStrategy::WindowsSendInput)
        );
        assert_eq!(
            map_sendinput_result(
                "abcd",
                Err(TypeError::Partial {
                    typed_chars: 3,
                    source: Box::new(TypeError::SendInputFailed("denied".into())),
                })
            ),
            InsertResult::partially_inserted(InsertionStrategy::WindowsSendInput, 3)
        );
    }
}
