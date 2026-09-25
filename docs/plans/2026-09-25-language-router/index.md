# Language router

AI polish should know which language a dictation is in (English, Mandarin, Cantonese, later
others) before it polishes it, so it can pick a language-specific model or prompt. The
router decides this per transcript. This plan's first change is groundwork only: a
character-based router, and everything Laya needs to run inside Typelite (Laya is a small
"System 1" decision model, Apache-2.0). Routing polish by the answer and the Settings for it
come later in this plan.

Status: building — 2026-09-25

## Goals

- One decision per transcript: `English | Mandarin | Cantonese | Other`, with a confidence, and
  Simplified vs Traditional for Chinese text.
- No Python, no ONNX runtime, no second ggml linked into the app: Laya's backbone runs in the
  `llama-server` program Typelite already ships (plan
  [ai-polish-setup](../2026-09-25-ai-polish-setup/index.md)); its head and tokenizer run in plain
  Rust.
- A router that works with no model at all (the character router), so routing never depends on
  a download.
- Later in this plan: polish picks its model or prompt by the router's answer; Settings shows
  the choice.

## Non-goals

- Translating, or changing the speech step. Whisper keeps `-l auto`.
- Training or fine-tuning Laya inside Typelite.
- UI in this change. Nothing is wired into the pipeline yet.

## Key decisions

| Decision | Reason |
|---|---|
| Laya's backbone runs in a **second** `llama-server` (`--embeddings --pooling none`), owned by its own `ServerManager` | Same program, lifecycle, key and log handling as Built-in AI; the chat server's arguments and behaviour stay unchanged. A router-mode server with both models is possible later but mixes a chat and an embeddings-only setup. |
| The head is plain Rust on `safetensors` weights; matrix products use Accelerate's `cblas_sgemm` on macOS, portable loops elsewhere | No ML crates or runtimes; the head is 2 transformer layers and a scorer (14 M parameters). Accelerate makes it about 6 times faster (40 ms instead of 250 ms). |
| Laya's tokenizer is reimplemented in Rust from `tokenizer.json`, and ids (not text) go to llama-server | The GGUF's embedded tokenizer is wrong for this model (default BPE pre-tokenizer). The `tokenizers` crate would add a large dependency for one BPE. |
| Backbone file: Q8_0 GGUF (341 MB) | Probabilities within 0.03 of the original; Q4_K_M drifted by up to 0.17 (see [laya-head.md](laya-head.md)). |
| All three Laya files (backbone, head, tokenizer) are downloaded with the shared download and SHA-256 code, pinned to a commit | One tested path, like the speech and AI models. |
| The character router comes first; Laya is optional | Zero-shot, Laya answered 9 of 18 test sentences right and never picked Cantonese; the character router got 18 of 18. |

## Parts

| File | Covers |
|---|---|
| [laya-head.md](laya-head.md) | How Laya works (files, sequence, head maths), the Rust design, the llama.cpp check, and measured results. |

## Open questions

- **Is Laya worth shipping for language routing?** Zero-shot it cannot tell Cantonese from
  Mandarin and often calls English "Mandarin". Options: fine-tune Laya's head on dictated
  English / Mandarin / Cantonese text (the model card says fine-tuning is where its accuracy
  comes from), keep Laya for other decisions (is this a question for Ask anything, which app
  tone), or drop it and keep the character router.
- **Whisper writes Cantonese speech as standard Chinese** (CLAUDE.md). Such a transcript has no
  Cantonese characters, so no text router can see it is Cantonese. The speech step's language
  guess, or Whisper's Cantonese output setting, may be needed.
- **Fallback when Laya is not installed:** the character router. Whether it should also run
  when Laya is installed (for example Laya only when the character router is unsure) depends on
  the first question.
- **Code-switched text** ("我哋今個 project 嘅 deadline"): counted as Cantonese when it has a
  Cantonese character; which polish prompt suits it best is not tested.
- **Latin-script languages other than English:** the character router only spots them by
  accents; unaccented Spanish or Italian reads as English.
- **Lifecycle:** when the Laya server starts (with the app, or on first use) and when it stops;
  it uses about 0.5 GB of memory while running. Not wired yet (`laya_server().set_paths` and
  the exit handler).
- **Calibration:** Laya-multilingual ships temperature 1.0 (uncalibrated). Fit per-question
  temperatures on our own data before trusting its confidences.
