//! Plan `language-router`: the character-based router. It needs no model, so it is also the
//! fallback when Laya is not installed.
//!
//! - **Units.** Each Chinese character counts as one unit, each run of Latin letters (a word)
//!   as one, each kana or Hangul character as one, and each word of another script as one.
//!   Digits, spaces and punctuation do not count.
//! - **Chinese** when Chinese characters are at least half of the units, or at least a fifth
//!   with a Cantonese-only character among them ("我覺得佢 is right"). Kana or Hangul makes
//!   it Japanese or Korean, which is `Other`.
//! - **Cantonese vs Mandarin.** Characters of written Cantonese that standard written Chinese
//!   does not use (嘅 咗 喺 啲 冇 嚟 唔 佢 㗎 囉 哋 ...) score 1 each; sentence particles that
//!   Mandarin also uses (喇 啦 呀) score 0.3. A score of 1 or more is Cantonese. Whisper often
//!   writes Cantonese speech as standard Chinese (CLAUDE.md), and such text reads as
//!   Mandarin here; nothing in the characters can tell them apart then.
//! - **Latin text** is English unless at least two letters, and at least 3 % of them, carry
//!   accents (French, German, ...), then `Other`. Unaccented Spanish or Italian reads as
//!   English.
//! - **Script.** Characters that exist in only one of the two forms (这/這, 们/們, ...) decide
//!   Simplified vs Traditional.

use async_trait::async_trait;

use super::{ChineseScript, Language, LanguageGuess, LanguageRouter, RouterError};

/// Written only in Cantonese.
const CANTONESE_STRONG: &[char] = &[
    '嘅', '咗', '喺', '啲', '冇', '嚟', '唔', '佢', '㗎', '囉', '哋', '嘢', '嗰', '咩', '乜', '睇',
    '諗', '搵', '攞', '啱', '冚', '嘥', '瞓', '咁', '噉', '畀',
];
/// Cantonese sentence particles that Mandarin also writes.
const CANTONESE_WEAK: &[char] = &['喇', '啦', '呀'];
const WEAK_WEIGHT: f32 = 0.3;

/// `(simplified, traditional)` pairs of common characters that differ between the forms.
const SCRIPT_PAIRS: &[(char, char)] = &[
    ('们', '們'),
    ('这', '這'),
    ('个', '個'),
    ('来', '來'),
    ('时', '時'),
    ('说', '說'),
    ('国', '國'),
    ('会', '會'),
    ('对', '對'),
    ('发', '發'),
    ('过', '過'),
    ('还', '還'),
    ('没', '沒'),
    ('问', '問'),
    ('题', '題'),
    ('请', '請'),
    ('帮', '幫'),
    ('订', '訂'),
    ('张', '張'),
    ('机', '機'),
    ('开', '開'),
    ('为', '為'),
    ('么', '麼'),
    ('样', '樣'),
    ('学', '學'),
    ('见', '見'),
    ('东', '東'),
    ('车', '車'),
    ('长', '長'),
    ('门', '門'),
    ('电', '電'),
    ('话', '話'),
    ('买', '買'),
    ('卖', '賣'),
    ('书', '書'),
    ('飞', '飛'),
    ('马', '馬'),
    ('鸟', '鳥'),
    ('鱼', '魚'),
    ('让', '讓'),
    ('认', '認'),
    ('识', '識'),
    ('经', '經'),
    ('给', '給'),
    ('华', '華'),
    ('业', '業'),
    ('实', '實'),
    ('现', '現'),
    ('应', '應'),
    ('该', '該'),
    ('关', '關'),
    ('头', '頭'),
    ('气', '氣'),
    ('两', '兩'),
    ('边', '邊'),
    ('进', '進'),
    ('运', '運'),
    ('动', '動'),
    ('岁', '歲'),
    ('钱', '錢'),
    ('银', '銀'),
    ('间', '間'),
    ('从', '從'),
    ('写', '寫'),
    ('听', '聽'),
    ('讨', '討'),
    ('论', '論'),
    ('饭', '飯'),
    ('吗', '嗎'),
    ('级', '級'),
    ('难', '難'),
    ('热', '熱'),
    ('网', '網'),
    ('线', '線'),
    ('几', '幾'),
    ('万', '萬'),
    ('与', '與'),
    ('条', '條'),
    ('号', '號'),
    ('处', '處'),
    ('务', '務'),
    ('员', '員'),
    ('单', '單'),
    ('标', '標'),
    ('声', '聲'),
    ('际', '際'),
    ('亲', '親'),
    ('爱', '愛'),
    ('节', '節'),
    ('礼', '禮'),
    ('备', '備'),
    ('决', '決'),
];

/// Counts of one text (see the module comment).
#[derive(Debug, Default, Clone, PartialEq)]
struct Counts {
    chinese: usize,
    latin_words: usize,
    latin_letters: usize,
    accented_letters: usize,
    kana_or_hangul: usize,
    other_words: usize,
    cantonese_score: f32,
    simplified: usize,
    traditional: usize,
}

impl Counts {
    fn of(text: &str) -> Self {
        let mut counts = Counts::default();
        let mut in_latin = false;
        let mut in_other = false;
        for c in text.chars() {
            let latin = is_latin_letter(c);
            let other = !latin && c.is_alphabetic() && !is_chinese(c) && !is_kana_or_hangul(c);
            if latin {
                counts.latin_letters += 1;
                if !c.is_ascii() {
                    counts.accented_letters += 1;
                }
                if !in_latin {
                    counts.latin_words += 1;
                }
            }
            if other && !in_other {
                counts.other_words += 1;
            }
            in_latin = latin;
            in_other = other;
            if is_chinese(c) {
                counts.chinese += 1;
                if CANTONESE_STRONG.contains(&c) {
                    counts.cantonese_score += 1.0;
                } else if CANTONESE_WEAK.contains(&c) {
                    counts.cantonese_score += WEAK_WEIGHT;
                }
                if SCRIPT_PAIRS.iter().any(|&(s, _)| s == c) {
                    counts.simplified += 1;
                } else if SCRIPT_PAIRS.iter().any(|&(_, t)| t == c) {
                    counts.traditional += 1;
                }
            } else if is_kana_or_hangul(c) {
                counts.kana_or_hangul += 1;
            }
        }
        counts
    }

    fn units(&self) -> usize {
        self.chinese + self.latin_words + self.kana_or_hangul + self.other_words
    }

    fn script(&self) -> ChineseScript {
        match self.simplified.cmp(&self.traditional) {
            std::cmp::Ordering::Greater => ChineseScript::Simplified,
            std::cmp::Ordering::Less => ChineseScript::Traditional,
            std::cmp::Ordering::Equal => ChineseScript::Unknown,
        }
    }
}

fn is_chinese(c: char) -> bool {
    matches!(c,
        '\u{3400}'..='\u{4DBF}'      // Extension A (㗎 is here)
        | '\u{4E00}'..='\u{9FFF}'    // Unified Ideographs
        | '\u{F900}'..='\u{FAFF}'    // Compatibility Ideographs
        | '\u{20000}'..='\u{3134F}') // Extensions B to G (many Cantonese characters)
}

fn is_kana_or_hangul(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{30FF}'      // Hiragana, Katakana
        | '\u{31F0}'..='\u{31FF}'
        | '\u{1100}'..='\u{11FF}'    // Hangul Jamo
        | '\u{3130}'..='\u{318F}'
        | '\u{AC00}'..='\u{D7AF}') // Hangul syllables
}

fn is_latin_letter(c: char) -> bool {
    c.is_ascii_alphabetic()
        || (c.is_alphabetic()
            && matches!(c, '\u{00C0}'..='\u{024F}' | '\u{1E00}'..='\u{1EFF}')
            && c != '\u{00D7}'
            && c != '\u{00F7}')
}

/// Decides from the characters alone.
#[derive(Debug, Default, Clone, Copy)]
pub struct CharRouter;

impl CharRouter {
    pub fn guess(&self, text: &str) -> LanguageGuess {
        let counts = Counts::of(text);
        let units = counts.units();
        if units == 0 {
            return LanguageGuess {
                language: Language::Other,
                confidence: 0.0,
                script: None,
            };
        }
        let share = |n: usize| n as f32 / units as f32;
        let chinese_share = share(counts.chinese);
        if counts.kana_or_hangul > 0 && counts.kana_or_hangul * 5 >= counts.chinese {
            // Japanese (kana between the kanji) or Korean.
            return LanguageGuess {
                language: Language::Other,
                confidence: share(counts.kana_or_hangul + counts.chinese).min(0.95),
                script: None,
            };
        }
        let cantonese = counts.cantonese_score >= 1.0;
        if chinese_share >= 0.5 || (cantonese && chinese_share >= 0.2) {
            let script = Some(counts.script());
            if cantonese {
                // One Cantonese-only character: 0.75; two: 0.875; ...
                let sure = 1.0 - 0.5f32.powf(counts.cantonese_score + 1.0);
                return LanguageGuess {
                    language: Language::Cantonese,
                    confidence: (sure * (0.5 + chinese_share).min(1.0)).min(0.99),
                    script,
                };
            }
            // No Cantonese character: surer the longer the text is.
            let sure = 1.0 - 0.5f32.powf(1.0 + counts.chinese as f32 / 8.0);
            return LanguageGuess {
                language: Language::Mandarin,
                confidence: (sure * chinese_share).min(0.95),
                script,
            };
        }
        if counts.latin_words >= counts.other_words {
            let accented = counts.accented_letters >= 2
                && counts.accented_letters as f32 >= 0.03 * counts.latin_letters as f32;
            let latin_share = share(counts.latin_words);
            return LanguageGuess {
                language: if accented {
                    Language::Other
                } else {
                    Language::English
                },
                confidence: (latin_share * if accented { 0.7 } else { 0.9 }).min(0.95),
                script: (counts.chinese > 0).then(|| counts.script()),
            };
        }
        LanguageGuess {
            language: Language::Other,
            confidence: share(counts.other_words).min(0.9),
            script: None,
        }
    }
}

#[async_trait]
impl LanguageRouter for CharRouter {
    async fn route(&self, text: &str) -> Result<LanguageGuess, RouterError> {
        Ok(self.guess(text))
    }
}

/// Simplified vs Traditional for any text (`Unknown` when it has no telling character).
pub fn chinese_script(text: &str) -> ChineseScript {
    Counts::of(text).script()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn language(text: &str) -> Language {
        CharRouter.guess(text).language
    }

    #[test]
    fn english_mandarin_and_cantonese_sentences() {
        for text in [
            "Can you send me the report by Friday?",
            "Remind me to call mom tonight.",
            "OK, see you tomorrow at 3 p.m.",
            "I had a café au lait.",
        ] {
            assert_eq!(language(text), Language::English, "{text}");
        }
        for text in [
            "我明天要去北京开会，请帮我订一张机票。",
            "這個問題我們下週再討論吧，今天時間不夠了。",
            "你今天晚上有空吗？我们一起去吃饭啦。",
        ] {
            assert_eq!(language(text), Language::Mandarin, "{text}");
        }
        for text in [
            "我聽日要去北京開會，唔該幫我訂張機票啦。",
            "佢哋啱啱食咗飯，你嚟唔嚟呀？",
            "今日好熱㗎",
            "你喺邊度？",
        ] {
            assert_eq!(language(text), Language::Cantonese, "{text}");
        }
    }

    #[test]
    fn mixed_cantonese_and_english_counts_as_cantonese() {
        assert_eq!(
            language("我哋今個 project 嘅 deadline 係下個禮拜五。"),
            Language::Cantonese
        );
        assert_eq!(
            language("我覺得佢 is right about the budget"),
            Language::Cantonese
        );
        // A few Chinese characters in English text without Cantonese ones stay English.
        assert_eq!(
            language("Please book the table at 北京樓 for Friday night"),
            Language::English
        );
        assert_eq!(
            language("这个 project 的 deadline 是下周五，大家抓紧时间。"),
            Language::Mandarin
        );
    }

    #[test]
    fn particles_mandarin_also_uses_count_less() {
        // 啦 and 呀 are 0.3 each; three of them are still under 1.
        assert_eq!(language("好啦好啦，走呀"), Language::Mandarin);
        assert_eq!(language("好啦好啦好啦，走呀"), Language::Cantonese);
    }

    #[test]
    fn other_languages() {
        assert_eq!(language("明日の会議は十時からです。"), Language::Other);
        assert_eq!(language("안녕하세요, 만나서 반갑습니다"), Language::Other);
        assert_eq!(
            language("Je voudrais réserver une table près de la fenêtre."),
            Language::Other
        );
        assert_eq!(language("Привет, как дела?"), Language::Other);
        let empty = CharRouter.guess(" 123 !? ");
        assert_eq!(empty.language, Language::Other);
        assert_eq!(empty.confidence, 0.0);
    }

    #[test]
    fn confidence_grows_with_evidence() {
        let one = CharRouter.guess("佢好忙");
        let three = CharRouter.guess("佢哋啱啱食咗飯");
        assert!(one.confidence >= 0.7 && one.confidence < three.confidence);
        let short = CharRouter.guess("好的");
        let long = CharRouter.guess("我明天要去北京开会，请帮我订一张机票，谢谢你。");
        assert!(short.confidence < long.confidence);
        assert!(long.confidence > 0.8);
        for guess in [one, three, short, long] {
            assert!((0.0..=1.0).contains(&guess.confidence));
        }
    }

    #[test]
    fn simplified_and_traditional_are_told_apart() {
        assert_eq!(
            CharRouter.guess("我们这个问题").script,
            Some(ChineseScript::Simplified)
        );
        assert_eq!(
            CharRouter.guess("我們這個問題").script,
            Some(ChineseScript::Traditional)
        );
        assert_eq!(chinese_script("我好忙"), ChineseScript::Unknown);
        assert_eq!(CharRouter.guess("Hello there").script, None);
    }

    #[test]
    fn the_script_pairs_really_differ() {
        for (index, &(simplified, traditional)) in SCRIPT_PAIRS.iter().enumerate() {
            assert_ne!(simplified, traditional);
            assert!(is_chinese(simplified) && is_chinese(traditional));
            assert!(!SCRIPT_PAIRS.iter().any(|&(s, _)| s == traditional));
            assert!(!SCRIPT_PAIRS[index + 1..]
                .iter()
                .any(|&(s, _)| s == simplified));
        }
    }

    #[tokio::test]
    async fn the_router_trait_answers_without_a_model() {
        let router: Box<dyn LanguageRouter> = Box::new(CharRouter);
        let guess = router.route("唔該晒").await.unwrap();
        assert_eq!(guess.language, Language::Cantonese);
    }
}
