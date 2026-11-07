//! Speech recognizers (Qwen3-ASR, Whisper) put a Chinese or Japanese comma (，、) at every
//! pause, so a transcript such as "我想問下你，聽日，得唔得閒" splits one thought into pieces.
//! The polish model copies those commas instead of punctuating by grammar. Removing them from
//! the text sent to polish lets the model place commas where the sentence needs them; the
//! sentence ends (。？！) stay, and so does the transcript pasted when polish is off or fails.

use std::borrow::Cow;

/// The transcript without its Chinese and Japanese pause commas (，、).
pub fn without_pause_commas(transcript: &str) -> Cow<'_, str> {
    if transcript.contains(['，', '、']) {
        Cow::Owned(transcript.replace(['，', '、'], ""))
    } else {
        Cow::Borrowed(transcript)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_pause_commas_and_keeps_sentence_ends() {
        assert_eq!(
            without_pause_commas("佢話，佢今晚，會遲少少返嚟。你食咗飯未？"),
            "佢話佢今晚會遲少少返嚟。你食咗飯未？"
        );
        assert_eq!(
            without_pause_commas("明日の午後に、資料を送ります"),
            "明日の午後に資料を送ります"
        );
    }

    #[test]
    fn leaves_other_text_alone() {
        assert!(matches!(
            without_pause_commas("Milk, eggs, and 1,000 notes"),
            Cow::Borrowed("Milk, eggs, and 1,000 notes")
        ));
    }
}
