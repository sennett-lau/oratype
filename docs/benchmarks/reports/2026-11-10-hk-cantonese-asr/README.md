# Qwen3-ASR-1.7B-HK-Cantonese as the built-in speech model

OraType's built-in Qwen3-ASR (the model suggested for any language list with Cantonese) was the
stock Qwen3-ASR 1.7B, Q4_K_M. This report measures the OraType team's fine-tune for Hong Kong
Cantonese with English mixed in,
[`joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF`](https://huggingface.co/joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF)
(Apache-2.0), in each of its files, and picks the one OraType downloads (plan
[`hk-cantonese-asr`](../../../plans/2026-11-10-hk-cantonese-asr/index.md)).

## Provenance

- Measured 2026-11-10 (JST) on an Apple M1 Pro, 32 GB, macOS 26, with the bundled `llama-server`
  (llama.cpp `b11177`, Metal, every layer on the GPU) and the app's speech arguments:
  `--ctx-size 12288 --parallel 1 --mmproj …`.
- Files:
  - stock: `mradermacher/Qwen3-ASR-1.7B-GGUF` at 664b960, `Qwen3-ASR-1.7B.Q4_K_M.gguf` with its
    Q8_0 projector (OraType 1.2.1);
  - fine-tune version 1: revision 1afe540, Q4_K_M and Q8_0 (Q6_K made here from the Q8_0);
  - version 2: revision 74e1290, Q4_K_M and Q8_0 (Q6_K made here), then revision c2b90a4 with
    the published Q6_K. Every fine-tune file uses the fine-tune's own Q8_0 projector.
- Data: the 41 speech clips of `evals/` (15 English, 14 Cantonese, 12 Mandarin; macOS `say`,
  with dictionary words since PR #110), one sample each. Speed: two long Cantonese clips made the
  same way from [yue-a.txt](scripts/yue-a.txt) and [yue-b.txt](scripts/yue-b.txt) (`say -v
  Sinji`), median of 5 requests, each after a different clip so the server cannot reuse audio.
- Speed runs alternate the files (stock, fine-tune, stock, fine-tune), because the Mac's load
  changed between rounds by up to 30%. Harness: [scripts/bench.py](scripts/bench.py). Raw
  results: [raw/](raw/).

## Results

| File | Size (model + projector) | Clips passed (of 41) | Cantonese | English written in Chinese | Cantonese ending in 。 |
|---|---:|---:|---:|---:|---:|
| Stock Qwen3-ASR 1.7B Q4_K_M (1.2.1) | 1.64 GB | 34 | 10 / 14 | 0 | 14 / 14 |
| Fine-tune v1, Q4_K_M | 1.46 GB | 31 | 11 / 14 | 2 | – |
| Fine-tune v1, Q6_K (made here) | 1.77 GB | 33 | 10 / 14 | 0 | – |
| Fine-tune v1, Q8_0 | 2.19 GB | 32 | 10 / 14 | 0 | – |
| Fine-tune v2, Q4_K_M | 1.46 GB | 33 | 12 / 14 | 2 | – |
| Fine-tune v2, Q6_K (made here) | 1.77 GB | 34 | 12 / 14 | 0 | 4 / 14 |
| Fine-tune v2, Q8_0 | 2.19 GB | 32 | 10 / 14 | 0 | 3 / 14 |
| **Fine-tune v2, Q6_K (published)** | **1.77 GB** | **34** | **11 / 14** | **0** | **4 / 14** |

Speed, alternated runs (round 3):

| File | 28 s clip | 56 s clip | Resident memory |
|---|---:|---:|---:|
| Stock Q4_K_M | 1.24–1.26 s | 2.90–2.91 s | 3.2 GB |
| Fine-tune v2 Q6_K (published) | 1.53–1.63 s (+23–30%) | 3.57–3.62 s (+23–25%) | 3.4–3.5 GB |

In round 2, the same alternation gave Q4_K_M about the stock speed, Q6_K +8–18% and Q8_0 about
+30%.

- **Q4_K_M is rejected:** in both versions it wrote two English clips in Chinese.
- **Q8_0 is rejected:** slower than Q6_K and no clip better.
- **Q6_K is chosen:** as many clips as the stock model, one more Cantonese clip right, Hong Kong
  Traditional characters straight from the model, and English words kept, at about a quarter
  more time.

## What the fine-tune does differently

- It writes Hong Kong Traditional characters itself, so OraType no longer has to convert a
  Cantonese transcript. A transcript that is already mostly Traditional is no longer converted
  again: the conversion turned 涌 (鰂魚涌) into 湧.
- It usually leaves out the final 。 of a Cantonese sentence (4 of 14). Polish adds the
  punctuation, and a one-sentence dictation has no final period anyway (polish rule 7).
- On one Mandarin clip it wrote Cantonese words: 請喺呢度簽名或者輸入你嘅名字 for
  请在这里签名或者输入你的名字. The model was trained and evaluated on Cantonese and
  Cantonese–English speech only.
- The remaining failures are the same names and terms the stock model misses (Niamh, OraType,
  葉梓聰, Grafana, Kubernetes).

## Reproduce

```sh
python3 -I docs/benchmarks/reports/2026-11-10-hk-cantonese-asr/scripts/bench.py <label> <model.gguf> <mmproj.gguf> <out_dir> [--eval]
```

`bench.py` expects the long clips as `long/yue-a.wav` and `long/yue-b.wav` next to it
(`say -v Sinji -f yue-a.txt -o yue-a.aiff`, then `afconvert -f WAVE -d LEI16@16000 -c 1`).
