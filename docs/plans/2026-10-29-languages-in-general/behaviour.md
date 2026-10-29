# Behaviour

Where Your languages appear and what reads them. Back to [index](index.md).

## Settings → General → Your languages

- A group after the interface language, labelled **Your languages**, with an **About language
  presets** link (the same guide section as Translate's).
- One row per language, in the saved order: the name, the line saying where its instructions
  come from (preset and version, edited, built-in, or off), the on/off switch, **Edit** and ✕.
  These are the Translate rows without the drag handle and number. Any language can be removed;
  the list may be empty.
- **＋ Add language** ("up to 12") lists the supported languages not yet chosen. Adding one
  saves the list and, when it has no settings yet, gives it its official built-in preset (see
  "Built-in presets"), so the row shows the preset at once. No sheet opens; **Edit** changes it.
- The switch and the sheet save the language's settings at once, as in Translate. They are the
  same settings Translate uses for that language.
- A note under the rows: when you dictate or ask, OraType picks the language you spoke from this
  list and follows its preset, with built-in models, your own server or an API key; the preset
  also applies when translating into the language.
- While the user has never chosen (no saved list), the note offers to add the Mac's preferred
  languages with one button. An empty list shows only the Add row.

## Built-in presets

- The official presets of `presets/languages/` (with `index.json`) are compiled into the app and
  pass the same size, SHA-256 and validation checks as a download. A test fails when an official
  preset is missing from the built-in list.
- A chosen language (Your languages, then Translate's) without a settings entry gets the best
  direct official match, as Browse would list it first: `en` → English, `zh-Hant-HK` →
  Cantonese (Hong Kong), `zh-Hant-TW` → Mandarin (Taiwan); `zh-Hans` and `ja` have none today.
  The file is stored like a download and the language's `library_preset` points to it.
  Automatic updates stay off: OraType never goes online for a preset the user did not pick
  (only the app-update check runs on its own, per `CLAUDE.md`). At startup, an official preset
  older than the app's built-in copy moves to it, the way an automatic update would, and edited
  text is still never replaced.
- It runs when Your languages are saved (General, onboarding) and once at startup, before any
  window loads the config, which covers migrated lists and languages added earlier.
- A language with settings (another preset, its own text, switched off, its own hints) is never
  touched. The codes given a preset are recorded in the store (`bundled.json`), so a language the
  user moved back to the built-in text is not given it again. Logs name codes and ids only.

## Settings → Translate

Unchanged list and sheet. A note under the rows says only the Translate shortcut uses this list
and links to General → Your languages. The rows still show and edit each language's preset,
since it is the shared setting.

## Built-in Speech and AI polish pages

The Your languages picker is gone. The "Suggested for your languages" banner stays (when the
user's languages have a suggested setup with a model not in use) and gains **Change your
languages**, which opens Settings → General. The Suggested tag on model cards is unchanged.

## Onboarding

The languages step keeps its token picker, now over written codes with the names above. Until
the user changes it, it shows the Mac's preferred languages (`zh-HK` → Cantonese (Hong Kong),
`zh-Hant-TW` → Mandarin (Traditional), `zh-Hans-CN` → Mandarin (Simplified)). **Next** saves the
shown list when the user has not chosen one, so routing has it.

## Home

The "Suggested for your languages" card works as before; its names come from Your languages.

## Routing

```
transcript ─┬─> router over Your languages ──> polish notes (Dictate)
            │                               ──> answer language (Ask)
            │                               ──> Hong Kong / Taiwan characters (Qwen3-ASR)
Translate ──┴─> its target ─────────────────> translation instructions
```

- `language_profiles` resolves Your languages in list order; the router's steps are those of
  plans `language-prompt-library` and `language-routing-everywhere`.
- A language without a preset gets speech codes from its written code, with no hint needed:

  | Code | Speech codes |
  |---|---|
  | `zh-Hant-HK` | `yue` |
  | `zh-Hans`, `zh-Hant-TW` | `zh` |
  | any other (`en`, `ja`, `pt-BR`) | its primary subtag (`en`, `ja`, `pt`), when `language-codes.json` knows it |

  A preset's own codes, hints and `require_hint` take priority (Cantonese: `yue` and `zh`, a hint
  needed for `zh`), so Mandarin goes to Mandarin and never to Cantonese.
- Polish and Ask get the routed language's instructions, except the generic built-in text of a
  language without a preset or an edit, which is written for translation; Ask then keeps its own
  question-language detection.
- The AI request is the same for every AI preset: the routed language's instructions go into the
  system prompt, whether the preset is Built-in, a server or an API key.
- Translate uses its target's settings, as before; the Switch language key and the pill follow
  Translate's list.
- The daily preset update check looks at Your languages, then Translate's languages not already
  in it.
