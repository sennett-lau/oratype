# Built-in models for Cantonese: Qwen3-ASR 1.7B and Qwen 3.5 4B

Evidence for the first suggested setup of plan `language-model-suggestions`: any combination of
spoken languages with Cantonese gets Qwen3-ASR 1.7B for speech recognition and Qwen 3.5 4B for
AI polish, both run by the `llama-server` that ships with Typelite.

## Provenance

- Branch `feature/language-model-suggestions`, from `main` at ff631ff. Measured 2026-10-16 (JST).
- Machine: Apple M1 Pro, 32 GB, macOS 26.
- Server: the bundled `llama-server` (llama.cpp `b11177`, the tag `scripts/build-llama-server.sh`
  pins), Metal, every layer on the GPU.
- Speech: `Qwen3-ASR-1.7B-Q8_0.gguf` with `mmproj-Qwen3-ASR-1.7B-Q8_0.gguf` from
  `ggml-org/Qwen3-ASR-1.7B-GGUF` at 36a6786, `--ctx-size 8192 --parallel 1`. Baseline: whisper
  large-v3-turbo q5_0 (`evals/baselines/speech-ggml-large-v3-turbo-q5-0.json`).
- Polish: `Qwen3.5-4B-Q4_K_M.gguf` from `unsloth/Qwen3.5-4B-GGUF` at e87f176, with the app's
  arguments (`--ctx-size 4096 --parallel 1 --reasoning off`) and the app's polish prompt.
  Baseline: Qwen3 4B Instruct 2507 Q4_K_M (`evals/baselines/polish-qwen3-4b-instruct-2507-q4-k-m.json`,
  same prompt).
- Data: the `evals/` dataset. Speech clips are synthetic (macOS `say`), so they are cleaner than
  real speech.

Commands:

```sh
npm run eval -- synth
npm run eval -- speech --speech-url http://127.0.0.1:<port>/v1 --speech-model qwen3-asr-1.7b --split all
npm run eval -- polish --llama-model <path>/Qwen3.5-4B-Q4_K_M.gguf --split dev --n 3
```

The full summaries are [speech-qwen3-asr.md](speech-qwen3-asr.md) and
[polish-qwen3.5-4b-dev.md](polish-qwen3.5-4b-dev.md).

## Speech recognition

| Language (8 clips each) | whisper large-v3-turbo | Qwen3-ASR 1.7B |
|---|---:|---:|
| Cantonese, pass / mean error | 0 of 8 / 44% | 8 of 8 / 2% |
| Mandarin, pass / mean error | 8 of 8 / 2% | 8 of 8 / 2% |
| English, pass / mean error | 8 of 8 / 2% | 8 of 8 / 3% |

No clip got worse; the 8 Cantonese clips improved. Whisper writes Cantonese as standard written
Chinese (佢頭先話…俾我哋 becomes 他剛才說…給我們); Qwen3-ASR writes the spoken words. Its answers
are in Simplified characters with a `language Cantonese<asr_text>` tag; the app removes the tag
and writes routed Cantonese in Hong Kong characters (plan `qwen3-asr-support`).

Speed on the M1 Pro: about 0.35–0.5 s per short clip (2–5 s of audio); 52.6 s of audio in one
request used 696 tokens, about 13 tokens per second of audio. The server starts in 3–4 s once
macOS has compiled the Metal shaders, and uses about 2.9 GB of memory.

## AI polish (dev split, 3 samples per case)

| Language | Cases | Qwen3 4B Instruct | Qwen 3.5 4B |
|---|---:|---:|---:|
| All | 114 | 78% | 78% |
| Cantonese (yue) | 30 | 70% | 78% |
| Mandarin (zh-Hans) | 28 | 86% | 77% |
| English | 32 | 92% | 85% |
| Spanish | 8 | 50% | 79% |
| French | 8 | 88% | 75% |
| Japanese | 8 | 42% | 58% |

Qwen 3.5 4B is level overall: better on Cantonese, Spanish and Japanese, worse on English,
French and Mandarin. Spanish, French and Japanese have 8 cases each, so their differences are
within noise. This is why it is suggested only for combinations with Cantonese, and the standard
Qwen3 4B stays the default otherwise.

Cantonese cases that changed by more than one sample: yue-013 (numbers, mixed language), yue-022
(dialect) and yue-024 (fillers) went from 0% to 100%; yue-025 (Hong Kong characters) went from 100%
to 33%. Cantonese self-corrections (唔係…) still fail with both models.

## Not measured

- Real Cantonese recordings: the speech clips are synthetic.
- The uncensored Qwen 3.5 4B that the Cantonese language guide recommends for slang and swear
  words; the suggestion uses the stock model.
- Macs with 8 GB of memory, where Qwen3-ASR (about 2.9 GB) shares memory with the faster AI model.
