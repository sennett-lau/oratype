# Languages in General

The languages a user speaks get one place: **Settings → General → Your languages**. Each row is
a language with its preset, as in the Translate list, and the list decides how dictation is
written: Dictate and Ask pick the language the user spoke from it and follow that language's
preset, whether speech and AI run built in, on the user's own server or through an API key.
Translate keeps its own list of up to three languages, used only by the Translate shortcut.
Until now the language picker sat on the built-in Speech and AI polish pages, so it looked
built-in only, and dictation was routed by the Translate list, so a Cantonese speaker had to
add Cantonese as a translation language to get its preset.

Status: building — 2026-10-29

Changes decisions of earlier plans (each has a "Superseded in part" note):

- [language-model-suggestions](../2026-10-16-language-model-suggestions/index.md): spoken
  languages become Your languages, stored as written-language codes, and the picker leaves the
  built-in pages for General.
- [language-prompt-library](../2026-09-26-language-prompt-library/index.md) and
  [language-routing-everywhere](../2026-10-02-language-routing-everywhere/index.md): the polish
  router picks from Your languages, not from the Translate languages.
- [settings-order](../2026-10-05-settings-order/index.md): General gains Your languages, and the
  Translate section's list is for the Translate shortcut only.

## Goals

- One list of the languages the user speaks and dictates in, in Settings → General, each with
  its preset and a way to change it.
- Dictate, Ask and the Qwen3-ASR script conversion use that list with every speech and AI
  service, built in or not.
- Translate's list (up to three) is used only by Translate: its targets, the pill and the
  Switch language key.
- Existing users keep the routing they had after the upgrade.

- An added language is recognised and uses its official preset with no manual step.

## Non-goals

- Downloading a preset when a language is added: the official ones are built in, and others are
  fetched only when the user browses or has automatic updates on (plan `language-prompt-library`).
- Changing the router's steps, the preset format or the Translate list.
- Regional variants (en-GB, pt-BR) or a free "any language" field.

## Key decisions

| Decision | Reason |
|---|---|
| Your languages is a Settings → General group, with the Translate pane's row, sheet and preset browser | The languages a user speaks are not a property of one engine; General is where every service can see them. |
| The router reads only Your languages | What someone dictates in and what they translate into are different lists; a translation target should not change dictation, and a spoken language should not need to be a translation target. |
| Your languages store written-language codes (`en`, `zh-Hant-HK`, `zh-Hans`, `zh-Hant-TW`…), the codes of presets and Translate | A preset is written for one written form; with the same code a language has one preset and one settings entry. Built-in model suggestions map the codes to spoken ones (`zh-Hant-HK` → `yue`). |
| One settings entry per language, shared by both lists (still stored under `translation.languages`) | Changing Cantonese's preset in General also changes how Translate writes Cantonese; no second copy can drift. The stored key stays for compatibility. |
| In Your languages the three Chinese codes are named by the spoken language: Cantonese (Hong Kong), Mandarin (Simplified), Mandarin (Traditional) | People say which language they speak; Translate keeps its written-form names. |
| Your languages saves at once (its own command), not through the Save bar | As before: onboarding, Home and Settings share one save path, and other windows get the change. |
| No drag order; the list keeps the order languages were added, at most 12 | Order only breaks a tie between two languages with the same number of hints; Translate's order exists for the Switch language key. |
| The built-in Speech and AI polish pages lose the picker; their "Suggested for your languages" banner links to General | Choosing languages there looked like a built-in setting. |
| Onboarding keeps its languages step, and Next saves the list it shows | The step fills in the Mac's languages; routing needs them saved. |
| A one-time migration builds Your languages from the old spoken languages and the Translate languages | Dictation was routed by the Translate list; keeping those languages keeps every user's routing. |
| Automatic preset updates cover Your languages and Translate's languages | A language used only for dictation can also follow new versions. |
| The official presets are built into the app (`include_str!`), checked with the download checks | A language can get its preset with no network request; the hash check is the same as for a download. |
| A chosen language without settings gets its official built-in preset once, with automatic updates off; app updates bring newer built-in versions | Found in a dev build: with no preset nothing was recognised, so every dictation got plain polish. No new network traffic the user did not choose; a language the user set up, or moved back to the built-in text, is left alone. |
| A language without a preset is heard by its own speech code: `zh-Hant-HK` → `yue`, other Chinese → `zh`, else its primary subtag | Every listed language can be recognised; Cantonese without its preset never takes Mandarin's `zh`. |
| The generic built-in text (translation wording) is not added to polish or Ask | It says how to translate, not how to write; the Chinese built-in texts, presets and edits are added. |

## Considered

- **Keep spoken codes (`yue`, `zh`) and map them when routing**: `zh` could be Simplified or
  Taiwan Mandarin, and presets and settings are per written form, so the list would still need
  a written code per language.
- **Route over Your languages and Translate's list together**: keeps today's surprise that a
  translation target changes dictation.
- **Separate per-list settings**: two presets for one language, which can disagree.

## Parts

| File | What it covers |
|---|---|
| [behaviour.md](behaviour.md) | Settings → General, Translate, the built-in pages, onboarding, Home and routing |
| [migration.md](migration.md) | The stored field, the one-time migration and its rules |

## Open questions

- Should Your languages be reorderable, if tie-breaking between hints ever matters in practice?
- Mandarin (Taiwan)'s preset needs a hint for `zh`, so with no other Mandarin listed, Taiwan
  Mandarin without one of its hint words gets no notes. Should a preset's `require_hint` apply
  only when another listed language also hears the code?

Answered while building: adding a language picks its official preset by itself, from the copy
built into the app (no download).
