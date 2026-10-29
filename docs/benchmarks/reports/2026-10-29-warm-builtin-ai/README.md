# Built-in AI: where a session's time goes, and keeping it warm

For plan [`warm-builtin-ai`](../../../plans/2026-10-29-warm-builtin-ai/index.md). It replays a
scripted dictation session against three polish models with the app's `llama-server` arguments
and the app's real polish prompts, and measures what each request costs: tokens read fresh against
tokens taken from the server's cache, read and write speed, and wall time.

## Setup

- Mac: M1 Pro, 32 GB, macOS 26. llama.cpp `b11177` (the bundled build), all layers on the GPU,
  `--ctx-size 8192` per slot, `--reasoning off`, as the app starts it.
- Models: Qwen3 1.7B Q4_K_M, Qwen 3.5 4B Q4_0 (the Cantonese suggestion,
  plan [`faster-cantonese-models`](../../../plans/2026-10-20-faster-cantonese-models/index.md)),
  Qwen3 4B Instruct 2507 Q4_K_M (the general model).
- Prompts: recorded from the app's own prompt code (`npm run eval` through the
  [proxy](../2026-10-20-cantonese-model-sizes/scripts/proxy.py)), routed as a Cantonese user's
  app (`--languages en,zh-Hant-HK --presets`): about 4,050 tokens for Cantonese (preset
  `cantonese-hong-kong` v4) and 3,330 for English (`english` v3). Transcripts are the eval
  inputs; a "long" dictation joins eight of them (about a minute of speech).
- [scripts/session.py](scripts/session.py), one server per model:
  1. the first dictation after the server starts (after the app's short setup test)
  2. 6 short Cantonese dictations
  3. 4 long ones
  4. a language switch: en, yue, en, yue, en long, yue long
  5. a 30-dictation session, 75% Cantonese and 25% English, 40% long (fixed seed, the same for every model)
  6. a second start with a warm-up request before the first dictation

  [scripts/appswitch.py](scripts/appswitch.py) changes only the app part of the instructions.
  [scripts/mem.py](scripts/mem.py) measures the server's memory.
- Quality: `npm run eval -- polish` as in the
  [2026-10-20 report](../2026-10-20-cantonese-model-sizes/README.md): 52 Cantonese cases ×3,
  English and Mandarin dev ×2.

Each phase is one run, so treat differences under about 10% as noise. The first Qwen 3.5 4B run
overlapped with dictations in the dev app; the numbers below are from a clean rerun.

## Results

Median seconds per request, unless marked.

| | Qwen3 1.7B | Qwen 3.5 4B Q4_0, 1 slot | Qwen 3.5 4B, slot per language | Qwen3 4B, 1 slot | Qwen3 4B, slot per language |
|---|---:|---:|---:|---:|---:|
| First dictation after start | 2.5 | 11.1 | 10.7 | 5.5 | 13.7 |
| First dictation after a warm-up | 0.48 | 0.84 | 0.92 | 1.12 | 1.06 |
| Short Cantonese, warm | 0.33 | 0.61 | 0.60 | 0.60 | 0.62 |
| Long Cantonese, warm | 2.6 | 4.5 | 4.4 | 5.2 | 5.5 |
| Language switch (median / worst) | 2.4 / 4.2 | 11.1 / 15.0 | 0.9 / 4.3 | 5.6 / 10.1 | 1.1 / 5.4 |
| Another app, same language | 0.6 | 2.2 | — | 1.2 | — |
| 30-dictation session, total | 52 | 171 | 64 | 117 | 86 |
| Requests that re-read the whole prompt | 0 of 48 | 18 of 48 | 1 of 48 | 0 of 48 | 1 of 48 |
| Server memory (RSS) | 2.1 GB | 3.1 GB | 4.2 GB | 3.7 GB | 5.9 GB |
| Writing speed (tokens/s) | ~65 | ~37 | ~37 | ~32 | ~32 |
| Polish quality: Cantonese / English / Mandarin | 52%* | 79% / 91% / 79% | same | 76% / 92% / 88% | same |

\* One sample per case (the other columns take three for Cantonese), Cantonese only.

## What it shows

- **The 4B models are not slow once warm.** A short Cantonese dictation takes about 0.6 s on
  either; Qwen 3.5 4B writes faster than Qwen3 4B on long ones.
- **The slow requests are re-reads of the instructions.** The first request after a start reads
  all of them: 11 s on Qwen 3.5 4B, 5.5 s on Qwen3 4B, which reuses the 2,576 tokens the
  setup test shares with them.
- **Language switches hurt Qwen 3.5 4B most.** Qwen 3.5 is a hybrid model: llama.cpp cannot cut
  its cache at an arbitrary point, only restore a checkpoint it saved about 512 tokens before the
  end of the last prompt. The Cantonese and English instructions part at about 65% of the way
  through, so every switch re-reads all ~4,000 tokens (11 s). Qwen3 models keep the shared
  2,576 tokens and read about 1,130 (1.6–3.8 s).
- **An app switch is cheap for every model.** The app's part comes at 93–97% of the
  instructions, after Qwen 3.5's checkpoint, so it re-reads about 500 tokens (2.2 s).
- **A warm-up at start removes the first slow request**, and a slot per language removes the
  switch: the session drops from 171 s to 64 s on Qwen 3.5 4B (117 s to 86 s on Qwen3 4B). The
  cost is memory: about 1.1 GB per extra slot on Qwen 3.5 4B and 2.2 GB on Qwen3 4B.
- **Qwen3 1.7B is the fastest everywhere but writes Cantonese poorly** (52%), so it stays a
  fallback for small Macs.
- **Qwen3 4B stays the general model.** It matches Qwen 3.5 4B on Cantonese (76% vs 79%, within
  noise) and English, and is clearly better on Mandarin (88% vs 79%).

## Reproduce

Record prompts once (any model; the proxy saves each request body to `prompts/`):

```sh
llama-server --model <model.gguf> --port 8311 --ctx-size 8192 --reasoning off &
python3 -I ../2026-10-20-cantonese-model-sizes/scripts/proxy.py http://127.0.0.1:8311 8312 $BENCH_DIR/prompts &
npm run eval -- polish --ai-url http://127.0.0.1:8312/v1 --ai-model rec --lang yue --split all --n 1 --languages en,zh-Hant-HK --presets --baseline none --out /tmp/rec-yue
npm run eval -- polish --ai-url http://127.0.0.1:8312/v1 --ai-model rec --lang en --split all --n 1 --languages en,zh-Hant-HK --presets --baseline none --out /tmp/rec-en
```

Then, with `BENCH_DIR` holding `prompts/`:

```sh
python3 -I scripts/session.py <model.gguf> <label> 1   # one slot (slots off)
python3 -I scripts/session.py <model.gguf> <label> 2   # one slot per language
python3 -I scripts/appswitch.py <model.gguf>
python3 -I scripts/mem.py <model.gguf> 1
```

The per-request rows of these runs were not kept; the tables above are their summaries.
