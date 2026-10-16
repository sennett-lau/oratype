# Model catalogue and suggestion rules

The built-in models this plan proposes, and how Typelite picks one per step. Part of plan
[`language-model-suggestions`](index.md).

## Speech recognition

| Model | Download | Runs in | Strong at | Weak at |
|---|---|---|---|---|
| Whisper large-v3 turbo (q5_0) | 0.57 GB | Typelite (whisper.cpp) | English, Mandarin, Japanese, European languages | Cantonese: written Chinese |
| Whisper small (q5_1) | 0.19 GB | Typelite (whisper.cpp) | Clear speech on small Macs | Cantonese: written Chinese |
| Qwen3-ASR 1.7B (Q8_0 + mmproj) | 2.5 GB | Built-in llama-server | Spoken Cantonese, mixed Chinese and English | To be measured |

## AI polish

| Model | Download | Memory | Note |
|---|---|---|---|
| Qwen3 4B Instruct 2507 (Q4_K_M) | 2.5 GB | 16 GB | Today's best-quality model |
| Qwen 3.5 4B (Q4_K_M) | 2.7 GB | 16 GB | Pairs with Qwen3-ASR for Cantonese; needs thinking off (`--reasoning off`) |
| Qwen3 1.7B (Q4_K_M) | 1.1 GB | 8 GB | Today's faster model |

## Fit labels

Each model has a label per language: Recommended, Good, Fair, or a named problem such as
"Written Chinese". A label other than Good needs a result in `evals/` to back it. Today only
Whisper on Cantonese is measured.

## Suggestion rules

1. Speech: Qwen3-ASR if the user speaks Cantonese; otherwise Whisper large-v3 turbo, or
   Whisper small when the Mac cannot run the larger model.
2. AI polish: Qwen3 1.7B when the Mac has under 16 GB; otherwise Qwen 3.5 4B if the user speaks
   Cantonese, else Qwen3 4B Instruct.
3. A suggestion that is already installed and in use shows no prompt. Settings shows a banner
   only when the suggested model differs from the one in use, and "Not now" hides it.
