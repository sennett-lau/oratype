# Built-in models suggested by language

Typelite asks which languages the user speaks and checks whether it has a suggested setup for
that combination. If it has, the speech and AI steps start with those built-in models chosen;
otherwise the standard ones are used. The built-in lists also grow beyond a larger and a smaller
model, so a Cantonese speaker can install Qwen3-ASR 1.7B and Qwen 3.5 4B on the Mac, the pair
that already works well on a llama.cpp server, without setting up a server.

Status: draft - 2026-10-16.

## Goals and non-goals

- Offer more built-in models per step than "Best" and "Faster".
- A "Your languages" onboarding step (also in Settings) that looks up a suggested setup for the
  chosen language combination.
- Run Qwen3-ASR built in, so spoken Cantonese no longer needs a self-hosted server.
- Not a goal: rating every model for every language. Typelite cannot benchmark every language,
  so it shows no per-language quality labels.
- Not a goal: a new runtime, cloud models, or a different default for combinations without a
  suggested setup.

## Key decisions

- **Suggestions are per language combination, not per language.** A short table lists the
  combinations Typelite has a setup for; each entry needs eval results behind it. Reason: we can
  test a few combinations well, not every language.
- **No quality labels.** Model lists show name and size only; the one tag is "Suggested for
  your languages", on the model from a matching setup. Reason: labels would claim results we
  have not measured.
- **The languages step only preselects.** Speech recognition and AI polish keep their Built-in
  card and per-step download; the suggested model is chosen and tagged there, and the user can
  pick another. Reason: one way to install models, and the user confirms each 2–3 GB download.
- **First setup: any combination with Cantonese** gets Qwen3-ASR 1.7B and Qwen 3.5 4B. Reason:
  Whisper writes Cantonese speech as standard written Chinese (about 44 % character error on the
  eval clips), which polish cannot undo.
- **Qwen3-ASR runs in a second built-in llama-server** on its own port. Reason: llama-server
  already ships with Typelite and serves `/v1/audio/transcriptions`, so the existing
  OpenAI-compatible speech client (with its `language X<asr_text>` handling) can use it.
- **Memory rules stay.** The 4B AI models need 16 GB; below that the setup falls back to
  Qwen3 1.7B and says so. Reason: same limits as today.

## Onboarding flow

```
Welcome → Voice input → Your languages (new) → Speech recognition → AI polish → Web search → shortcut tour
                              │ suggested setup?        ▲ chosen + tagged      ▲ chosen + tagged
                              └─────────────────────────┴──────────────────────┘
```

## Parts

| File | What it covers |
|---|---|
| [catalogue.md](catalogue.md) | The proposed models, sizes and suggested setups |
| [mock.html](mock.html) | UI mock: the whole onboarding flow and the Settings model lists |

## Open questions

- Does llama.cpp `b11177` (the built-in server's tag) load Qwen3-ASR, or does the tag need to move
  to `b11205` or later, as on the PC?
- Official Qwen 3.5 4B or the abliterated build the PC uses? The official weights are the safer
  default to suggest; the evals should compare them on Cantonese polish.
- Speed and memory of Qwen3-ASR on Apple Silicon with both servers loaded, measured with
  `scripts/benchmark.mjs`.
- Which other combinations deserve a setup (Mandarin with English, for example)? Each needs eval
  results first.
- Should the languages step also fill the Translate languages later in the tour?
