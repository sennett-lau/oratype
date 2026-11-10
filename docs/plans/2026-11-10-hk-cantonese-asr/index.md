# Qwen3-ASR-1.7B-HK-Cantonese as the built-in speech model

The OraType team fine-tuned Qwen3-ASR 1.7B for Hong Kong Cantonese with English mixed in
([`joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF`](https://huggingface.co/joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF),
Apache-2.0). It replaces the stock Qwen3-ASR as OraType's built-in Qwen3-ASR model, which is the
speech model suggested for any language list with Cantonese, under the name
**Qwen3-ASR-1.7B-HK-Cantonese**.

Status: building - 2026-11-10.

## Goals and non-goals

- Cantonese written as Hongkongers write it, in Hong Kong characters, straight from the speech
  model, with English words kept.
- No loss on OraType's speech evaluation, in any language.
- Existing installs keep working without a download, and update in one click.
- Not a goal: a separate Mandarin or English model. Users who do not speak Cantonese are not
  suggested this model.

## Key decisions

- **Version 2, Q6_K, pinned at revision c2b90a4** with the fine-tune's own Q8_0 projector
  (1.77 GB in all). Reason: its Q4_K_M wrote English clips in Chinese, and its Q8_0 is slower
  for no gain ([report](../../benchmarks/reports/2026-11-10-hk-cantonese-asr/README.md)).
- **Same model id (`qwen3-asr-1.7b`), new files.** The stock files of OraType 1.2.1 (mradermacher
  Q4_K_M) and of 1.1.3 to 1.2.0 (ggml-org Q8_0) are the model's earlier versions, so presets on
  them keep working and Settings offers the new files as an update, as plan
  [`faster-cantonese-models`](../2026-10-20-faster-cantonese-models/index.md) set up. The row says
  "new version available" instead of "faster version available".
- **The suggestion and the guides name only this model** for Cantonese.
- **No second conversion of Traditional text.** A transcript that is already mostly Traditional
  is not converted again (the conversion turned 涌 into 湧).

## Considered

- Keeping the stock model and offering the fine-tune as a second choice: two Qwen3-ASR models in
  the picker for the same use, and the stock one writes Simplified Chinese that needs converting.
- Q8_0: the most faithful to the trained weights, but about 30% slower than the stock Q4_K_M
  and no better on our clips.

## Parts

| File | What it covers |
|---|---|
| [Report](../../benchmarks/reports/2026-11-10-hk-cantonese-asr/README.md) | Each file of versions 1 and 2 against the stock model: clips, speed, memory |

## Open questions

- Real voices: the clips are synthetic. If Cantonese users find the fine-tune worse than the
  stock model, the stock files are one revision pin away.
- Mandarin: the fine-tune once wrote Mandarin speech in Cantonese words. Users whose list has
  both Cantonese and Mandarin get this model; watch for reports.
