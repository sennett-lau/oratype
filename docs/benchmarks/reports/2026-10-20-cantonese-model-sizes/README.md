# Built-in models for Cantonese: smaller files, and where the time goes

The suggested built-in setup for Cantonese (plan `language-model-suggestions`) is Qwen3-ASR 1.7B
(Q8_0, 2.17 GB plus a 0.36 GB audio projector) for speech and Qwen 3.5 4B (Q4_K_M, 2.74 GB) for
polish. On a Mac it felt slow. This report measures smaller quantizations of the **same two
models** on the same Mac, for speed and quality, and finds where a dictation's time goes.

## Provenance

- `main` at eb99ead (OraType 1.2.0). Measured 2026-10-20 (JST).
- Machine: Apple M1 Pro, 32 GB, macOS 26.5.2. Server: llama.cpp `b11177` (the tag
  `scripts/build-llama-server.sh` pins), Metal, every layer on the GPU, started with the app's own
  arguments (`llm::builtin::launch_spec_for`): speech `--ctx-size 12288 --parallel 1 --mmproj …`,
  polish `--ctx-size 8192 --parallel 1 --reasoning off`, plus `--metrics`.
- Speech files: `Qwen3-ASR-1.7B-Q8_0.gguf` (shipped) from `ggml-org/Qwen3-ASR-1.7B-GGUF` at
  36a6786. That repository has no smaller files, so Q6_K, Q5_K_M and Q4_K_M were made here from its
  `Qwen3-ASR-1.7B-bf16.gguf` with the same llama.cpp tag: `llama-quantize <bf16> <out> Q4_K_M`. The
  audio projector stays `mmproj-Qwen3-ASR-1.7B-Q8_0.gguf` for all of them.
- Polish files: `Qwen3.5-4B-{Q4_K_M,Q4_0,IQ4_XS,Q3_K_M}.gguf` from `unsloth/Qwen3.5-4B-GGUF` at
  e87f176 (Q4_K_M is shipped).
- Data: the `evals/` dataset (synthetic speech from macOS `say`), and two long Cantonese clips
  made the same way from the eval sentences: 27.9 s and 56.1 s (`say -v Sinji`).
- In the background: an idle Typelite 1.1.3 (its two built-in servers loaded, no dictation) and
  an idle OraType Dev. No builds or other benchmarks ran meanwhile.
- Scripts: [scripts/](scripts/) (`bench.py`, `long.py`, `switch.py`, `proxy.py`; set `BENCH_DIR`
  to a folder with the model files in `dl/`). Raw results: [raw/](raw/).

## Speech recognition (Qwen3-ASR 1.7B)

| File | Size | Eval clips (24): pass / Cantonese error | 28 s clip | 56 s clip | Long transcripts vs Q8_0 | Resident memory |
|---|---:|---|---:|---:|---:|---:|
| Q8_0 (shipped) | 2.17 GB | 24 / 2% | 2.09 s | 4.41 s | — | 4.1 GB |
| Q6_K | 1.67 GB | 24 / 2% | 1.98 s | 3.90 s | identical | 3.6 GB |
| Q5_K_M | 1.47 GB | 24 / 2% | 1.89 s | 4.11 s | identical | 3.5 GB |
| **Q4_K_M** | **1.28 GB** | 24 / 2% | **1.50 s (−28%)** | **3.48 s (−21%)** | identical | 3.4 GB |

- Times are the median of 5 requests per clip. Before each one a different clip went through, so
  the server could not reuse the audio from its cache: a real dictation is always new audio.
  (Without that, a repeated clip took 1.73 s instead of 2.09 s on Q8_0.)
- Most of the time is writing the text, not reading the audio: the 56 s clip gives about 235
  output tokens at 56–58 tokens/s on Q8_0 and 72–81 tokens/s on Q4_K_M. Writing speed follows the
  file size (memory bandwidth), so a smaller file helps directly.
- Quality did not move on this data: every clip passed with the same error rate in every language,
  and all four files wrote character-for-character the same long transcripts (216 characters).
  The data is synthetic and clean; real voices and noise are harder, so the margin a 4-bit file
  leaves on real speech is not measured here.
- The eval clips' own throughput numbers ([raw/](raw/)) are not a speed measure: each clip gives
  about 17 output tokens, so request overhead dominates.

## AI polish (Qwen 3.5 4B)

Routed as a Cantonese user's app (`--languages en,zh-Hant-HK --presets`, presets
cantonese-hong-kong v4 and english v3): all 52 Cantonese cases (dev and holdout) with 3 samples
each, plus English and Mandarin dev cases (32 and 28) with 2 samples each as controls.

| File | Size | Cantonese | English | Mandarin | Writing speed | Prompt reading |
|---|---:|---:|---:|---:|---:|---:|
| Q4_K_M (shipped) | 2.74 GB | 76% | 89% | 79% | 28.9 tokens/s | 248 tokens/s |
| **Q4_0** | **2.58 GB** | **79%** | **91%** | **79%** | **39.4 tokens/s (+36%)** | **302 tokens/s (+22%)** |
| IQ4_XS | 2.48 GB | 77% | 86% | 82% | 35.0 tokens/s | 280 tokens/s |
| Q3_K_M | 2.29 GB | 75% | 86% | 71% | 31.0 tokens/s | 287 tokens/s |

- Speeds are totals over every request of the run (`/metrics`), about 4,300 written tokens each.
- Q4_0 is the fastest of the four: llama.cpp's Metal kernels are best for that format, so it beats
  the smaller IQ4_XS and Q3_K_M. Its pass rates are at least as high as Q4_K_M's (differences of
  2–3 points are within this dataset's noise).
- Q3_K_M is barely faster than Q4_0 and loses Mandarin (79% → 71%).

## Where a polish request's time goes: the prompt

With the language presets the polish prompt is long: about **4,080 tokens** for Cantonese and
**3,320** for English (14,600 and 12,300 characters). Reading it from scratch takes 8–15 s on this
Mac. The server keeps the last prompt in its cache, so the next dictation in the **same language**
reads only its own few dozen tokens (0.1–0.3 s).

When the language changes, the whole prompt is read again. The two prompts share their first
9,500 characters, but Qwen 3.5 is a hybrid model (recurrent layers), and llama.cpp cannot reuse
its cache from the middle of a prompt; only a match that runs almost to the end is reused.
Replaying real recorded prompts (3 Cantonese, 3 English, alternating, Q4_0;
[raw/polish-language-switch.json](raw/polish-language-switch.json)):

| Server setup | Every request after the first of each language |
|---|---|
| One slot (the app today) | 8.5–10.9 s each; the full prompt read every time |
| Two slots, the server picks | the same: it puts each request in the slot with the most similar cache, which is the other language's |
| **Two slots, one per language** (`--parallel 2`, `--ctx-size 16384`, `id_slot` in the request) | **0.3–0.9 s each**; 32–40 new tokens read |

The same happened inside the eval runs: each polish run read the full prompt ten times, 11–15 s
each on Q4_K_M, whenever a case routed to another language.

## What this means for one dictation

A 56 s Cantonese dictation on this Mac, polish answer of about 200 tokens:

| | Speech | Polish, same language as before | Polish, after another language (or the first after start) |
|---|---:|---:|---:|
| Today (Q8_0 + Q4_K_M, one slot) | 4.4 s | ~7.1 s | ~21 s |
| Smaller files (Q4_K_M + Q4_0) | 3.5 s | ~5.3 s | ~16 s |
| Smaller files and a slot per language | 3.5 s | ~5.3 s | ~5.3 s |

(Polish ≈ prompt reading + 200 tokens at the measured writing speed. Reading the whole Cantonese
prompt took about 14.3 s on Q4_K_M and 10.5 s on Q4_0 in the server logs; a warm request about
0.2 s.)

## Recommendation

1. **Polish: switch the Cantonese suggestion to Qwen 3.5 4B Q4_0** (`unsloth`, same revision).
   +36% writing and +22% reading speed, 160 MB smaller, no quality loss in this evaluation.
   A one-line model change (file, size, SHA-256) in `llm/models.rs`.
2. **Speech: switch to Qwen3-ASR 1.7B Q4_K_M** if a trusted copy can be hosted. 21–28% faster on
   long dictations, 0.9 GB smaller download and less memory, with no measurable quality change
   here. ggml-org publishes only Q8_0 and bf16, so the app would download a file made from
   ggml-org's bf16 with `llama-quantize` (as here), published with the model's Apache-2.0 notice
   in a repository we control, and pinned by SHA-256. Q6_K is the cautious middle (12% faster on
   the long clip) if real-voice tests show the 4-bit file slipping.
3. **The larger win is not the file: give each language its own polish slot.** Run the built-in
   polish server with `--parallel 2` and 16,384 tokens of context, and send each request with
   `id_slot` for its language route (and send a short warm-up per language when the server
   starts). A bilingual user then no longer waits 10–15 s for the prompt after each language change. This is a code change
   in `llm/builtin.rs` and its provider, not part of this report.

## Not measured

- Real Cantonese recordings and noisy rooms (all speech here is synthetic).
- Other Macs: 8 and 16 GB machines, and newer chips with more memory bandwidth.
- Quantized copies published by others (for example `mradermacher/Qwen3-ASR-1.7B-GGUF`); their
  compatibility with ggml-org's audio projector was not checked.
- Memory and speed of the two-slot setup over a long session (its second slot costs about one more
  8,192-token KV cache, which is small for this hybrid model: the whole polish server's own memory
  was 0.7 GB with one slot).
- The uncensored Qwen 3.5 4B that the Cantonese language guide mentions.
