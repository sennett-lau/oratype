# Dictionary words given to the speech recognizer

A quality change, measured with `evals/` (plan
[`dictionary-to-speech`](../../../plans/2026-11-08-dictionary-to-speech/index.md)). The personal
dictionary reached only polish, so a name the recognizer misheard arrived already wrong:
"Siasha moved the vectors from Pinecone to Quadrant" for "Saoirse … Qdrant", "Oragrip" for
OraType.

## Provenance

| Field | Value |
|---|---|
| Date | 2026-11-08 |
| Kind | Quality change (code) |
| Revision | `main` at `dac1dbd` plus this PR; "without" runs use this PR's build with `--dictionary off`, which sends what `main` sends |
| Hardware | Apple M1 Pro, macOS 26 (built-in Whisper); an OpenAI-compatible llama-server on a separate PC (CUDA) for Qwen3-ASR |
| Qwen3-ASR | llama-server, `Qwen3-ASR-1.7B-Q8_0.gguf` with its mmproj, through `/v1/audio/transcriptions` |
| Whisper | built-in whisper.cpp, `ggml-large-v3-turbo-q5_0.bin` |
| Data | 17 new `dictionary` clips (en 7, yue 6, zh-Hans 4; 13 dev, 4 holdout), macOS `say` voices; 11 say dictionary words (24 terms), 6 say only sound-alikes or none; plus the 24 earlier clips as controls. One run per setting (greedy decoding). |

## Change

Dictate and Ask pass the dictionary to the speech provider (`SttConfig.vocabulary`). Uploads to
an OpenAI-compatible server send the words as the `prompt` field; built-in Whisper gets them as
prompt tokens followed by a full stop. At most 40 words and 400 characters
(`stt/vocabulary.rs`). A transcript that is only dictionary words is dropped as no speech.

## Results

Dictionary terms spelled exactly as in the dictionary, case included (of 24), and dictionary
words forced into clips that do not say them:

| Model | Terms, without | Terms, with | Forced, without | Forced, with |
|---|---:|---:|---:|---:|
| Qwen3-ASR 1.7B | 3 | 16 | 0 | 0 |
| Whisper large-v3-turbo | 6 | 15 | 0 | 0 |

Dictionary clips passed (error ≤ 15%, terms right, nothing forced; of 17):

| Model | Without | With |
|---|---:|---:|
| Qwen3-ASR 1.7B | 6 | 11 |
| Whisper large-v3-turbo | 4 | 7 |

The 24 earlier clips (no dictionary) gave identical results in all runs: Qwen3-ASR 24 of 24,
Whisper 16 of 24 (its Cantonese written as Mandarin, as before). Against the stored Whisper
baseline: no regressions.

Average time per clip did not grow: Qwen3-ASR 120 ms without and 133 ms with on the dictionary
clips (network noise of the same size as the difference), Whisper 951 ms and 970 ms.

Examples (without → with):

- Qwen3-ASR: "Tyge said the superbase migration" → "Tadhg said the Supabase migration";
  "啲录已经send咗去Griffin，你睇下Cubanets" → "…Grafana，你睇下Kubernetes"; "Aura Type" → "OraType".
- Whisper: "Penacone … Juant" → "Pinecone … Drant" (Qdrant still wrong); "陳海晴說Super Base" →
  "陳凱澄說 Supabase".
- Sound-alikes unchanged with the words given: "The Tory leader …" (Tauri in the dictionary),
  "Please sign here or type your name" and "或者type個名" (OraType in the dictionary).

## Whisper's final punctuation

A first Whisper run sent the bare list. Whisper copies its prompt's style: four of the 17
dictionary clips lost their final 。 ("…已经完成了"). Ending the list with a full stop brought it
back in all four, with the same terms right (15 of 24). Qwen3-ASR's prompt has no stop: its
answers kept their punctuation without one.

## Left wrong

- Irish names in a `say` voice: Niamh came out "Neve", "Niam" or "Nieve" from Qwen3-ASR in all
  four clips that say it, and Siobhan as "Shavon". Whisper got Niamh in three of the four.
- 葉梓聰 after 鰂魚涌 stays 叶子涌 in both models (聰 and 涌 are both *cung1*).
- Whisper wrote "update or OraType" (it kept the heard "or" and added the word) and "Ni幫我同
  Niamh講", the name at the start as well.

## Limitations

- Synthetic voices only; they pronounce some names oddly (Tauri as "Tory"), which makes those
  terms harder than real speech would.
- One run per setting. Greedy decoding, so reruns give the same text, but the clips are few;
  read the term counts, not the pass rates per language.
- Qwen Cloud and ElevenLabs were not measured and do not get the words yet.
- The echo guard was never triggered here; it is a guard for short noisy recordings.
