# Dictionary to the speech model

The personal dictionary used to reach only polish. By then a name the recognizer misheard
("Senate" for Sennett, "or type" for OraType) is already wrong, and polish can only guess. This
plan gives the dictionary's words to the speech recognizer as well, so the names come out
spelled as the user writes them.

Status: building - 2026-11-08

## Goals and non-goals

- Words in the dictionary come out of speech recognition spelled as in the dictionary, case
  included, for Dictate and Ask.
- Words that are not said are not forced in: "or type your name" stays "or type".
- No slower runs and no change for users without a dictionary.
- Measured with `evals/`: a dictionary per speech clip, clips that say the words and clips that
  say only sound-alikes.
- Not a goal: Qwen Cloud and ElevenLabs. Qwen Cloud's API takes context in a system message and
  ElevenLabs Scribe has key terms, but neither could be measured here; each can follow with its
  own eval run.
- Not a goal: a setting. The dictionary page already says its words improve recognition.

## Key decisions

- **One field, `SttConfig.vocabulary`.** Dictate and Ask fill it from the dictionary store;
  each provider decides how to send it. Reason: one place to read, and providers that ignore it
  are unchanged.
- **OpenAI-compatible uploads send the words as the `prompt` form field.** This covers
  `llama-server` with Qwen3-ASR (built-in and a server) and Whisper servers. Reason: the app
  already uses `/v1/audio/transcriptions`. llama-server puts `prompt` in place of its
  "Transcribe audio to text" instruction before the audio, and Qwen3-ASR then spells the words as
  given; it gave the same transcripts as a system message through `/v1/chat/completions` on four probe clips, so
  no second request format is needed. Whisper servers read it as earlier text, which is OpenAI's
  documented use of `prompt`.
- **Built-in whisper.cpp gets the words as prompt tokens**, tokenized by the loaded model, with
  a full stop after the list. Reason: whisper-rs's `set_initial_prompt` leaks its string on every
  run; and Whisper copies the prompt's style, so a list without a stop dropped the final
  punctuation in four of 17 clips.
- **A plain comma-separated list, capped at 40 words and 400 characters**, blanks, repeats and
  entries over 40 characters skipped, in dictionary order (`stt/vocabulary.rs`). Reason: Whisper
  reads at most 224 prompt tokens, and every word costs Qwen3-ASR a few tokens on each run.
- **A transcript that is only dictionary words, two or more, separated by commas, is dropped**
  as no speech ("dictionary echo" in the log). Reason: Whisper can read its prompt back when the
  audio says little; a person rarely dictates only a list of their own dictionary words. Not seen
  in the evaluation, so this is a guard, not a fix.

## Results

[Report](../../benchmarks/reports/2026-11-08-dictionary-to-speech/README.md): dictionary words
spelled right went from 3 to 16 of 24 with Qwen3-ASR 1.7B and from 6 to 15 of 24 with Whisper
large-v3-turbo; no word was forced into a clip that did not say it; the 24 clips without a
dictionary did not change, and runs were no slower.

## Parts

| File | Covers |
|---|---|
| `src-tauri/src/stt/vocabulary.rs` | The list sent, its caps and the echo guard. |
| [evals/README.md](../../../evals/README.md) | `dictionary`, `terms` and `must_not_contain` on speech clips; `--dictionary off`. |

## Considered

- **`/v1/chat/completions` with a system message for Qwen3-ASR**, as Qwen3-ASR's model card
  shows. Same transcripts as the `prompt` field on four probe clips, and a second request format.
- **Correcting names after recognition** (fuzzy matching against the dictionary). Polish already
  does this with the whole sentence as context; the recognizer can use the sound, which polish
  never hears.

## Open questions

- Chinese names that are homophones of common words (葉梓聰 heard as 叶子涌 after 鰂魚涌) were
  not fixed by either model. Whether a pronunciation in the dictionary entry helps is untested.
- Whether the targets of correction rules ("or type" → "OraType") should be sent as well.
- Real recordings: the clips are macOS `say` voices, which pronounce some names oddly (Tauri as
  "Tory").
