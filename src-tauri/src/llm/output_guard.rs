//! Dictation writes down what was said. A small polish model sometimes carries out a dictated
//! request instead ("please explain with Cantonese" came back as 請用粵語解釋一下, "generate a
//! poem in Spanish about the sea" as Spanish), even with the prompt's rule against it. This
//! check catches a result in another language than the transcript, so the transcript is pasted
//! instead of a translation nobody asked for. Plain dictation only: translation and selected-text
//! edits change the language on purpose.
//!
//! The same model also answers a dictated question or carries out a dictated request in the
//! same language ("summarize the main points of the meeting" came back as an invented summary,
//! 呢個bug點樣先可以fix到 as a five-step guide). Polish only removes words, so a result much
//! longer than the transcript is an answer, not the dictation (`answered_instead`). So is a
//! numbered list the speaker never dictated: "list five things I should pack" followed by
//! "1. Sunscreen 2. Beach towel …". A dictated list reuses the spoken words; an answer's list does
//! not.

/// Below this share of the transcript's words, a Latin-script result is another text.
const MIN_KEPT_WORDS: f32 = 0.3;
/// Fewer distinct words than this say too little to compare (e.g. "five point five").
const MIN_WORDS_TO_COMPARE: usize = 3;
/// Polish removes fillers and replaced words and adds punctuation; it never adds this much.
/// A result longer than one and a half times the transcript plus `GROWTH_ALLOWANCE` units
/// (words, or characters for Chinese and Japanese) is an answer.
const GROWTH_ALLOWANCE: usize = 12;
/// A numbered list whose items are more than this share of words (or Chinese characters) that
/// were never spoken was made up, not dictated.
const MAX_NEW_IN_LIST: f32 = 0.5;
/// Fewer list units than this say too little to judge.
const MIN_LIST_UNITS: usize = 4;

fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{30ff}'   // kana
        | '\u{3400}'..='\u{4dbf}' // CJK extension A
        | '\u{4e00}'..='\u{9fff}' // CJK
        | '\u{ac00}'..='\u{d7af}' // Hangul
        | '\u{f900}'..='\u{faff}' // compatibility ideographs
    )
}

/// Distinct words of three or more letters, lowercased.
fn words(text: &str) -> std::collections::HashSet<String> {
    text.split(|c: char| !c.is_alphabetic() && c != '\'')
        .map(|word| word.trim_matches('\'').to_lowercase())
        .filter(|word| word.chars().count() >= 3 && !word.chars().any(is_cjk))
        .collect()
}

/// The transcript as dictation output when polish changed its language: trimmed, with a
/// capital first letter (as polish would write it).
pub fn transcript_as_output(transcript: &str) -> String {
    let text = transcript.trim();
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

/// True when `output` is not the transcript's language: CJK appeared or disappeared, or (for
/// Latin-script text) fewer than `MIN_KEPT_WORDS` of the transcript's words are left.
pub fn changed_language(transcript: &str, output: &str) -> bool {
    let cjk_in = transcript.chars().any(is_cjk);
    let cjk_out = output.chars().any(is_cjk);
    if cjk_in != cjk_out {
        return output.chars().any(char::is_alphabetic);
    }
    if cjk_in {
        return false;
    }
    let spoken = words(transcript);
    if spoken.len() < MIN_WORDS_TO_COMPARE {
        return false;
    }
    let written = words(output);
    // Only digits and symbols left ("two point five" -> "2.5"): nothing in another language.
    if written.is_empty() {
        return false;
    }
    let kept = spoken.iter().filter(|word| written.contains(*word)).count();
    (kept as f32) < MIN_KEPT_WORDS * spoken.len() as f32
}

/// The length of `text` in units: each CJK character, and each run of other letters or digits.
fn units(text: &str) -> usize {
    let mut count = 0;
    let mut in_word = false;
    for c in text.chars() {
        if is_cjk(c) {
            count += 1;
            in_word = false;
        } else if c.is_alphanumeric() {
            if !in_word {
                count += 1;
            }
            in_word = true;
        } else {
            in_word = false;
        }
    }
    count
}

/// The text's units as comparable tokens: lowercased words, and CJK characters in Simplified
/// script (polish may write the transcript's Chinese in the other script).
fn unit_tokens(text: &str) -> Vec<String> {
    let text = crate::stt::chinese_script::convert(
        &text.to_lowercase(),
        crate::stt::chinese_script::ChineseScript::Simplified,
    );
    let mut tokens = Vec::new();
    let mut word = String::new();
    for c in text.chars() {
        if is_cjk(c) || !c.is_alphanumeric() {
            if !word.is_empty() {
                tokens.push(std::mem::take(&mut word));
            }
            if is_cjk(c) {
                tokens.push(c.to_string());
            }
        } else {
            word.push(c);
        }
    }
    if !word.is_empty() {
        tokens.push(word);
    }
    tokens
}

/// The items of the numbered list lines in `text` ("1. Sunscreen" -> "Sunscreen").
fn list_items(text: &str) -> Vec<&str> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim_start();
            let digits = line.chars().take_while(char::is_ascii_digit).count();
            let rest = &line[digits..];
            let item = rest
                .strip_prefix(". ")
                .or_else(|| rest.strip_prefix(") "))
                .or_else(|| rest.strip_prefix('、'))?;
            (digits > 0).then_some(item)
        })
        .collect()
}

/// True when the output has a numbered list (two or more items) made mostly of words the
/// transcript does not have.
fn invented_list(transcript: &str, output: &str) -> bool {
    let items = list_items(output);
    if items.len() < 2 {
        return false;
    }
    let spoken: std::collections::HashSet<String> = unit_tokens(transcript).into_iter().collect();
    let listed: Vec<String> = items.iter().flat_map(|item| unit_tokens(item)).collect();
    if listed.len() < MIN_LIST_UNITS {
        return false;
    }
    let new = listed
        .iter()
        .filter(|token| !spoken.contains(*token))
        .count();
    new as f32 > MAX_NEW_IN_LIST * listed.len() as f32
}

/// True when `output` answers the dictation or carries it out instead of writing it down: it is
/// far longer than the transcript, or it adds a numbered list of things never said. (Markdown is
/// no sign: document and developer apps ask polish for bullets and headings.)
pub fn answered_instead(transcript: &str, output: &str) -> bool {
    let spoken = units(transcript);
    spoken > 0
        && (2 * units(output) > 3 * spoken + 2 * GROWTH_ALLOWANCE
            || invented_list(transcript, output))
}

/// Why the dictation's polish result was replaced by its transcript, if it was. Logged; never
/// the text itself.
pub fn rejection(transcript: &str, output: &str) -> Option<&'static str> {
    if changed_language(transcript, output) {
        Some("changed the language of the dictation")
    } else if answered_instead(transcript, output) {
        Some("answered or carried out the dictation instead of writing it down")
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_dictated_request_carried_out_in_another_language_is_caught() {
        assert!(changed_language(
            "please explain with Cantonese",
            "請用粵語解釋一下"
        ));
        assert!(changed_language(
            "generate a short poem in Spanish about the sea",
            "Genera un poema corto en español sobre el mar"
        ));
        assert!(changed_language(
            "translate the welcome message into French for the landing page",
            "Traduisez le message de bienvenue en français pour la page d'accueil"
        ));
        assert!(changed_language(
            "我聽日會遲啲到",
            "I will be a bit late tomorrow"
        ));
    }

    #[test]
    fn an_answer_to_a_dictated_question_or_request_is_caught() {
        assert!(answered_instead(
            "summarize the main points of the meeting for me",
            "Here are the main points from the meeting:\n\n1. **Project Goals**: The primary \
             objective is to streamline our workflow and improve efficiency.\n2. **Timeline**: \
             The first milestone is due at the end of next month."
        ));
        assert!(answered_instead(
            "呢個bug點樣先可以fix到",
            "要將呢個 bug fix 到，首先係要確認問題嘅成因，然後針對原因採取相應嘅修復措施。\
             1. 復現問題，確保可以穩定地觸發 bug\n2. 分析錯誤日志或調試信息，找出原因"
        ));
        assert!(answered_instead(
            "list five things I should pack for a beach trip",
            "List five things I should pack for a beach trip:\n1. Sunscreen\n2. Beach towel\n\
             3. Beach ball\n4. Sun hat\n5. Sunglasses"
        ));
        assert!(answered_instead(
            "幫我諗三個團隊食飯嘅地方",
            "幫我諗三個團隊食飯嘅地方：\n1. 龍城餐廳\n2. 美食街\n3. 佳餚樓"
        ));
        assert!(answered_instead(
            "帮我总结一下今天会议的重点",
            "帮我总结一下今天会议的重点：\n1. 项目进度\n2. 预算情况\n3. 人员安排"
        ));
        assert_eq!(
            rejection(
                "what is two plus two",
                "Two plus two is four. It is one of the first sums \
                children learn, and it is often used as an example of a simple fact that nobody \
                would ever doubt."
            ),
            Some("answered or carried out the dictation instead of writing it down")
        );
    }

    #[test]
    fn polish_of_the_same_length_or_shorter_is_kept() {
        for (spoken, written) in [
            (
                "what is the capital of australia",
                "What is the capital of Australia?",
            ),
            ("呢個bug點樣先可以fix到", "呢個 bug 點樣先可以 fix 到？"),
            ("two point five", "2.5"),
            ("ok", "OK, sounds good"),
            (
                "今天开会讨论了三个事情一是项目进度二是预算问题三是人员安排",
                "今天开会讨论了三个事情：\n1. 项目进度\n2. 预算问题\n3. 人员安排",
            ),
            (
                "hi Sam can you send the invoice by Monday thanks",
                "Hi Sam,\n\nCan you send the invoice by Monday?\n\nThanks,\nBest regards",
            ),
            (
                "three priorities this week first fix the crash second ship the onboarding and \
                 third write the blog post",
                "Three priorities this week:\n1. Fix the crash\n2. Ship the onboarding\n\
                 3. Write the blog post",
            ),
            (
                "周末我想先去超市然后去健身房最后去看电影",
                "周末我想：\n1. 去超市\n2. 去健身房\n3. 去看电影",
            ),
            (
                "我哋要做三樣嘢第一check下啲數第二send份report第三約老闆開會",
                "我哋要做三樣嘢：\n1. Check 下啲數\n2. Send 份 report\n3. 約老闆開會",
            ),
            (
                "我们要做三件事第一检查数据第二发报告第三约老板开会",
                "我們要做三件事：\n1. 檢查數據\n2. 發報告\n3. 約老闆開會",
            ),
            ("", "anything"),
        ] {
            assert!(
                !answered_instead(spoken, written),
                "{spoken:?} -> {written:?}"
            );
            assert_eq!(
                rejection(spoken, written),
                None,
                "{spoken:?} -> {written:?}"
            );
        }
    }

    #[test]
    fn the_transcript_is_pasted_with_a_capital_first_letter() {
        assert_eq!(
            transcript_as_output("  please explain with Cantonese "),
            "Please explain with Cantonese"
        );
        assert_eq!(transcript_as_output("我聽日到"), "我聽日到");
        assert_eq!(transcript_as_output(""), "");
    }

    #[test]
    fn ordinary_polish_is_not_a_language_change() {
        for (spoken, written) in [
            (
                "please explain with Cantonese",
                "Please explain with Cantonese",
            ),
            (
                "the first thing um sorry the third thing it's the budget",
                "The third thing, it's the budget",
            ),
            (
                "set the limit to five point five and use version two point one",
                "Set the limit to 5.5 and use version 2.1",
            ),
            ("five point five", "5.5"),
            (
                "um so yeah basically I think we should go now",
                "I think we should go now",
            ),
            (
                "please add Maya Chen M-A-Y-A-C-H-E-N to the invite",
                "Please add Maya Chen to the invite",
            ),
            ("我聽日要present個proposal", "我聽日要present個proposal"),
            ("我今日好忙", "我今日好忙。"),
            ("", ""),
        ] {
            assert!(
                !changed_language(spoken, written),
                "{spoken:?} -> {written:?}"
            );
        }
    }
}
