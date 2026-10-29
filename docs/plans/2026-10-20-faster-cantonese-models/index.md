# Faster built-in models for Cantonese

The built-in setup suggested for Cantonese (plan
[`language-model-suggestions`](../2026-10-16-language-model-suggestions/index.md)) felt slow on a
Mac. The same two models in smaller files are faster at the same quality on our evaluation, so
OraType now downloads those: Qwen3-ASR 1.7B as Q4_K_M with its Q8_0 audio projector, and Qwen 3.5
4B as Q4_0. Macs that already have the earlier files keep working with them, and Settings offers
the faster files as an update.

Status: building - 2026-10-20.

## Goals and non-goals

- Faster speech and polish for the Cantonese setup, with no measured loss of quality.
- Smaller downloads: 1.6 GB instead of 2.5 GB for speech, 2.6 GB instead of 2.7 GB for polish.
- Existing installs keep working without a download, and can update in one click.
- Not a goal: the polish prompt cache (a server slot per language). It is the larger win for
  bilingual users but is a separate change to the built-in AI server.
- Not a goal: other models. Qwen3 4B Instruct and Qwen3 1.7B stay Q4_K_M; they were not measured.

## Key decisions

- **Speech: Qwen3-ASR 1.7B Q4_K_M with its own Q8_0 projector, from
  `mradermacher/Qwen3-ASR-1.7B-GGUF` at revision 664b960.** Reason: 21–28% faster than Q8_0 on
  28 s and 56 s Cantonese clips, 0.9 GB smaller, the same eval results, and the same transcripts
  but one punctuation mark. ggml-org publishes only Q8_0 and bf16, and this published file
  measured the same as one we quantised ourselves, so there is no copy of ours to host.
- **Polish: Qwen 3.5 4B Q4_0 from `unsloth/Qwen3.5-4B-GGUF`, same revision as before.** Reason:
  36% faster writing and 22% faster prompt reading than Q4_K_M on Apple GPUs (llama.cpp's Metal
  kernels suit Q4_0 best), 160 MB smaller, and pass rates at least as high.
- **Files stay pinned** by revision URL, size and SHA-256, as every built-in model is. Reason:
  the download is checked the same way whoever publishes it.
- **The earlier files are kept as a model's earlier version** (`previous` in each catalog
  entry). They count as installed, so `reconcile_builtin_models` does not move or clear a preset
  that uses them, and a Qwen3-ASR preset on the earlier model file runs with the earlier
  projector. Reason: an update must never leave a working setup without a model.
- **Update is the normal setup of the model.** Settings shows "faster version available" and an
  Update button on the model's row; it downloads the current files and tests them. Only after the
  test passes are the earlier files deleted and the preset moved; if it fails, the presets go back
  to the earlier files. Delete removes both versions. Reason: one download path, and the old
  files are never removed before the new ones are known to work.
- **When both versions are on disk, the current one is listed**, and the earlier one still keeps
  any preset that uses it. Reason: an update whose test failed must not switch the preset at the
  next start.

## Considered

- Hosting our own Q4_K_M made from ggml-org's bf16: works, but means a model repository to
  maintain when a published file measures the same.
- IQ4_XS or Q3_K_M for polish: smaller but slower than Q4_0 on Metal; Q3_K_M loses Mandarin.
- Q6_K for speech: the cautious middle (12% faster on the long clip); kept as the fallback if
  real voices show the 4-bit file slipping.

## Parts

| File | What it covers |
|---|---|
| [Report](../../benchmarks/reports/2026-10-20-cantonese-model-sizes/README.md) | Speed and quality of each file on an M1 Pro, the mradermacher addendum, and where a dictation's time goes |

## Open questions

- Real Cantonese recordings and noisy rooms: all speech in the evaluation is synthetic.
- 8 and 16 GB Macs, and newer chips, were not measured.
