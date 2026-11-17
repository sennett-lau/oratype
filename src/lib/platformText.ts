import { isWindowsPlatform } from '../stores/appStore'

export type TranslateFn = (key: string, options?: Record<string, unknown>) => string

/**
 * Translation keys whose macOS copy mentions Mac / macOS / Keychain / AppleScript and that have
 * an equivalent `*Windows` sibling key in every locale file.
 */
export const WINDOWS_TEXT_KEYS: ReadonlySet<string> = new Set([
  'settings.storedLocally',
  'settings.muteOutputHint',
  'settings.handsFree.enableHelp',
  'settings.handsFree.needsModel',
  'presetShare.keysFailed',
  'presetShare.errors.key_read_failed',
  'whatsNew.fastLanguageSwitching',
  'whatsNew.renamed',
  'whatsNew.handsFree',
  'whatsNew.fasterPaste',
  'whatsNew.webSearch',
  'whatsNew.builtin',
  'webSearch.keySaved',
  'webSearch.builtinDetail',
  'webSearch.setupTitle',
  'webSearch.setupBody',
  'webSearch.readyTitle',
  'webSearch.thisMacOnly',
  'languageSetup.fromMac',
  'languageSetup.homeBodyBoth',
  'languageSetup.homeBodySpeech',
  'languageSetup.homeBodyAi',
])

export function platformTextKey(key: string): string {
  return isWindowsPlatform() && WINDOWS_TEXT_KEYS.has(key) ? `${key}Windows` : key
}

export function tp(t: TranslateFn, key: string, options?: Record<string, unknown>): string {
  const resolved = platformTextKey(key)
  return options ? t(resolved, options) : t(resolved)
}
