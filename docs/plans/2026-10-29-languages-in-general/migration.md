# Migration

How Your languages are stored and how existing configs get them. Back to [index](index.md).

## Config

```json
"your_languages": ["en", "zh-Hant-HK"],
"translation": {
  "targets": ["en", "ja"],
  "languages": { "zh-Hant-HK": { "library_preset": { "id": "cantonese-hong-kong", "…": "…" } } }
}
```

- `your_languages`: written-language codes, canonical (`zh` becomes `zh-Hans`), without repeats,
  at most 12. `null` until the user has chosen; the app then shows the Mac's languages.
- `spoken_languages` (plan `language-model-suggestions`) is read only for the migration and is
  no longer written.

## One-time migration

It runs when a stored config has no `your_languages` key. Until then dictation was routed by
Translate's list, so the result keeps those languages:

1. The old spoken languages, in order, as written codes:
   - `yue` → `zh-Hant-HK`;
   - `zh` → the first Mandarin code (`zh-Hans` or `zh-Hant-TW`) in Translate's list, else
     `zh-Hans`;
   - every other code as it is (unsupported codes are dropped).
2. Then every Translate language not already in the list.
3. Without repeats, at most 12.

When `spoken_languages` was never set, the result is Translate's languages; with no Translate
languages either, it stays unset. An explicit empty spoken list counts as set. The migrated
config is written back at once, so the migration runs once; later loads only normalize the list,
so a language the user removes stays removed. The log names the codes, never any text.

## Examples

| Spoken | Translate | Your languages |
|---|---|---|
| `en`, `yue` | `zh-Hant-HK`, `en` | `en`, `zh-Hant-HK` |
| `zh` | `ja`, `zh-Hant-TW` | `zh-Hant-TW`, `ja` |
| `zh`, `ja` | `en` | `zh-Hans`, `ja`, `en` |
| never set | `en`, `ja` | `en`, `ja` |
| never set | none | unset |
| empty | `ja` | `ja` |
