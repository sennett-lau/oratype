# Built-in models suggested by language

Typelite asks which languages the user speaks and checks whether it has a suggested setup for
that combination. If it has, the speech and AI steps start with those built-in models chosen;
otherwise the standard ones are used. The built-in lists also grow beyond a larger and a smaller
model, so a Cantonese speaker can install Qwen3-ASR 1.7B and Qwen 3.5 4B on the Mac, the pair
that already works well on a llama.cpp server, without setting up a server.

Status: building - 2026-10-16.

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
  OpenAI-compatible speech client (with its `language X<asr_text>` handling) can use it. It is
  still the one "Built-in" speech preset, with Qwen3-ASR as its model; the provider picks
  whisper.cpp or the speech server by model. The server gets 12288 tokens of context (a ten-minute
  dictation is about 7,800 audio tokens at 13 per second, plus its transcript) and is never asked
  for `verbose_json`, which it refuses.
- **Spoken languages are their own setting** (`spoken_languages`), not the Translate list.
  Reason: Translate holds up to three languages the user writes into; spoken languages are what
  they say. Codes are spoken languages (`yue`, `zh`), not scripts. Until the user chooses, the
  Mac's preferred languages stand in (`zh-Hant-HK` counts as Cantonese). Changes save at once,
  not through the Settings Save bar.
- **The stock Qwen 3.5 4B, not the uncensored build the PC runs.** Reason: a setup suggested to
  everyone should be the publisher's model. The Cantonese guide still recommends the uncensored
  one, on a server, for slang and swear words.
- **Home leaves services on the user's own server alone.** Reason: someone who already runs
  Qwen3-ASR on a server needs no nudge to install it.
- **Home shows the suggestion until it is used or dismissed.** A "Suggested for your languages"
  group, styled like Finish setup, appears while a model from the matching setup is not in use
  (for example after skipping it in onboarding). "Set up" opens the Settings pane, as Finish
  setup does, so nothing downloads from Home; "Don't show again" hides it on Home for good, and
  the Settings tag stays. Reason: users who skipped onboarding still learn about the setup,
  without a nag they cannot turn off.
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
| [mock.html](mock.html) | UI mock: the whole onboarding flow, Settings and Home |
| [Report](../../benchmarks/reports/2026-10-16-language-model-suggestions/README.md) | Speech and polish evaluations behind the Cantonese setup |

## Open questions

- Which other combinations deserve a setup (Mandarin with English, for example)? Each needs eval
  results first.
- Real Cantonese recordings in `evals/speech`: the clips so far are synthetic.
- How Qwen3-ASR and the faster AI model share an 8 GB Mac (about 2.9 GB plus 1.5 GB).
- Should the languages step also fill the Translate languages later in the tour?

Answered while building: llama.cpp `b11177` loads Qwen3-ASR; on an M1 Pro a short clip takes
0.35–0.5 s and the server uses about 2.9 GB of memory; the stock Qwen 3.5 4B is suggested (see
Key decisions and the report).
