# Model catalogue and suggested setups

The built-in models this plan proposes, and the language combinations that have a suggested
setup. Part of plan [`language-model-suggestions`](index.md).

## Speech recognition

| Option | Model | Download | Runs in |
|---|---|---|---|
| Best accuracy | Whisper large-v3-turbo (q5_0) | 574 MB | Typelite (whisper.cpp) |
| Faster | Whisper small (q5_1) | 190 MB | Typelite (whisper.cpp) |
| Qwen3-ASR 1.7B | Qwen3-ASR 1.7B Q8_0 + mmproj | 2.5 GB | Built-in llama-server |

## AI polish

| Option | Model | Download | Memory |
|---|---|---|---|
| Best quality | Qwen3 4B Instruct 2507 (Q4_K_M) | 2.5 GB | 16 GB |
| Faster | Qwen3 1.7B (Q4_K_M) | 1.1 GB | 8 GB |
| Qwen 3.5 4B | Qwen 3.5 4B (Q4_K_M), thinking off (`--reasoning off`) | 2.7 GB | 16 GB |

## Suggested setups

A setup matches a combination of languages and names one speech and one AI model. When none
matches, the standard models (Best accuracy, Best quality) are used.

| Combination | Speech | AI polish | Evidence |
|---|---|---|---|
| Any combination with Cantonese | Qwen3-ASR 1.7B | Qwen 3.5 4B (Qwen3 1.7B under 16 GB) | Speech evals: Whisper writes Cantonese as written Chinese; the PC setup keeps spoken Cantonese |

Adding a row needs results from `evals/` for that combination. The table lives in code next to
the model lists, so a new setup is one entry plus its tests.
