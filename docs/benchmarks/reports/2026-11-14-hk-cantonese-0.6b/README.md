# Qwen3-ASR-0.6B-HK-Cantonese as a built-in speech model

OraType's built-in Qwen3-ASR model is Qwen3-ASR-1.7B-HK-Cantonese, version 2, Q6_K (report
[`2026-11-10-hk-cantonese-asr`](../2026-11-10-hk-cantonese-asr/README.md)). The OraType team has
since published a fine-tune of the smaller Qwen3-ASR 0.6B for the same speech,
[`joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF`](https://huggingface.co/joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF)
(Apache-2.0). This report measures its three files against the shipped 1.7B fine-tune and the
stock Qwen3-ASR 0.6B, to decide whether it can be a built-in model.

No code changes here; the decision is below.

## Provenance

- Measured 2026-11-14 (JST) on an Apple M1 Pro, 32 GB, macOS 26.5.2, `main` at 1d8cd58, with the
  bundled `llama-server` (`src-tauri/binaries`, commit 1ab7e5a, Metal, every layer on the GPU)
  and the app's speech arguments: `--ctx-size 12288 --parallel 1 --mmproj …`.
- Files (SHA-256 checked against each model card and `stt/models.rs`):
  - 0.6B fine-tune: revision 9b67dd0, `Qwen3-ASR-0.6B-Cantonese-HK.{Q4_K_M,Q6_K}.gguf` and
    `Qwen3-ASR-0.6B-Cantonese-HK-Q8_0.gguf`, each with the fine-tune's own
    `Qwen3-ASR-0.6B-Cantonese-HK.mmproj-Q8_0.gguf` (214 MB);
  - shipped: `joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF` at c2b90a4, Q6_K with its Q8_0 projector;
  - stock 0.6B: `mradermacher/Qwen3-ASR-0.6B-GGUF` at 39c0af5, `Qwen3-ASR-0.6B.Q6_K.gguf` with its
    Q8_0 projector.
- Data: the 41 speech clips of `evals/` (15 English, 14 Cantonese, 12 Mandarin; macOS `say`,
  dictionary on), one sample each. Speed: the two long Cantonese clips of the previous report
  (27.9 s and 56.1 s, `say -v Sinji`), median of 5 requests, each after a different clip so the
  server cannot reuse audio.
- The Mac was in normal use (load average 6–8 of 10 cores). Round 2 ran the 1.7B file during a
  load spike (medians 2.0–2.5 s on the short clip), so the speed table uses round 3: three
  alternated cycles (1.7B, 0.6B Q6_K, 0.6B Q4_K_M, repeat). All rounds are in [raw/](raw/).
- Harness: the previous report's
  [bench.py](../2026-11-10-hk-cantonese-asr/scripts/bench.py), unchanged.

## Results

| File | Download (model + projector) | Clips passed (of 41) | English | Cantonese | Mandarin | Cantonese mean error | English written in Chinese | Cantonese words in Mandarin clips |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **1.7B fine-tune v2 Q6_K (shipped)** | 1.77 GB | 34 | 12 / 15 | 12 / 14 | 10 / 12 | 4.0% | 1 | 2 |
| 0.6B stock Q6_K | 0.84 GB | 33 | 12 / 15 | 10 / 14 | 11 / 12 | 7.1% | 0 | 0 |
| 0.6B fine-tune Q4_K_M | 0.61 GB | 33 | 12 / 15 | 11 / 14 | 10 / 12 | 5.9% | 0 | 1 |
| **0.6B fine-tune Q6_K** | **0.71 GB** | **34** | **13 / 15** | **11 / 14** | **10 / 12** | **6.1%** | **0** | **1** |
| 0.6B fine-tune Q8_0 | 0.85 GB | 33 | 13 / 15 | 10 / 14 | 10 / 12 | 7.4% | 0 | 1 |

Speed, round 3 (three alternated cycles, range of the medians):

| File | 28 s clip | 56 s clip | Resident memory |
|---|---:|---:|---:|
| 1.7B fine-tune v2 Q6_K (shipped) | 1.52–1.62 s | 3.07–3.63 s | 3.2–3.4 GB |
| 0.6B fine-tune Q6_K | 0.70–0.74 s (−54%) | 1.48–1.70 s (−52%) | 2.3 GB |
| 0.6B fine-tune Q4_K_M | 0.66–0.71 s | 1.43–1.46 s | 2.3 GB |

Long clips, characters wrong against the text `say` read (punctuation, 畀/俾 and 噉/咁 not
counted): every fine-tune file got the 28 s clip exactly right (0 of 91). On the 56 s clip the
1.7B got 34–38 of 226 wrong and the 0.6B Q6_K 34; nearly all of them are the English names and
terms it has no dictionary for (Niamh, OraType, Grafana, Kubernetes) and 葉梓聰. The stock 0.6B
writes Simplified characters, so it scores 23 and 87 here.

- **The 0.6B Q6_K passes as many clips as the shipped 1.7B (34 of 41)** at about half the time,
  1 GB less memory and 40% of the download. It is one Cantonese clip behind (11 against 12) and
  its Cantonese error is higher (6.1% against 4.0%), in line with the model card's own numbers
  (MCE code-switching 6.98% against 5.62%, FLEURS 5.70% against 3.95%).
- **Q6_K is the 0.6B file to use:** Q4_K_M is only 5–8% faster and missed one more English clip;
  Q8_0 is slower and missed one more Cantonese clip.
- **The fine-tune is worth it over the stock 0.6B:** one more Cantonese clip, Hong Kong characters
  straight from the model (the stock writes Simplified, which OraType converts), and a lower
  Cantonese error, for one Mandarin clip less (it writes Mandarin in Traditional, below).

## What the 0.6B fine-tune does differently

- **No English clip written in Chinese.** In this run the shipped 1.7B wrote `sp-en-009`
  ("Please ask Niamh to update OraType before the Tauri release on Friday.") as
  請問需要在週五前更新oraType. On 2026-11-10 the same file wrote that clip in English, so the
  1.7B is not stable on it; the 0.6B kept it in English in all three files.
- **Dictionary words reach it better on English clips:** it wrote Niamh and OraType in
  `sp-en-009` and Tadhg in `sp-en-011`, which the 1.7B missed. It misses Kubernetes and Grafana
  in the Cantonese clip `sp-yue-011`, which the 1.7B got (Qubunet, provider).
- **Mandarin stays Mandarin but in Traditional characters.** Like the 1.7B fine-tune it writes
  Mandarin speech in Traditional characters (11 of 12 clips; the eval counts this as a warning,
  not a failure, because OraType does not convert text that is already Traditional). It put
  Cantonese words into one Mandarin clip (呢個 in `sp-zh-001`); the 1.7B did in two (`sp-zh-001`
  failed, 嘅 and 喇 in `sp-zh-012`).
- **No final 。 on Cantonese sentences** (0 of 14, the 1.7B 3 of 14), and fewer commas inside
  them. Polish adds the punctuation.

## Decision

The 0.6B fine-tune Q6_K is workable as a built-in model: on our clips it matches the 1.7B
overall, is twice as fast and needs about 1 GB less memory. It is a little less accurate on
Cantonese itself (one clip here, 1–2 points of character error on the model card's test sets),
so it does not replace the 1.7B for Macs that run the 1.7B well.

Where it fits is the small-Mac case: the 1.7B needs 8 GB of memory and shares it with the AI
polish model, while the 0.6B leaves another gigabyte free and halves the wait. Adding it means a
second Qwen3-ASR model in the built-in list (a new model id, its own download and update path,
and a suggestion rule by memory), which plan `hk-cantonese-asr` rejected for the stock model.
That needs its own plan before any code.

## Not measured

- Real voices and noise; the clips are synthetic. The model card's test sets (FLEURS, Common
  Voice, MDCC, LegCo, MCE) are real speech.
- An 8 GB Mac: the speed and memory numbers are from a 32 GB M1 Pro.
- The 1.7B fine-tune's version 2.1 (published 2026-11-13), which the model card says improves
  hotword context and short voice commands.

## Reproduce

```sh
python3 -I docs/benchmarks/reports/2026-11-10-hk-cantonese-asr/scripts/bench.py <label> <model.gguf> <mmproj.gguf> <out_dir> [--eval]
```

`bench.py` expects the long clips as `long/yue-a.wav` and `long/yue-b.wav` next to it (see that
report's Reproduce section).
