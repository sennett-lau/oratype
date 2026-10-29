import { TARGET_LANGUAGES, canonicalTranslationCode } from './constants'

/**
 * Plan `languages-in-general`: Your languages, the languages the user speaks and dictates in.
 * Codes are written languages, the same codes as the translation languages and the language
 * presets (`en`, `zh-Hant-HK`), so a language has one preset whichever list it is in. Dictate and
 * Ask pick from this list; built-in model suggestions read it as spoken languages (`spokenCodeFor`).
 */

/** Most languages Your languages keeps (`MAX_YOUR_LANGUAGES` in src-tauri/src/storage/mod.rs). */
export const MAX_YOUR_LANGUAGES = 12

/**
 * The three Chinese codes are named by the language spoken (Cantonese, Mandarin), not by the
 * script as in the Translate list; the others show their own name, as there.
 */
const NAME_KEYS: Record<string, string> = {
  'zh-Hans': 'languageSetup.names.zhHans',
  'zh-Hant-HK': 'languageSetup.names.zhHantHK',
  'zh-Hant-TW': 'languageSetup.names.zhHantTW',
}

type Translate = (key: string, values?: Record<string, unknown>) => string

/** The languages a user can add: every translation language, in the same order. */
export const YOUR_LANGUAGE_CODES: string[] = TARGET_LANGUAGES.map((language) => language.value)

/** A language's name in Your languages: "Cantonese (Hong Kong)", "English", "日本語". */
export function yourLanguageName(code: string, t: Translate): string {
  const key = NAME_KEYS[code]
  if (key) return t(key)
  return TARGET_LANGUAGES.find((language) => language.value === code)?.label ?? code
}

/** "English and Cantonese (Hong Kong)" or "English, 日本語 and …", in the interface language. */
export function yourLanguageList(codes: readonly string[], t: Translate): string {
  const names = codes.map((code) => yourLanguageName(code, t))
  if (names.length <= 1) return names.join('')
  return t('languageSetup.list', {
    items: names.slice(0, -1).join(t('languageSetup.listSeparator')),
    last: names[names.length - 1],
  })
}

/**
 * The written language a macOS language tag stands for: `zh-Hant-HK`, `zh-HK`, `zh-MO` and `yue`
 * are Cantonese (Hong Kong), other Traditional Chinese (`zh-Hant`, `zh-TW`) is Taiwan Mandarin,
 * any other Chinese is Simplified, the rest is their primary code. `null` for a language OraType
 * does not list.
 */
export function writtenCodeFor(tag: string): string | null {
  const parts = tag.trim().toLowerCase().replace(/_/g, '-').split('-')
  if (!parts[0]) return null
  if (parts[0] === 'yue') return 'zh-Hant-HK'
  if (parts[0] === 'zh') {
    if (parts.includes('hk') || parts.includes('mo')) return 'zh-Hant-HK'
    return parts.includes('hant') || parts.includes('tw') ? 'zh-Hant-TW' : 'zh-Hans'
  }
  return canonicalTranslationCode(parts[0])
}

/** Your languages from a list of macOS language tags, in order, without repeats. */
export function yourLanguagesFromTags(tags: readonly string[]): string[] {
  const codes: string[] = []
  for (const tag of tags) {
    const code = writtenCodeFor(tag)
    if (code && !codes.includes(code) && codes.length < MAX_YOUR_LANGUAGES) codes.push(code)
  }
  return codes
}

/** The Mac's preferred languages, as the web view reports them. */
export function macLanguageTags(): readonly string[] {
  if (typeof navigator === 'undefined') return []
  return navigator.languages?.length ? navigator.languages : [navigator.language].filter(Boolean)
}

/**
 * The user's languages: what they chose, or, until they have chosen, what the Mac's preferred
 * languages suggest.
 */
export function effectiveYourLanguages(
  chosen: string[] | null | undefined,
  tags: readonly string[] = macLanguageTags(),
): string[] {
  return chosen ?? yourLanguagesFromTags(tags)
}

/**
 * Plan `language-model-suggestions`: the spoken language a language code or macOS tag stands
 * for, as the suggested setups use them. `zh-Hant-HK`, `zh-HK`, `zh-MO` and `yue` are Cantonese
 * (`yue`), any other Chinese is Mandarin (`zh`), the rest is their primary code. `null` for a
 * language OraType does not list.
 */
export function spokenCodeFor(tag: string): string | null {
  const parts = tag.trim().toLowerCase().replace(/_/g, '-').split('-')
  if (!parts[0]) return null
  if (parts[0] === 'yue') return 'yue'
  if (parts[0] === 'zh') {
    return parts.includes('hk') || parts.includes('mo') ? 'yue' : 'zh'
  }
  return canonicalTranslationCode(parts[0]) ? parts[0] : null
}

/** The spoken languages of a list of codes, in order, without repeats. */
export function spokenCodesOf(codes: readonly string[]): string[] {
  const spoken: string[] = []
  for (const code of codes) {
    const language = spokenCodeFor(code)
    if (language && !spoken.includes(language)) spoken.push(language)
  }
  return spoken
}
