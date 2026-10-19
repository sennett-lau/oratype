//! Plan `hands-free-mode` (routing.md): what a hands-free request does.
//!
//! A hands-free request is recorded like Ask anything, then routed by its first words. What it
//! does without such a word is the user's setting ([`HandsFreeAction`], default: type it).
//!
//! When it types by default:
//!
//! - "ask …" (Cantonese 問下 / Mandarin 问一下 …) asks the rest; "translate …" (翻譯 / 翻译)
//!   goes to Ask whole, which translates.
//! - "type …" (幫我打 / 输入 …) still types the rest, without the word. A bare 打 stays: it
//!   starts too many sentences (打風, 打電話).
//! - Everything else is typed as said, questions included ("Can we meet at six?").
//!
//! When it asks by default:
//!
//! - "type …" / "dictate …" / "write down …" (Cantonese 打 / 寫低, Mandarin 输入 / 写下 …):
//!   dictate the rest into the focused app, as the Dictate shortcut would.
//! - "write …" without a draft object ("write I'll be there at six"): dictate as well. "Write an
//!   email to …" / 写一封 … stays Ask's draft command.
//! - Anything else goes to Ask unchanged, so Ask's own intent detection still finds questions,
//!   translations ("translate … into French"), drafts, edits and searches.
//!
//! In both, "open …" / "switch to …" (打开 / 切換到 …) is a voice command, when that feature
//! exists. Until then the hook returns "not handled" and the request does the default.

use super::HandsFreeAction;

/// Where a hands-free request goes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandsFreeRoute {
    /// Dictate this text (the request without its "type" prefix).
    Dictate(String),
    /// A voice command ("open Safari"); the text after the verb.
    VoiceCommand(String),
    /// Ask anything with this question (the request, or the rest after "ask").
    Ask(String),
}

/// Latin prefixes that always mean "dictate the rest".
const DICTATE_PREFIXES_EN: &[&str] = &[
    "type out",
    "type in",
    "type",
    "dictate",
    "write down",
    "write out",
    "note down",
];
/// "write" means "dictate" unless it is followed by one of these (then it is Ask's draft
/// command: "write an email", "write me a reply", "write back to Tom").
const DRAFT_OBJECTS_EN: &[&str] = &[
    "a",
    "an",
    "the",
    "me",
    "us",
    "him",
    "her",
    "them",
    "back",
    "to",
    "some",
    "something",
    "another",
    "my",
    "our",
    "your",
];
/// Chinese prefixes that always mean "dictate the rest" (Cantonese and Mandarin, both scripts).
const DICTATE_PREFIXES_ZH: &[&str] = &[
    "幫我打",
    "帮我打",
    "打字",
    "輸入",
    "输入",
    "寫低",
    "写低",
    "寫下",
    "写下",
    "記低",
    "记低",
    "聽寫",
    "听写",
];
/// Chinese draft prefixes (Ask's grammar); a bare 写 / 寫 not followed by these dictates.
const DRAFT_PREFIXES_ZH: &[&str] = &[
    "写一封",
    "寫一封",
    "写一份",
    "寫一份",
    "写个",
    "寫個",
    "写封",
    "寫封",
    "帮我写",
    "幫我寫",
];
/// 打 after these is another word (打开 open, 打电话 call, 打算 plan ...), not "type".
const NOT_TYPE_AFTER_DA: &[char] = &[
    '开', '開', '电', '電', '算', '工', '车', '車', '扫', '掃', '球', '架', '包', '折', '印', '卡',
    '针', '針', '听', '聽', '招', '败', '敗', '破', '扮', '理',
];
/// Latin words that ask the rest when hands-free types by default.
const ASK_PREFIXES_EN: &[&str] = &["ask anything", "ask"];
/// The same in Chinese. Only the "ask a little" forms: a bare 問 / 问 starts many dictated
/// sentences ("問佢幾點到").
const ASK_PREFIXES_ZH: &[&str] = &["問一問", "问一问", "問一下", "问一下", "問下", "问下"];
/// Words that start a translation request, which Ask handles whole.
const TRANSLATE_PREFIXES_EN: &[&str] = &["translate"];
const TRANSLATE_PREFIXES_ZH: &[&str] = &["翻譯", "翻译"];
/// Latin prefixes of a voice command.
const COMMAND_PREFIXES_EN: &[&str] = &["switch to", "open", "launch", "go to", "bring up"];
const COMMAND_PREFIXES_ZH: &[&str] = &[
    "切换到",
    "切換到",
    "打开",
    "打開",
    "开启",
    "開啟",
    "转去",
    "轉去",
];

/// Routes a hands-free request; `action` is what it does without a routing word.
pub fn route(request: &str, action: HandsFreeAction) -> HandsFreeRoute {
    let text = trim_lead(request);
    if text.is_empty() {
        return fallback(request, action);
    }
    if let Some(rest) =
        after_any_en(text, COMMAND_PREFIXES_EN).or_else(|| after_any_zh(text, COMMAND_PREFIXES_ZH))
    {
        return HandsFreeRoute::VoiceCommand(rest);
    }
    match action {
        HandsFreeAction::Dictate => route_typing_first(text),
        HandsFreeAction::Ask => route_asking_first(text),
    }
}

/// What a request does when no word routes it (and when a voice command was not handled).
pub fn fallback(request: &str, action: HandsFreeAction) -> HandsFreeRoute {
    match action {
        HandsFreeAction::Dictate => HandsFreeRoute::Dictate(request.trim().to_string()),
        HandsFreeAction::Ask => HandsFreeRoute::Ask(request.to_string()),
    }
}

/// Hands-free types by default: "ask …" and "translate …" go to Ask, "type …" drops the word.
fn route_typing_first(text: &str) -> HandsFreeRoute {
    if let Some(rest) =
        after_any_en(text, ASK_PREFIXES_EN).or_else(|| after_any_zh(text, ASK_PREFIXES_ZH))
    {
        return HandsFreeRoute::Ask(rest);
    }
    if after_any_en(text, TRANSLATE_PREFIXES_EN)
        .or_else(|| after_any_zh(text, TRANSLATE_PREFIXES_ZH))
        .is_some()
    {
        return HandsFreeRoute::Ask(text.to_string());
    }
    if let Some(rest) =
        after_any_en(text, DICTATE_PREFIXES_EN).or_else(|| after_any_zh(text, DICTATE_PREFIXES_ZH))
    {
        return HandsFreeRoute::Dictate(rest);
    }
    HandsFreeRoute::Dictate(text.to_string())
}

/// Hands-free asks by default: "type …" (and "write …" without a draft object) dictates.
fn route_asking_first(text: &str) -> HandsFreeRoute {
    let ask = || HandsFreeRoute::Ask(text.to_string());
    if let Some(rest) =
        after_any_en(text, DICTATE_PREFIXES_EN).or_else(|| after_any_zh(text, DICTATE_PREFIXES_ZH))
    {
        return HandsFreeRoute::Dictate(rest);
    }
    if let Some(rest) = after_en(text, "write") {
        let first = rest
            .split_whitespace()
            .next()
            .unwrap_or("")
            .trim_matches(|c: char| !c.is_alphanumeric())
            .to_lowercase();
        if !DRAFT_OBJECTS_EN.contains(&first.as_str()) {
            return HandsFreeRoute::Dictate(rest);
        }
        return ask();
    }
    if DRAFT_PREFIXES_ZH.iter().any(|p| text.starts_with(p)) {
        return ask();
    }
    if let Some(rest) = after_any_zh(text, &["写", "寫"]) {
        return HandsFreeRoute::Dictate(rest);
    }
    if let Some(after) = text.strip_prefix('打') {
        if after
            .chars()
            .next()
            .is_some_and(|next| !NOT_TYPE_AFTER_DA.contains(&next))
        {
            if let Some(rest) = payload(after) {
                return HandsFreeRoute::Dictate(rest);
            }
        }
    }
    ask()
}

/// Plan `hands-free-mode`: the hook for spoken app commands ("open Safari", "switch to Mail").
/// Voice commands are a separate feature (branch `feature/voice-commands`); until it is on main
/// this returns false and the caller does the default instead ([`fallback`]). When that feature lands,
/// call its executor here and return true when it handled the command.
pub fn run_voice_command(_app: &tauri::AppHandle, _command: &str) -> bool {
    false
}

/// Drops leading punctuation and filler a transcript may start with ("…", ", type ...").
fn trim_lead(text: &str) -> &str {
    text.trim_start_matches(|c: char| !c.is_alphanumeric())
        .trim_end()
}

/// The non-empty payload after a prefix, without the punctuation that joins them.
fn payload(rest: &str) -> Option<String> {
    let rest = rest
        .trim_start_matches(|c: char| c.is_whitespace() || is_joiner(c))
        .trim();
    (!rest.is_empty()).then(|| rest.to_string())
}

fn is_joiner(c: char) -> bool {
    matches!(
        c,
        ',' | ':' | ';' | '-' | '，' | '：' | '；' | '、' | '—' | '.' | '。'
    )
}

/// The payload after a Latin prefix that ends at a word boundary, case-insensitively.
fn after_en(text: &str, prefix: &str) -> Option<String> {
    let head = text.get(..prefix.len())?;
    if !head.eq_ignore_ascii_case(prefix) {
        return None;
    }
    let rest = &text[prefix.len()..];
    if rest.chars().next().is_some_and(char::is_alphanumeric) {
        return None;
    }
    payload(rest)
}

fn after_any_en(text: &str, prefixes: &[&str]) -> Option<String> {
    prefixes.iter().find_map(|prefix| after_en(text, prefix))
}

fn after_any_zh(text: &str, prefixes: &[&str]) -> Option<String> {
    prefixes
        .iter()
        .find_map(|prefix| text.strip_prefix(prefix).and_then(payload))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictate(text: &str) -> HandsFreeRoute {
        HandsFreeRoute::Dictate(text.to_string())
    }

    fn ask(text: &str) -> HandsFreeRoute {
        HandsFreeRoute::Ask(text.to_string())
    }

    /// Routing when hands-free asks by default.
    fn asking(text: &str) -> HandsFreeRoute {
        route(text, HandsFreeAction::Ask)
    }

    /// Routing when hands-free types by default (the default).
    fn typing(text: &str) -> HandsFreeRoute {
        route(text, HandsFreeAction::Dictate)
    }

    #[test]
    fn typing_first_types_everything_but_ask_and_translate() {
        assert_eq!(
            typing("I'll be there at six."),
            dictate("I'll be there at six.")
        );
        // A dictated question is typed, not answered.
        assert_eq!(
            typing("Can we meet tomorrow?"),
            dictate("Can we meet tomorrow?")
        );
        assert_eq!(typing("我今晚唔返嚟食飯"), dictate("我今晚唔返嚟食飯"));
        assert_eq!(
            typing("Write an email to Tom"),
            dictate("Write an email to Tom")
        );
        assert_eq!(typing("Type I'll be late."), dictate("I'll be late."));
        assert_eq!(typing("幫我打，我遲啲到"), dictate("我遲啲到"));
        // A bare 打 starts too many sentences to drop when typing anyway.
        assert_eq!(typing("打風呀今日"), dictate("打風呀今日"));
        assert_eq!(
            typing("Ask what's the weather in Hong Kong?"),
            ask("what's the weather in Hong Kong?")
        );
        assert_eq!(
            typing("ask, how far is the moon"),
            ask("how far is the moon")
        );
        assert_eq!(typing("Ask anything: who won?"), ask("who won?"));
        assert_eq!(typing("問下聽日會唔會落雨"), ask("聽日會唔會落雨"));
        assert_eq!(typing("问一下明天几点开会"), ask("明天几点开会"));
        assert_eq!(
            typing("Translate good morning into French"),
            ask("Translate good morning into French")
        );
        assert_eq!(typing("翻譯早晨做日文"), ask("翻譯早晨做日文"));
        // "Asking" and a bare 問 are not the ask word.
        assert_eq!(
            typing("Asking price is fine."),
            dictate("Asking price is fine.")
        );
        assert_eq!(typing("問佢幾點到"), dictate("問佢幾點到"));
    }

    #[test]
    fn a_request_that_was_only_the_end_phrase_is_typed_empty() {
        assert_eq!(typing(""), dictate(""));
        assert_eq!(asking(""), ask(""));
    }

    #[test]
    fn type_prefixes_dictate_the_rest() {
        assert_eq!(asking("Type I'll be late."), dictate("I'll be late."));
        assert_eq!(asking("type: hello world"), dictate("hello world"));
        assert_eq!(
            asking("Type out, see you at six."),
            dictate("see you at six.")
        );
        assert_eq!(
            asking("Dictate thanks for the update"),
            dictate("thanks for the update")
        );
        assert_eq!(asking("Write down buy milk."), dictate("buy milk."));
        assert_eq!(
            asking("Write I'll call you back."),
            dictate("I'll call you back.")
        );
        assert_eq!(asking("...type ok"), dictate("ok"));
    }

    #[test]
    fn chinese_and_cantonese_prefixes_dictate() {
        assert_eq!(asking("打我今晚唔返嚟食飯"), dictate("我今晚唔返嚟食飯"));
        assert_eq!(asking("幫我打，收到"), dictate("收到"));
        assert_eq!(asking("寫低聽日開會"), dictate("聽日開會"));
        assert_eq!(asking("输入明天见"), dictate("明天见"));
        assert_eq!(asking("写：明天下午三点开会"), dictate("明天下午三点开会"));
        assert_eq!(asking("寫我遲啲到"), dictate("我遲啲到"));
    }

    #[test]
    fn drafts_questions_and_translations_stay_with_ask() {
        for text in [
            "Write an email to Tom about Friday",
            "write me a reply saying yes",
            "Write back to Anna",
            "What's the weather in Hong Kong?",
            "Translate good morning into French",
            "Typically, how long does it take?",
            "Typewriters are old.",
            "写一封邮件给老板",
            "幫我寫一封信",
            "打算几点出发？",
            "打电话给妈妈",
            "type",
        ] {
            assert_eq!(asking(text), ask(text), "{text:?}");
        }
    }

    #[test]
    fn open_and_switch_go_to_the_voice_command_hook() {
        for route_with in [asking, typing] {
            assert_eq!(
                route_with("Open Safari."),
                HandsFreeRoute::VoiceCommand("Safari.".to_string())
            );
            assert_eq!(
                route_with("switch to Mail"),
                HandsFreeRoute::VoiceCommand("Mail".to_string())
            );
            assert_eq!(
                route_with("打開 Safari"),
                HandsFreeRoute::VoiceCommand("Safari".to_string())
            );
            assert_eq!(
                route_with("切换到微信"),
                HandsFreeRoute::VoiceCommand("微信".to_string())
            );
        }
        // "Opening hours?" is not "open".
        assert_eq!(
            asking("Opening hours of the library?"),
            ask("Opening hours of the library?")
        );
        // An unhandled voice command does the default.
        assert_eq!(
            fallback("Open the door at six.", HandsFreeAction::Dictate),
            dictate("Open the door at six.")
        );
        assert_eq!(
            fallback("Open Safari.", HandsFreeAction::Ask),
            ask("Open Safari.")
        );
    }
}
