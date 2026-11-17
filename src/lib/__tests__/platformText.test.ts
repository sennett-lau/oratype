import { afterEach, describe, expect, it, vi } from 'vitest'
import { UI_LANGUAGES } from '../../i18n/languages'
import { WINDOWS_TEXT_KEYS, platformTextKey, tp } from '../platformText'

const FORBIDDEN_WINDOWS_TERMS = [
  /\bmac\b/i,
  /macos/i,
  /apple\s*silicon/i,
  /applescript/i,
  /keychain/i,
  /鑰匙串/,
  /鑰匙圈/,
  /钥匙串/,
  /キーチェーン/,
  /schlüsselbund/i,
  /trousseau/i,
  /llavero/i,
]

function readPath(obj: unknown, dotPath: string): unknown {
  return dotPath
    .split('.')
    .reduce<unknown>(
      (acc, part) =>
        acc && typeof acc === 'object' ? (acc as Record<string, unknown>)[part] : undefined,
      obj,
    )
}

describe('platformText & Windows copy across all 7 locales', () => {
  const originalPlatformDescriptor = Object.getOwnPropertyDescriptor(window.navigator, 'platform')

  function setPlatform(value: string) {
    Object.defineProperty(window.navigator, 'platform', {
      value,
      configurable: true,
    })
  }

  afterEach(() => {
    if (originalPlatformDescriptor) {
      Object.defineProperty(window.navigator, 'platform', originalPlatformDescriptor)
    }
  })

  it('selects *Windows keys on Win32 and base keys elsewhere without altering unmapped keys', () => {
    setPlatform('MacIntel')
    expect(platformTextKey('webSearch.setupTitle')).toBe('webSearch.setupTitle')
    expect(platformTextKey('settings.generalPane.output')).toBe('settings.generalPane.output')

    setPlatform('Win32')
    expect(platformTextKey('webSearch.setupTitle')).toBe('webSearch.setupTitleWindows')
    expect(platformTextKey('settings.generalPane.output')).toBe('settings.generalPane.output')

    const t = vi.fn((k: string, opts?: Record<string, unknown>) =>
      opts ? `${k}:${JSON.stringify(opts)}` : k,
    )
    expect(tp(t, 'settings.handsFree.needsModel', { name: 'Sam', size: 60 })).toBe(
      'settings.handsFree.needsModelWindows:{"name":"Sam","size":60}',
    )
    expect(tp(t, 'webSearch.thisMacOnly')).toBe('webSearch.thisMacOnlyWindows')
  })

  it('provides non-empty Windows translations without Mac/Keychain wording in all 7 locales', () => {
    for (const { value: lang, messages: json } of UI_LANGUAGES) {
      for (const baseKey of WINDOWS_TEXT_KEYS) {
        const winKey = `${baseKey}Windows`
        const value = readPath(json, winKey)
        expect(typeof value, `${lang}:${winKey} must be a string`).toBe('string')
        expect(
          (value as string).trim().length,
          `${lang}:${winKey} must be non-empty`,
        ).toBeGreaterThan(0)
        for (const forbidden of FORBIDDEN_WINDOWS_TERMS) {
          expect(value, `${lang}:${winKey} leaked ${forbidden}`).not.toMatch(forbidden)
        }
      }
    }
  })
})
