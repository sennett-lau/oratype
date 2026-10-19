//! Plan `hands-free-mode` (wake-check.md): Typelite's own fuzzy match of a short transcript
//! against the wake phrase ("Hey" + the user's wake name, default "Sam").
//!
//! The transcript comes from a small Whisper model, so the match tolerates what such a model
//! writes for the same sound: punctuation and case ("Hey, Sam!"), a different greeting spelling
//! ("Hay Sam", "Hi Sam", "嘿 Sam"), a joined word ("Heysam"), a vowel heard differently
//! ("Hey Sem") and, at higher sensitivity, one wrong letter. The phrase must come at the start
//! of what was heard (up to two filler words before it, "Oh, hey Sam"), so a conversation that
//! merely mentions "hey Sam" in the middle of a sentence does not wake Typelite.
//!
//! The same name ends a request: "Thanks, Sam" at the very end of what was heard (routing.md).

use serde::{Deserialize, Serialize};

/// How loosely the wake name is matched (Settings → Hands-free → Sensitivity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Sensitivity {
    /// Exact name after normalising; fewest false wakes.
    Low,
    /// A vowel may differ ("Sem" for "Sam").
    #[default]
    Normal,
    /// Also one wrong letter in a name of four or more letters, and the name alone without
    /// "Hey".
    High,
}

/// Greetings accepted before the name, after normalising. Whisper writes "Hey" in many ways
/// (measured on synthetic voices: "K. Sam", "He is Sam", "A Sam"), and Chinese speech gives a
/// character for it.
const GREETINGS: &[&str] = &[
    "hey", "hay", "hei", "hej", "he", "hes", "hi", "hai", "ay", "aye", "eh", "heh", "a", "k",
    "kay", "嘿", "嗨", "黑", "喂", "哎", "诶", "欸",
];
/// A word Whisper may put between "He" and the name ("He is Sam" for "Hey Sam").
const AFTER_HE: &[&str] = &["is", "s"];
/// Words a speaker may say before the greeting ("Oh, hey Sam").
const MAX_LEADING_WORDS: usize = 2;
/// Words before the name that end a request ("Thanks, Sam"), after normalising. Chinese is one
/// word per character: 多謝 / 谢谢 / 唔該 in both scripts.
const CLOSINGS: &[&[&str]] = &[
    &["thanks"],
    &["thank", "you"],
    &["thankyou"],
    &["thank"],
    &["thx"],
    &["多", "謝"],
    &["多", "谢"],
    &["謝", "謝"],
    &["谢", "谢"],
    &["唔", "該"],
    &["唔", "该"],
];
/// Chinese transliterations Whisper writes for the default name.
const SAM_ALIASES: &[&str] = &["山姆", "森姆", "三姆", "沙姆", "心", "森", "三"];

/// The wake phrase, prepared for matching.
#[derive(Debug, Clone, PartialEq)]
pub struct WakePhrase {
    /// Folded spellings of the name (the name itself and any known transliteration).
    names: Vec<String>,
    /// The name as the user typed it, for the prompt.
    display: String,
}

/// The result of a successful match. Safe to log: it holds a number, not text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WakeMatch {
    /// 1.0 for an exact name; lower for each edit.
    pub score: f32,
    /// The greeting was missing (only accepted at high sensitivity).
    pub without_greeting: bool,
}

impl WakePhrase {
    /// Builds the phrase from the wake name in Settings. A leading greeting the user typed
    /// ("Hey Sam") is dropped, so "Sam" and "Hey Sam" mean the same. Empty means "Sam".
    pub fn from_name(name: &str) -> Self {
        let tokens = tokens(name);
        let start = usize::from(tokens.len() > 1 && is_greeting(&tokens[0]));
        let mut folded = fold(&tokens[start.min(tokens.len())..].concat());
        if folded.is_empty() {
            folded = "sam".to_string();
        }
        let mut names = vec![folded.clone()];
        if folded == "sam" {
            names.extend(SAM_ALIASES.iter().map(|alias| alias.to_string()));
        }
        let display = name
            .split_whitespace()
            .skip(start)
            .collect::<Vec<_>>()
            .join(" ");
        Self {
            names,
            display: if display.is_empty() {
                "Sam".to_string()
            } else {
                display
            },
        }
    }

    /// The wake name as typed ("Sam").
    pub fn name(&self) -> &str {
        &self.display
    }

    /// "Hey Sam", the phrase as shown and used as the wake check's prompt.
    pub fn spoken(&self) -> String {
        format!("Hey {}", self.display)
    }

    /// Matches a transcript. `None` when it is not the wake phrase at this sensitivity.
    pub fn matches(&self, transcript: &str, sensitivity: Sensitivity) -> Option<WakeMatch> {
        let words = tokens(transcript);
        if words.is_empty() {
            return None;
        }
        let mut best: Option<WakeMatch> = None;
        fn consider(best: &mut Option<WakeMatch>, candidate: WakeMatch) {
            if best.is_none_or(|b| candidate.score > b.score) {
                *best = Some(candidate);
            }
        }
        let last_start = MAX_LEADING_WORDS.min(words.len() - 1);
        for start in 0..=last_start {
            let word = &words[start];
            // "Hey Sam": a greeting word, then the name.
            if is_greeting(word) {
                let mut after = start + 1;
                if word == "he"
                    && words
                        .get(after)
                        .is_some_and(|w| AFTER_HE.contains(&w.as_str()))
                {
                    after += 1;
                }
                if let Some(score) = self.name_score(&words[after.min(words.len())..], sensitivity)
                {
                    consider(
                        &mut best,
                        WakeMatch {
                            score,
                            without_greeting: false,
                        },
                    );
                }
            }
            // "Heysam" / "Hasam": greeting and name written as one word.
            for greeting in GREETINGS.iter().filter(|g| g.is_ascii()) {
                if let Some(rest) = word.strip_prefix(greeting).filter(|rest| !rest.is_empty()) {
                    let mut joined = vec![rest.to_string()];
                    joined.extend(words[start + 1..].iter().cloned());
                    if let Some(score) = self.name_score(&joined, sensitivity) {
                        consider(
                            &mut best,
                            WakeMatch {
                                score,
                                without_greeting: false,
                            },
                        );
                    }
                }
            }
        }
        // High sensitivity: the name alone, when it is all that was heard.
        if best.is_none() && sensitivity == Sensitivity::High {
            if let Some(score) = self.name_score(&words, sensitivity) {
                let all_used = self
                    .names
                    .iter()
                    .any(|name| fold(&words.concat()).chars().count() <= name.chars().count() + 1);
                if all_used {
                    consider(
                        &mut best,
                        WakeMatch {
                            score,
                            without_greeting: true,
                        },
                    );
                }
            }
        }
        best
    }

    /// Best score of the name against the first one to four words of `words`.
    fn name_score(&self, words: &[String], sensitivity: Sensitivity) -> Option<f32> {
        (1..=words.len().min(4))
            .filter_map(|take| self.exact_name_score(&words[..take], sensitivity))
            .max_by(f32::total_cmp)
    }

    /// Score of the name against exactly `words`, joined.
    fn exact_name_score(&self, words: &[String], sensitivity: Sensitivity) -> Option<f32> {
        let heard = fold(&words.concat());
        self.names
            .iter()
            .filter_map(|name| {
                let length = name.chars().count().max(1) as f32;
                let cost = weighted_edit_distance(name, &heard);
                (cost <= allowed_cost(length, sensitivity)).then(|| (1.0 - cost / length).max(0.0))
            })
            .max_by(f32::total_cmp)
    }

    /// The text was only a greeting ("Hey."). A quiet "Hey, … Sam" can reach the wake check
    /// as two stretches; the listener keeps such a greeting for the next one (wake-check.md).
    pub fn is_greeting_only(&self, transcript: &str) -> bool {
        let words = tokens(transcript);
        !words.is_empty() && words.len() <= 2 && words.iter().all(|word| is_greeting(word))
    }

    /// Where the end phrase ("Thanks, Sam", 多謝 Sam) starts, as a byte offset into
    /// `transcript`, when the transcript ends with it. Only the very end counts: "thanks Sam
    /// for the notes" goes on.
    pub fn end_phrase_start(&self, transcript: &str, sensitivity: Sensitivity) -> Option<usize> {
        let words = tokens_at(transcript);
        let plain: Vec<String> = words.iter().map(|(word, _)| word.clone()).collect();
        let count = plain.len();
        for name_words in 1..=count.min(4) {
            let before = count - name_words;
            if self
                .exact_name_score(&plain[before..], sensitivity)
                .is_none()
            {
                continue;
            }
            for closing in CLOSINGS {
                let Some(start) = before.checked_sub(closing.len()) else {
                    continue;
                };
                if plain[start..before]
                    .iter()
                    .zip(closing.iter())
                    .all(|(word, expected)| word == expected)
                {
                    return Some(words[start].1);
                }
            }
        }
        None
    }

    /// The transcript without a closing "Thanks, Sam" and the punctuation that joined it.
    pub fn strip_end_phrase(&self, transcript: &str, sensitivity: Sensitivity) -> String {
        match self.end_phrase_start(transcript, sensitivity) {
            Some(start) => transcript[..start]
                .trim_end_matches(|c: char| {
                    c.is_whitespace()
                        || matches!(c, ',' | '，' | '、' | ';' | '；' | ':' | '：' | '-' | '—')
                })
                .to_string(),
            None => transcript.to_string(),
        }
    }
}

/// The largest edit cost accepted for a name of `length` letters.
fn allowed_cost(length: f32, sensitivity: Sensitivity) -> f32 {
    match sensitivity {
        Sensitivity::Low => 0.0,
        Sensitivity::Normal => (length * 0.15).max(0.5),
        // Not more than one vowel for a short name: one wrong consonant turned "Hey man" and
        // "Hey Pam" into wakes in the measurements.
        Sensitivity::High => (length * 0.25).max(0.75),
    }
}

fn is_greeting(word: &str) -> bool {
    GREETINGS.contains(&word)
}

fn is_cjk(c: char) -> bool {
    matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xAC00..=0xD7AF | 0xF900..=0xFAFF)
}

/// Lower-case words; punctuation splits words; each CJK character is its own word.
fn tokens(text: &str) -> Vec<String> {
    tokens_at(text).into_iter().map(|(word, _)| word).collect()
}

/// [`tokens`], each with the byte offset in `text` where it starts.
fn tokens_at(text: &str) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    let mut current = String::new();
    let mut start = 0;
    for (at, original) in text.char_indices() {
        for c in original.to_lowercase() {
            if is_cjk(c) {
                if !current.is_empty() {
                    out.push((std::mem::take(&mut current), start));
                }
                out.push((c.to_string(), at));
            } else if c.is_alphanumeric() || c == '\'' {
                if c != '\'' {
                    if current.is_empty() {
                        start = at;
                    }
                    current.push(c);
                }
            } else if !current.is_empty() {
                out.push((std::mem::take(&mut current), start));
            }
        }
    }
    if !current.is_empty() {
        out.push((current, start));
    }
    out
}

/// Spelling-insensitive form of a Latin word: accents removed, a few sound-alike spellings
/// unified, doubled letters single. CJK text is left as it is.
fn fold(word: &str) -> String {
    let plain: String = word
        .chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 's',
            'ñ' => 'n',
            other => other,
        })
        .collect();
    let mut s = plain
        .replace("ph", "f")
        .replace("ck", "k")
        .replace('z', "s")
        .replace('q', "k");
    // A silent "p" before "s" at the start ("Psalm" heard for "Sam").
    if s.starts_with("ps") {
        s.remove(0);
    }
    // "Psalm" / "Salm": silent "l" before "m".
    s = s.replace("lm", "m");
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if !out.ends_with(c) || !c.is_ascii_alphabetic() {
            out.push(c);
        }
    }
    out
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

/// Levenshtein distance where swapping one vowel for another costs half.
fn weighted_edit_distance(a: &str, b: &str) -> f32 {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<f32> = (0..=b.len()).map(|j| j as f32).collect();
    let mut current = vec![0.0; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        current[0] = (i + 1) as f32;
        for (j, &cb) in b.iter().enumerate() {
            let substitution = if ca == cb {
                0.0
            } else if is_vowel(ca) && is_vowel(cb) {
                0.5
            } else {
                1.0
            };
            current[j + 1] = (previous[j] + substitution)
                .min(previous[j + 1] + 1.0)
                .min(current[j] + 1.0);
        }
        std::mem::swap(&mut previous, &mut current);
    }
    previous[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wakes(name: &str, heard: &str, sensitivity: Sensitivity) -> bool {
        WakePhrase::from_name(name)
            .matches(heard, sensitivity)
            .is_some()
    }

    #[test]
    fn default_phrase_matches_whisper_spellings() {
        for heard in [
            "Hey Sam",
            "Hey Sam.",
            "Hey, Sam!",
            "hey sam",
            "Hay Sam.",
            "Hi Sam",
            "Heysam",
            "Hey Sam, what's the weather like?",
            "Oh, hey Sam.",
            "嘿 Sam",
            "嘿,Sam。",
            "嘿山姆",
            "Hey Psalm.",
            "Hey Sem.",
            "HEY SAM",
            "K. Sam.",
            "He is Sam.",
        ] {
            assert!(wakes("Sam", heard, Sensitivity::Normal), "{heard}");
        }
    }

    #[test]
    fn other_speech_does_not_wake() {
        for heard in [
            "",
            "Thank you.",
            "Same here.",
            "Hey Pam.",
            "Hey, Siri.",
            "Hey there, how are you?",
            "Samuel said hello.",
            "I told Sam about it, hey Sam.",
            "Okay so the thing is hey Sam",
            "Sam",
            "Hey",
            "Hey Jim.",
            "Let's sample it.",
            "你好",
            "我哋去食饭",
        ] {
            assert!(!wakes("Sam", heard, Sensitivity::Normal), "{heard:?}");
        }
    }

    #[test]
    fn sensitivity_changes_how_loose_the_name_may_be() {
        assert!(wakes("Sam", "Hey Sam", Sensitivity::Low));
        assert!(!wakes("Sam", "Hey Sem", Sensitivity::Low));
        assert!(wakes("Sam", "Hey Sem", Sensitivity::Normal));
        assert!(!wakes("Sam", "Hey Pam", Sensitivity::Normal));
        assert!(!wakes("Sam", "Hey Pam", Sensitivity::High));
        assert!(!wakes("Sam", "Hey man, how are you?", Sensitivity::High));
        assert!(!wakes("Jarvis", "Hey Garvis", Sensitivity::Normal));
        assert!(wakes("Jarvis", "Hey Garvis", Sensitivity::High));
        // The name alone only at high sensitivity, and only when nothing else was said.
        assert!(!wakes("Sam", "Sam.", Sensitivity::Normal));
        assert!(wakes("Sam", "Sam.", Sensitivity::High));
        assert!(!wakes(
            "Sam",
            "Sam is coming over later.",
            Sensitivity::High
        ));
    }

    #[test]
    fn custom_names_including_multi_word_and_chinese() {
        assert!(wakes("Jarvis", "Hey Jarvis.", Sensitivity::Normal));
        assert!(wakes("Hey Jarvis", "hey, jarvis", Sensitivity::Normal));
        assert!(!wakes("Jarvis", "Hey Sam", Sensitivity::Normal));
        assert!(wakes("Mary Ann", "Hey Mary-Ann!", Sensitivity::Normal));
        assert!(wakes("Mary Ann", "Hey Maryanne", Sensitivity::High));
        assert!(wakes("小森", "嘿小森", Sensitivity::Normal));
        assert!(wakes("小森", "嘿,小森。", Sensitivity::Low));
        assert!(!wakes("小森", "嘿小明", Sensitivity::Normal));
    }

    #[test]
    fn score_is_one_for_an_exact_name_and_lower_for_a_near_one() {
        let phrase = WakePhrase::from_name("Sam");
        assert_eq!(
            phrase
                .matches("Hey Sam", Sensitivity::Normal)
                .unwrap()
                .score,
            1.0
        );
        let near = phrase.matches("Hey Sem", Sensitivity::Normal).unwrap();
        assert!(near.score < 1.0 && near.score > 0.5);
        assert!(!near.without_greeting);
    }

    #[test]
    fn spoken_form_and_empty_name() {
        assert_eq!(WakePhrase::from_name("Sam").spoken(), "Hey Sam");
        assert_eq!(WakePhrase::from_name("  ").spoken(), "Hey Sam");
        assert_eq!(WakePhrase::from_name("hey Jarvis").spoken(), "Hey Jarvis");
        assert!(wakes("", "Hey Sam", Sensitivity::Low));
    }

    #[test]
    fn the_end_phrase_ends_a_request_only_at_the_very_end() {
        let sam = WakePhrase::from_name("Sam");
        let normal = Sensitivity::Normal;
        for heard in [
            "Thanks, Sam.",
            " Thank you, Sam!",
            "I'll be there at six. Thanks Sam",
            "see you then, thank you Sam.",
            "多謝 Sam",
            "我今晚唔返嚟食飯，多謝Sam。",
            "谢谢，Sam",
            "Thanks, Sem.",
        ] {
            assert!(sam.end_phrase_start(heard, normal).is_some(), "{heard:?}");
        }
        for heard in [
            "Thanks Sam for the notes.",
            "Thank you.",
            "Sam.",
            "Hey Sam.",
            "thanks, man",
            "Thanks, Pam.",
            "",
        ] {
            assert!(sam.end_phrase_start(heard, normal).is_none(), "{heard:?}");
        }
        let peter = WakePhrase::from_name("Peter");
        assert!(peter
            .end_phrase_start("All done. Thanks, Peter.", normal)
            .is_some());
        assert!(peter
            .end_phrase_start("All done. Thanks, Sam.", normal)
            .is_none());
    }

    #[test]
    fn the_end_phrase_is_cut_from_the_request() {
        let sam = WakePhrase::from_name("Sam");
        let normal = Sensitivity::Normal;
        assert_eq!(
            sam.strip_end_phrase("I'll be there at six. Thanks, Sam.", normal),
            "I'll be there at six."
        );
        assert_eq!(
            sam.strip_end_phrase("I'll be there at six, thanks Sam", normal),
            "I'll be there at six"
        );
        assert_eq!(
            sam.strip_end_phrase("我今晚唔返嚟食飯，多謝 Sam。", normal),
            "我今晚唔返嚟食飯"
        );
        assert_eq!(sam.strip_end_phrase("Thanks, Sam.", normal), "");
        assert_eq!(
            sam.strip_end_phrase("Thanks Sam for the notes.", normal),
            "Thanks Sam for the notes."
        );
    }

    #[test]
    fn a_lone_greeting_is_kept_for_the_next_stretch() {
        let sam = WakePhrase::from_name("Sam");
        assert!(sam.is_greeting_only(" Hey."));
        assert!(sam.is_greeting_only("Hey?"));
        assert!(sam.is_greeting_only("嘿"));
        assert!(!sam.is_greeting_only("Hey there."));
        assert!(!sam.is_greeting_only("Sam."));
        assert!(!sam.is_greeting_only(""));
        // The listener matches the next stretch with the greeting in front.
        assert!(sam.matches("Hey. Sam.", Sensitivity::Normal).is_some());
    }

    #[test]
    fn edit_distance_weights_vowels_half() {
        assert_eq!(weighted_edit_distance("sam", "sam"), 0.0);
        assert_eq!(weighted_edit_distance("sam", "sem"), 0.5);
        assert_eq!(weighted_edit_distance("sam", "pam"), 1.0);
        assert_eq!(weighted_edit_distance("sam", "sams"), 1.0);
        assert_eq!(fold("psalm"), "sam");
        assert_eq!(fold("summ"), "sum");
    }
}
