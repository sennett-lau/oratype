//! The hallucination guard shared by every speech provider.
//!
//! Given a short burst of noise that got past the voice check, ASR models can invent a fixed
//! subtitle/prior phrase ("Thank you.", "Thanks for watching!", "谢谢观看", or for Qwen3-ASR in
//! Cantonese mode, "我哋嘅生活…") or fall into a repetitive decoding loop.
//!
//! Three targeted rules keep real speech untouched while blocking known non-speech priors:
//!
//! 1. **Exact phrase + short voiced audio (`voiced_ms < MAX_VOICED_MS`)**: ordinary phrases a
//!    person could also dictate ("Thank you.", "多謝收看", "我哋嘅生活") are dropped only when the
//!    voiced duration is below [`MAX_VOICED_MS`].
//! 2. **Cantonese non-speech prior prefix outside normal speaking rate (`2.5..=8.0 chars/s`)**:
//!    Qwen3-ASR emits `"我哋嘅生活…"` on steady ambient/instrumental noise even when RMS windows stay
//!    above the threshold (`voiced_ms >= MAX_VOICED_MS`), either too sparsely (`< 2.5 chars/s` over
//!    the full recording) or as a dense loop (`> 8.0 chars/s`). Genuine spoken sentences starting
//!    with that phrase fall inside normal Cantonese articulation rates and are preserved.
//! 3. **Extreme repetition loop (`len >= 60`, repeat >= 5x & >= 50% coverage, `> 12 tokens/s`)**:
//!    catches degenerate repetitive decoding on background noise without touching real speech.

/// Transcripts of recordings with less voiced audio than this are checked for exact phrases.
pub const MAX_VOICED_MS: u32 = 1500;

/// Minimum plausible Cantonese speaking rate (normalised chars / total audio sec) for a
/// known Qwen3-ASR non-speech prior prefix.
pub const PRIOR_PREFIX_MIN_CHARS_PER_SEC: f64 = 2.5;

/// Maximum plausible Cantonese speaking rate (normalised chars / total audio sec) for a
/// known Qwen3-ASR non-speech prior prefix.
pub const PRIOR_PREFIX_MAX_CHARS_PER_SEC: f64 = 8.0;

/// Minimum normalised character count before repetition-loop detection runs.
pub const LOOP_MIN_CLEAN_CHARS: usize = 60;

/// Minimum consecutive repetitions of a 2..=32 char unit required for loop detection.
pub const LOOP_MIN_REPEATS: usize = 5;

/// Minimum fraction of normalised characters covered by the dominant repeat unit.
pub const LOOP_MIN_COVERAGE: f64 = 0.50;

/// Maximum MER tokens per total audio second before a repeating transcript is dropped.
pub const LOOP_MAX_TOKENS_PER_SEC: f64 = 12.0;

/// Known hallucinations, in normalised form (see [`normalise`]): lower case, letters and
/// digits only.
const KNOWN_HALLUCINATIONS: &[&str] = &[
    // English
    "thankyou",
    "thankyouverymuch",
    "thanks",
    "thanksforwatching",
    "thankyouforwatching",
    "thankyousomuchforwatching",
    "you",
    "bye",
    "byebye",
    // Chinese (simplified and traditional)
    "谢谢",
    "謝謝",
    "谢谢观看",
    "謝謝觀看",
    "谢谢大家",
    "字幕由amaraorg社区提供",
    "请不吝点赞订阅转发打赏支持明镜与点点栏目",
    "請不吝點贊訂閱轉發打賞支持明鏡與點點欄目",
    // Hong Kong Cantonese (Qwen3-ASR non-speech priors)
    "我哋嘅生活",
    "我哋嘅生活就係咁樣嘅",
    "我哋嘅生活就係咁樣樣",
    "我哋嘅生活係咁樣嘅",
    "我哋嘅生活係咁嘅",
    "我哋嘅生活環境",
    "多謝收看",
    "多謝觀看",
    "多謝大家收看",
    "謝謝收看",
    "谢谢收看",
    // Japanese
    "ご視聴ありがとうございました",
];

/// Longer Qwen3-ASR non-speech prior stems (ambient/instrumental audio) checked against total
/// speaking-rate bounds even when continuous background noise pushes `voiced_ms >= MAX_VOICED_MS`.
const KNOWN_CANTONESE_PRIOR_PREFIXES: &[&str] =
    &["我哋嘅生活", "我地的生話", "我们的生活", "我們的生活"];

/// Lower case with everything but letters and digits removed, so "Thank you." and
/// " thank  you!" compare equal.
pub fn normalise(text: &str) -> String {
    text.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

/// Counts Mixed Error Rate (MER) tokens: 1 per ASCII alphanumeric word (with internal `'`),
/// 1 per non-ASCII alphanumeric character (CJK, kana, etc.).
fn count_mer_tokens(text: &str) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut tokens = 0usize;
    let mut i = 0usize;
    while i < chars.len() {
        let ch = chars[i];
        if ch.is_ascii_alphanumeric() {
            tokens += 1;
            i += 1;
            while i < chars.len() {
                let c = chars[i];
                if c.is_ascii_alphanumeric()
                    || (c == '\'' && i + 1 < chars.len() && chars[i + 1].is_ascii_alphanumeric())
                {
                    i += 1;
                } else {
                    break;
                }
            }
        } else if ch.is_alphanumeric() {
            tokens += 1;
            i += 1;
        } else {
            i += 1;
        }
    }
    tokens
}

/// Finds `(repeats, coverage_ratio)` for the dominant consecutive repeat of a `2..=32` char unit.
fn max_repeat_coverage(chars: &[char]) -> (usize, f64) {
    let n = chars.len();
    if n < 6 {
        return (0, 0.0);
    }
    let mut best_cov = 0usize;
    let mut best_rep = 0usize;
    let max_u = 32.min(n / 3);
    for ulen in 2..=max_u {
        let mut i = 0usize;
        while i + ulen * 3 <= n {
            let unit = &chars[i..i + ulen];
            let mut k = 1usize;
            let mut j = i + ulen;
            while j + ulen <= n && &chars[j..j + ulen] == unit {
                k += 1;
                j += ulen;
            }
            if k >= 3 {
                let cov = k * ulen;
                if cov > best_cov {
                    best_cov = cov;
                    best_rep = k;
                }
                i = j;
            } else {
                i += 1;
            }
        }
    }
    (best_rep, best_cov as f64 / n as f64)
}

/// True when `text` is probably invented: the voiced audio was short and the whole transcript
/// is a known hallucination.
pub fn is_likely_hallucination(text: &str, voiced_ms: u32) -> bool {
    voiced_ms < MAX_VOICED_MS && KNOWN_HALLUCINATIONS.contains(&normalise(text).as_str())
}

/// Duration-aware hallucination guard combining short-voiced exact matches with Cantonese
/// prior-prefix rate bounds and extreme token-rate repetition loop filtering.
pub fn is_likely_hallucination_with_duration(text: &str, voiced_ms: u32, duration_ms: u32) -> bool {
    let norm = normalise(text);
    if norm.is_empty() {
        return false;
    }

    if voiced_ms < MAX_VOICED_MS && KNOWN_HALLUCINATIONS.contains(&norm.as_str()) {
        return true;
    }

    if duration_ms == 0 {
        return false;
    }
    let total_sec = f64::from(duration_ms) / 1000.0;
    let chars: Vec<char> = norm.chars().collect();
    let chars_per_sec = (chars.len() as f64) / total_sec;

    if KNOWN_CANTONESE_PRIOR_PREFIXES
        .iter()
        .any(|prefix| norm.starts_with(prefix))
        && !(PRIOR_PREFIX_MIN_CHARS_PER_SEC..=PRIOR_PREFIX_MAX_CHARS_PER_SEC)
            .contains(&chars_per_sec)
    {
        return true;
    }

    if chars.len() >= LOOP_MIN_CLEAN_CHARS {
        let (reps, cov) = max_repeat_coverage(&chars);
        let tokens_per_sec = (count_mer_tokens(text) as f64) / total_sec;
        if reps >= LOOP_MIN_REPEATS
            && cov >= LOOP_MIN_COVERAGE
            && tokens_per_sec > LOOP_MAX_TOKENS_PER_SEC
        {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_is_stored_normalised() {
        for phrase in KNOWN_HALLUCINATIONS {
            assert_eq!(&normalise(phrase), phrase);
        }
        for prefix in KNOWN_CANTONESE_PRIOR_PREFIXES {
            assert_eq!(&normalise(prefix), prefix);
        }
    }

    #[test]
    fn known_phrases_from_short_audio_are_dropped() {
        for text in [
            "Thank you.",
            " thank you!",
            "Thanks for watching!",
            "Thank you for watching.",
            "you",
            "Bye.",
            "谢谢",
            "謝謝。",
            "谢谢观看！",
            "字幕由Amara.org社区提供",
            "請不吝點贊訂閱轉發打賞支持明鏡與點點欄目",
            "ご視聴ありがとうございました",
            "我哋嘅生活",
            "我哋嘅生活，就係咁樣嘅。",
            "我哋嘅生活，就係咁樣樣。",
            "多謝收看。",
        ] {
            assert!(is_likely_hallucination(text, 300), "{text:?}");
            assert!(
                is_likely_hallucination_with_duration(text, 300, 1000),
                "{text:?}"
            );
        }
    }

    #[test]
    fn long_speech_is_kept_even_when_it_is_a_known_phrase() {
        assert!(!is_likely_hallucination("Thank you.", MAX_VOICED_MS));
        assert!(!is_likely_hallucination("谢谢", 4000));
        assert!(!is_likely_hallucination_with_duration(
            "Thank you.",
            MAX_VOICED_MS,
            2500
        ));
        assert!(!is_likely_hallucination_with_duration("谢谢", 4000, 4500));
    }

    #[test]
    fn cantonese_prior_prefix_drops_rate_outliers_but_keeps_natural_speech() {
        // 10-second steady noise clip -> 1.0 chars/sec (< 2.5), even with voiced_ms >= MAX_VOICED_MS
        assert!(is_likely_hallucination_with_duration(
            "我哋嘅生活，就係咁樣嘅。",
            5000,
            10_000
        ));
        // Overly dense 0.5-second runaway burst -> 20 chars/sec (> 8.0)
        assert!(is_likely_hallucination_with_duration(
            "我哋嘅生活就係咁樣嘅",
            400,
            500
        ));
        // Real dictated Cantonese sentence at natural speed (~5.0 chars/sec) is preserved
        assert!(!is_likely_hallucination_with_duration(
            "我哋嘅生活其實一直都好簡單。",
            2200,
            2800
        ));
    }

    #[test]
    fn extreme_repetition_loops_above_token_rate_threshold_are_dropped() {
        let loop_text = "有好多嘅節奏，".repeat(12); // 72 chars in 3 seconds = 24 tok/s
        assert!(is_likely_hallucination_with_duration(
            &loop_text, 2500, 3000
        ));
        // Same repeated text over 12 seconds (6 tok/s <= 12.0) is not flagged
        assert!(!is_likely_hallucination_with_duration(
            &loop_text, 10_000, 12_000
        ));
    }

    #[test]
    fn real_short_dictation_is_kept() {
        for text in [
            "Thank you, John.",
            "Yes.",
            "OK",
            "See you tomorrow.",
            "谢谢你",
            "好的",
            "係呀，今日天氣好好。",
        ] {
            assert!(!is_likely_hallucination(text, 300), "{text:?}");
            assert!(
                !is_likely_hallucination_with_duration(text, 300, 1500),
                "{text:?}"
            );
        }
    }
}
