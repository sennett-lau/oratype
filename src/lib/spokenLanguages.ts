import { TARGET_LANGUAGES } from './constants'

/**
 * Plan `language-model-suggestions`: the languages a user can say they speak. Codes are spoken
 * languages, not scripts: Cantonese (`yue`) and Mandarin (`zh`) are one entry each, whatever
 * characters the user writes. Names are the languages' own, as in the Translate list; the two
 * Chinese entries also get a translated hint (`hintKey`), since their own names alone do not tell
 * every reader which is which.
 */
export interface SpokenLanguage {
  code: string
  name: string
  hintKey?: string
}

export const SPOKEN_LANGUAGES: SpokenLanguage[] = [
  { code: 'en', name: 'English' },
  { code: 'yue', name: '廣東話', hintKey: 'languageSetup.hints.yue' },
  { code: 'zh', name: '普通话 / 國語', hintKey: 'languageSetup.hints.zh' },
  ...TARGET_LANGUAGES.filter(
    (language) => language.value !== 'en' && !language.value.startsWith('zh'),
  ).map((language) => ({ code: language.value, name: language.label })),
]

export function spokenLanguage(code: string): SpokenLanguage | undefined {
  return SPOKEN_LANGUAGES.find((language) => language.code === code)
}

/**
 * The spoken language a macOS language tag stands for: `zh-Hant-HK`, `zh-HK`, `zh-MO` and `yue`
 * are Cantonese, any other Chinese is Mandarin, the rest is their primary code. `null` for a
 * language OraType does not list.
 */
export function spokenCodeFor(tag: string): string | null {
  const lower = tag.trim().toLowerCase().replace(/_/g, '-')
  if (!lower) return null
  const parts = lower.split('-')
  if (parts[0] === 'yue') return 'yue'
  if (parts[0] === 'zh') {
    return parts.includes('hk') || parts.includes('mo') ? 'yue' : 'zh'
  }
  return spokenLanguage(parts[0]) ? parts[0] : null
}

/** The spoken languages a list of macOS language tags gives, in order, without repeats. */
export function spokenLanguagesFromTags(tags: readonly string[]): string[] {
  const codes: string[] = []
  for (const tag of tags) {
    const code = spokenCodeFor(tag)
    if (code && !codes.includes(code)) codes.push(code)
  }
  return codes
}

/** The Mac's preferred languages, as the web view reports them. */
export function macLanguageTags(): readonly string[] {
  if (typeof navigator === 'undefined') return []
  return navigator.languages?.length ? navigator.languages : [navigator.language].filter(Boolean)
}

/**
 * The user's spoken languages: what they chose, or, until they have chosen, what the Mac's
 * preferred languages suggest.
 */
export function effectiveSpokenLanguages(
  chosen: string[] | null | undefined,
  tags: readonly string[] = macLanguageTags(),
): string[] {
  return chosen ?? spokenLanguagesFromTags(tags)
}

type Translate = (key: string, values?: Record<string, unknown>) => string

/** A spoken language's name for sentences: its translated hint when it has one, else its own. */
export function spokenLanguageName(code: string, t: Translate): string {
  const language = spokenLanguage(code)
  if (!language) return code
  return language.hintKey ? t(language.hintKey) : language.name
}

/** "English and Cantonese" or "English, Japanese and Cantonese", in the interface language. */
export function spokenLanguageList(codes: readonly string[], t: Translate): string {
  const names = codes.map((code) => spokenLanguageName(code, t))
  if (names.length <= 1) return names.join('')
  return t('languageSetup.list', {
    items: names.slice(0, -1).join(t('languageSetup.listSeparator')),
    last: names[names.length - 1],
  })
}
