//! Plan `cursor-context`: the text before the cursor as polish context.
//!
//! With "Use text before the cursor" on (off by default), a dictation reads the last few hundred
//! characters before the caret in the field it goes into, through Accessibility, when the
//! recording starts. Polish gets them as a separate message before the transcript, so it can
//! continue what is there: no capital letter in the middle of a sentence, the same language and
//! style. The text is never pasted, stored or logged (only its length). Password fields are never
//! read.

/// At most this many UTF-16 units before the caret are read: a paragraph or two.
pub const MAX_UNITS: usize = 300;

/// Reads the text before the caret in the focused field, tidied, or `None`. Blocking: a few
/// Accessibility messages, each capped at 150 ms.
pub fn capture() -> Option<String> {
    let started = std::time::Instant::now();
    let raw = crate::output::focus::focused_text_before_cursor(MAX_UNITS);
    let text = raw.as_deref().and_then(tidy);
    tracing::info!(
        "Text before cursor: {} chars in {} ms",
        text.as_deref().map(|t| t.chars().count()).unwrap_or(0),
        started.elapsed().as_millis()
    );
    text
}

/// The text as polish gets it. A read that filled [`MAX_UNITS`] probably starts inside a word,
/// so that first piece is dropped; text that is only whitespace is nothing. Trailing spaces and
/// line breaks are kept: whether the dictation starts a new word or line depends on them.
pub fn tidy(raw: &str) -> Option<String> {
    let mut text = raw.trim_start_matches('\u{FFFD}');
    let cut = raw.encode_utf16().count() >= MAX_UNITS;
    if cut {
        if let Some(space) = text.find(char::is_whitespace).filter(|at| *at <= 40) {
            text = text[space..].trim_start();
        }
    }
    (!text.trim().is_empty()).then(|| text.to_string())
}

/// The user message polish gets before the transcript. It is not part of the system prompt, so
/// the built-in AI's read instructions stay valid (plan `warm-builtin-ai`).
pub fn polish_message(before: &str) -> String {
    format!(
        "<text_before_cursor>\n{before}\n</text_before_cursor>\nThis is what the field already \
         holds just before the cursor; the dictation is inserted right after it. Use it only so \
         the dictation continues it (capital letter, punctuation, language). Do not repeat, \
         change, continue on your own or answer it."
    )
}

/// The polished text as it should be inserted after `before`, whatever the model did with the
/// context: the field's text dropped when the answer repeats it, the first letter's case set by
/// how the field's text ends, and one space where the two meet. Measured: both a 4B built-in
/// and a server model sometimes copy the context in front, or start a lower-case sentence.
pub fn fit(before: &str, polished: &str) -> String {
    let text = without_repeated_context(before, polished);
    let text = with_sentence_case(before, &text);
    join_after(before, &text)
}

/// `polished` without a copy of the field's text at its start or its end: the whole text, its
/// last line or its last sentence, compared on letters and digits only (any case, spacing or
/// punctuation). A dictation into "Can you check why it failed?" came back as "It worked
/// yesterday. Can you check why it failed?" from the server model.
fn without_repeated_context(before: &str, polished: &str) -> String {
    let trimmed = before.trim();
    let last_line = trimmed.rsplit('\n').next().unwrap_or_default();
    let last_sentence = last_line
        .trim_end_matches(SENTENCE_ENDS)
        .rsplit(SENTENCE_ENDS)
        .next()
        .unwrap_or_default();
    let mut text = polished;
    for candidate in [trimmed, last_line, last_sentence] {
        if let Some(rest) = after_prefix(candidate, text) {
            text = rest;
            break;
        }
    }
    for candidate in [trimmed, last_line, last_sentence] {
        if let Some(rest) = before_suffix(candidate, text) {
            text = rest;
            break;
        }
    }
    text.to_string()
}

/// The letters and digits of `copy`, lower case, when it is long enough to call a copy: two
/// words, or four characters of a language without spaces ("Status: " then "Status update is
/// ready." is not a copy).
fn copy_key(copy: &str) -> Option<Vec<char>> {
    let wanted: Vec<char> = copy
        .chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect();
    let words = copy
        .split(|c: char| !c.is_alphanumeric())
        .filter(|word| !word.is_empty())
        .count();
    let unspaced = wanted.iter().any(|c| !c.is_ascii());
    (wanted.len() >= 4 && (words >= 2 || unspaced)).then_some(wanted)
}

/// What comes before `suffix` at the end of `text`, when `text` ends with it and something is
/// left. The text's own final punctuation stays; a comma or spaces before the copy go.
fn before_suffix<'a>(suffix: &str, text: &'a str) -> Option<&'a str> {
    let mut wanted = copy_key(suffix)?;
    wanted.reverse();
    let mut matched = 0;
    for (at, c) in text.char_indices().rev() {
        if !c.is_alphanumeric() {
            continue;
        }
        let lower: Vec<char> = c.to_lowercase().collect();
        for lower in lower.into_iter().rev() {
            if wanted.get(matched) != Some(&lower) {
                return None;
            }
            matched += 1;
        }
        if matched == wanted.len() {
            let rest =
                text[..at].trim_end_matches(|c: char| c.is_whitespace() || ",，;；、".contains(c));
            return (!rest.trim().is_empty()).then_some(rest);
        }
    }
    None
}

const SENTENCE_ENDS: &[char] = &['.', '!', '?', '。', '！', '？'];

/// What follows `prefix` at the start of `text`, when `text` starts with it (letters and digits
/// compared: at least two words, or four characters of a language without spaces) and something
/// is left.
fn after_prefix<'a>(prefix: &str, text: &'a str) -> Option<&'a str> {
    let wanted = copy_key(prefix)?;
    let mut matched = 0;
    for (at, c) in text.char_indices() {
        if !c.is_alphanumeric() {
            continue;
        }
        for lower in c.to_lowercase() {
            if wanted.get(matched) != Some(&lower) {
                return None;
            }
            matched += 1;
        }
        if matched == wanted.len() {
            let rest = text[at + c.len_utf8()..].trim_start_matches(|c: char| {
                c.is_whitespace() || ",，:：;；、.。!！?？".contains(c)
            });
            return (!rest.trim().is_empty()).then_some(rest);
        }
    }
    None
}

/// Small words that never start a name: a capital on one of them in the middle of the field's
/// sentence is the model's, not the user's.
const MID_SENTENCE_WORDS: &[&str] = &[
    "a",
    "an",
    "the",
    "and",
    "but",
    "or",
    "nor",
    "so",
    "yet",
    "then",
    "which",
    "who",
    "whose",
    "that",
    "because",
    "since",
    "although",
    "though",
    "if",
    "when",
    "while",
    "where",
    "as",
    "to",
    "for",
    "from",
    "of",
    "in",
    "on",
    "at",
    "by",
    "with",
    "without",
    "about",
    "after",
    "before",
    "until",
    "than",
    "plus",
    "including",
];

/// A capital first letter after a finished sentence or a line break; a lower-case small word
/// in the middle of the field's sentence (after a letter, digit, comma or semicolon). Letters of
/// other alphabets and other words are left as the model wrote them.
fn with_sentence_case(before: &str, text: &str) -> String {
    let tail = before.trim_end_matches([' ', '\t']);
    let Some(last) = tail.chars().last() else {
        return text.to_string();
    };
    let Some(first) = text.chars().next() else {
        return text.to_string();
    };
    let rest = &text[first.len_utf8()..];
    let new_sentence = last == '\n' || SENTENCE_ENDS.contains(&last);
    if new_sentence && first.is_ascii_lowercase() {
        return format!("{}{rest}", first.to_ascii_uppercase());
    }
    let mid_sentence = last.is_alphanumeric() || matches!(last, ',' | ';' | '，' | '；');
    let word: String = text
        .chars()
        .take_while(|c| c.is_ascii_alphabetic())
        .collect();
    let capitalised_small_word = first.is_ascii_uppercase()
        && word.chars().skip(1).all(|c| c.is_ascii_lowercase())
        && MID_SENTENCE_WORDS.contains(&word.to_ascii_lowercase().as_str());
    if mid_sentence && capitalised_small_word {
        return format!("{}{rest}", first.to_ascii_lowercase());
    }
    text.to_string()
}

/// The polished text as it should be inserted after `before`: a space in front when `before`
/// ends a word or sentence in a language written with spaces and the dictation starts with a
/// letter or digit, and no doubled space when both have one.
pub fn join_after(before: &str, polished: &str) -> String {
    let Some(last) = before.chars().last() else {
        return polished.to_string();
    };
    if last.is_whitespace() {
        return polished.trim_start_matches([' ', '\t']).to_string();
    }
    let first = polished.chars().next();
    let ends_word = last.is_ascii_alphanumeric() || ".,;:!?)\"'".contains(last);
    if ends_word && first.is_some_and(|c| c.is_ascii_alphanumeric()) {
        return format!(" {polished}");
    }
    polished.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tidy_drops_a_cut_first_word_and_empty_text() {
        assert_eq!(tidy("Hi team, ").as_deref(), Some("Hi team, "));
        assert_eq!(tidy("  \n "), None);
        assert_eq!(tidy("\u{FFFD}ok").as_deref(), Some("ok"));
        let long = format!("ontinued {}", "word ".repeat(70));
        let tidied = tidy(&long).unwrap();
        assert!(tidied.starts_with("word "), "{tidied}");
        // Chinese has no spaces to cut at: kept whole.
        let chinese = "字".repeat(MAX_UNITS);
        assert_eq!(tidy(&chinese).unwrap().chars().count(), MAX_UNITS);
    }

    #[test]
    fn joins_with_one_space_where_words_meet() {
        assert_eq!(
            join_after("I'll finish today,", "and then ship it."),
            " and then ship it."
        );
        assert_eq!(join_after("Done.", "Next we test."), " Next we test.");
        assert_eq!(join_after("Hi ", "there."), "there.");
        assert_eq!(join_after("Hi ", " there."), "there.");
        assert_eq!(join_after("Line one\n", "Line two"), "Line two");
        assert_eq!(join_after("我今日", "好忙。"), "好忙。");
        assert_eq!(join_after("(", "note"), "note");
        assert_eq!(join_after("", "Hello."), "Hello.");
    }

    #[test]
    fn a_copy_of_the_context_is_dropped() {
        assert_eq!(
            fit(
                "Meeting notes for Monday: ",
                "Meeting notes for Monday: we agreed."
            ),
            "we agreed."
        );
        assert_eq!(
            fit("Hi Sarah,\n\n", "Hi Sarah,\n\nThanks for the reply."),
            "Thanks for the reply."
        );
        assert_eq!(
            fit("Hi Ken，\n\n", "Hi Ken, 多謝你嘅報告。"),
            "多謝你嘅報告。"
        );
        assert_eq!(
            fit(
                "今日開會講咗好多嘢，",
                "今日開會講咗好多嘢，最重要係出新版。"
            ),
            "最重要係出新版。"
        );
        assert_eq!(
            fit(
                "Notes.\nI finished the report this morning, ",
                "I finished the report this morning, and then I sent it."
            ),
            "and then I sent it."
        );
        assert_eq!(
            fit(
                "Can you check why the deploy failed?",
                "It worked fine yesterday. Can you check why the deploy failed?"
            ),
            " It worked fine yesterday."
        );
        assert_eq!(
            fit("今日開會，", "最重要係出新版，今日開會。"),
            "最重要係出新版"
        );
        // Not a copy: kept, and an answer that is only the context is left to the guard.
        assert_eq!(fit("Hi ", "Hi there."), "Hi there.");
        assert_eq!(
            fit("Status: ", "Status update is ready."),
            "Status update is ready."
        );
        assert_eq!(fit("Meeting notes", "Meeting notes"), " Meeting notes");
    }

    #[test]
    fn the_first_letter_follows_the_field() {
        assert_eq!(fit("Can you check? ", "it worked."), "It worked.");
        assert_eq!(fit("Dear team,\n\n", "please reply."), "Please reply.");
        assert_eq!(
            fit("this morning, ", "And then I sent it."),
            "and then I sent it."
        );
        assert_eq!(fit("I talked to", "Sarah today."), " Sarah today.");
        assert_eq!(fit("this morning, ", "I sent it."), "I sent it.");
        assert_eq!(fit("Plan: ", "Ship it."), "Ship it.");
        assert_eq!(fit("我今日，", "好忙。"), "好忙。");
    }

    #[test]
    fn the_message_carries_the_text_and_the_rule() {
        let message = polish_message("Dear Anna,\n");
        assert!(message.starts_with("<text_before_cursor>\nDear Anna,\n\n</text_before_cursor>"));
        assert!(message.contains("Do not repeat"));
    }
}
