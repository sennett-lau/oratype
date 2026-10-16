# Built-in models suggested by language

Typelite asks which languages the user speaks and suggests built-in speech and AI models that
handle them, instead of offering only a larger and a smaller model per step. A Cantonese speaker
gets spoken Cantonese on the Mac with one download: Qwen3-ASR 1.7B for speech and Qwen 3.5 4B
for polish, the pair that already works well on a llama.cpp server. No server setup is needed.

Status: draft - 2026-10-16.

## Goals and non-goals

- Offer more built-in models per step, each labelled with how well it handles each language.
- Ask for the user's languages once (onboarding, changeable in Settings) and suggest one speech
  and one AI model from them and from the Mac's memory and free disk space.
- Run Qwen3-ASR built in, so spoken Cantonese no longer needs a self-hosted server.
- Not a goal: a new runtime, cloud models, or changing the default for users who speak only
  languages Whisper already handles well.

## Key decisions

- **Qwen3-ASR runs in a second built-in llama-server**, on its own port, next to the polish
  server. Reason: llama-server already ships with Typelite and serves Qwen3-ASR through
  `/v1/audio/transcriptions`, so the existing OpenAI-compatible speech client (with its
  `language X<asr_text>` handling) can talk to it.
- **Suggestions come from a static table** of model × language fit, kept in code and checked by
  the language evals. Reason: predictable, offline, and testable; the evals give each label its
  evidence.
- **Cantonese drives the first rule.** If the user speaks Cantonese, suggest Qwen3-ASR and Qwen
  3.5 4B. Reason: Whisper writes Cantonese speech as standard written Chinese (about 44 %
  character error on the eval clips), which polish cannot undo.
- **Memory rules stay as they are.** The larger AI models need 16 GB; below that, the faster
  model is suggested. Reason: same limits the current setup uses.
- **The user confirms every download.** Suggestions never download on their own. Reason:
  downloads are 2–3 GB each.

## Parts

| File | What it covers |
|---|---|
| [catalogue.md](catalogue.md) | The proposed models, sizes, language fit and the suggestion rules |
| [mock.html](mock.html) | UI mock: onboarding languages step and the Settings model lists |

## Open questions

- Does llama.cpp `b11177` (the built-in server's tag) load Qwen3-ASR, or does the tag need to move
  to `b11205` or later, as on the PC?
- Official Qwen 3.5 4B or the abliterated build the PC uses? The official weights are the safer
  default to suggest; the evals should compare them on Cantonese polish.
- Speed and memory of Qwen3-ASR on Apple Silicon with both servers loaded, measured with
  `scripts/benchmark.mjs`.
- Is Qwen3-ASR better than Whisper for Mandarin and mixed Mandarin–English too? The speech evals
  decide whether the rule widens beyond Cantonese.
- Where "languages you speak" lives: a new setting, or derived from the spoken-language and
  translation settings that exist today.
