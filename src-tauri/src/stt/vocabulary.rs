//! Plan `dictionary-to-speech`: the personal dictionary, given to the speech recognizer.
//!
//! Polish has always seen the dictionary, but by then a name the recognizer misheard ("Senate"
//! for "Sennett", "or type" for "OraType") is already wrong and polish can only guess. Each
//! engine can be told which words to expect:
//!
//! - an OpenAI-compatible transcription upload sends them as the `prompt` field. Whisper servers
//!   read it as earlier text; `llama-server` (Qwen3-ASR) uses it as the instruction before the
//!   audio, and Qwen3-ASR then spells those words as given;
//! - built-in whisper.cpp gets them as its initial prompt.
//!
//! The words go as a plain comma-separated list, capped so a long dictionary neither slows every
//! run nor pushes Whisper's prompt past what it reads (224 tokens).

/// At most this many words are sent.
pub const MAX_WORDS: usize = 40;
/// And at most this many characters in total.
pub const MAX_CHARS: usize = 400;
/// A dictionary entry longer than this is a phrase or a note, not a word to listen for.
const MAX_WORD_CHARS: usize = 40;

/// The words as the recognizer gets them, "OraType, Sennett, Tauri", or `None` when there is
/// nothing to send. Blank and over-long entries are skipped, repeats (any case) are sent once,
/// and the list stops at [`MAX_WORDS`] words or [`MAX_CHARS`] characters, in dictionary order.
pub fn speech_prompt(words: &[String]) -> Option<String> {
    let mut kept: Vec<String> = Vec::new();
    let mut chars = 0;
    for word in words {
        let word = word.split_whitespace().collect::<Vec<_>>().join(" ");
        let length = word.chars().count();
        if word.is_empty() || length > MAX_WORD_CHARS || word.chars().any(char::is_control) {
            continue;
        }
        let lower = word.to_lowercase();
        if kept.iter().any(|seen| seen.to_lowercase() == lower) {
            continue;
        }
        let added = if kept.is_empty() { length } else { length + 2 };
        if kept.len() == MAX_WORDS || chars + added > MAX_CHARS {
            break;
        }
        chars += added;
        kept.push(word);
    }
    (!kept.is_empty()).then(|| kept.join(", "))
}

/// True when a transcript is the word list read back rather than speech: two or more items,
/// split at commas, each one of the dictionary's words. Whisper can repeat its prompt when the
/// audio has little to say; a person rarely dictates only a list of their dictionary words.
pub fn is_prompt_echo(text: &str, words: &[String]) -> bool {
    let known: Vec<String> = words
        .iter()
        .map(|w| super::hallucination::normalise(w))
        .filter(|w| !w.is_empty())
        .collect();
    let items: Vec<String> = text
        .split([',', '，', '、'])
        .map(super::hallucination::normalise)
        .filter(|item| !item.is_empty())
        .collect();
    items.len() >= 2 && items.iter().all(|item| known.contains(item))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words(list: &[&str]) -> Vec<String> {
        list.iter().map(|w| w.to_string()).collect()
    }

    #[test]
    fn joins_the_words_with_commas() {
        assert_eq!(
            speech_prompt(&words(&["OraType", "Sennett", "鰂魚涌"])).as_deref(),
            Some("OraType, Sennett, 鰂魚涌")
        );
        assert_eq!(speech_prompt(&[]), None);
        assert_eq!(speech_prompt(&words(&["  ", ""])), None);
    }

    #[test]
    fn cleans_and_skips_entries() {
        let long = "a".repeat(MAX_WORD_CHARS + 1);
        assert_eq!(
            speech_prompt(&words(&[
                " Ora   Type ",
                "oratype",
                "ORA TYPE",
                &long,
                "a\u{7}b",
                "Tauri"
            ]))
            .as_deref(),
            Some("Ora Type, oratype, Tauri")
        );
    }

    #[test]
    fn stops_at_the_word_and_character_caps() {
        let many: Vec<String> = (0..100).map(|i| format!("w{i}")).collect();
        let prompt = speech_prompt(&many).unwrap();
        assert_eq!(prompt.split(", ").count(), MAX_WORDS);
        assert!(prompt.starts_with("w0, w1, "));

        let wide: Vec<String> = (0..30).map(|i| format!("{i:0>30}")).collect();
        let prompt = speech_prompt(&wide).unwrap();
        assert!(prompt.chars().count() <= MAX_CHARS);
        assert_eq!(prompt.split(", ").count(), 12);
    }

    #[test]
    fn a_list_of_dictionary_words_is_an_echo() {
        let dictionary = words(&["OraType", "Sennett", "Tauri"]);
        assert!(is_prompt_echo("OraType, Sennett, Tauri.", &dictionary));
        assert!(is_prompt_echo("Sennett，Tauri", &dictionary));
        assert!(!is_prompt_echo("OraType", &dictionary));
        assert!(!is_prompt_echo("Ask Sennett about OraType.", &dictionary));
        assert!(!is_prompt_echo("OraType, and the rest", &dictionary));
        assert!(!is_prompt_echo("OraType, Sennett", &[]));
    }
}
