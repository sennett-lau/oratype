import { describe, expect, it } from 'vitest'
import {
  FASTER_AI_MODEL,
  QWEN35_AI_MODEL,
  QWEN3_ASR_MODEL,
  missingModels,
  setupFor,
  suggestionFor,
} from '../languageSetups'
import {
  MAX_YOUR_LANGUAGES,
  YOUR_LANGUAGE_CODES,
  effectiveYourLanguages,
  spokenCodeFor,
  spokenCodesOf,
  writtenCodeFor,
  yourLanguageList,
  yourLanguageName,
  yourLanguagesFromTags,
} from '../yourLanguages'
import { TARGET_LANGUAGES } from '../constants'
import { translate } from '../../test-utils/i18nMock'
import { hardwareCheck } from '../../test-utils/speechHardware'

const speechAll = hardwareCheck(['large-v3-turbo', 'small', QWEN3_ASR_MODEL])
const aiAll = hardwareCheck(['qwen3-4b', 'qwen3-1.7b', QWEN35_AI_MODEL])
const builtin = (model: string | null) => ({ builtin: true, model })
const ownServer = { builtin: false, model: null }

describe('Your languages (plan `languages-in-general`)', () => {
  it('reads the Mac languages as written languages', () => {
    expect(writtenCodeFor('zh-Hant-HK')).toBe('zh-Hant-HK')
    expect(writtenCodeFor('zh-HK')).toBe('zh-Hant-HK')
    expect(writtenCodeFor('zh_Hant_MO')).toBe('zh-Hant-HK')
    expect(writtenCodeFor('yue-Hant')).toBe('zh-Hant-HK')
    expect(writtenCodeFor('zh-Hant-TW')).toBe('zh-Hant-TW')
    expect(writtenCodeFor('zh-TW')).toBe('zh-Hant-TW')
    expect(writtenCodeFor('zh-Hans-CN')).toBe('zh-Hans')
    expect(writtenCodeFor('zh')).toBe('zh-Hans')
    expect(writtenCodeFor('en-GB')).toBe('en')
    expect(writtenCodeFor('ja-JP')).toBe('ja')
    expect(writtenCodeFor('xx')).toBeNull()
    expect(writtenCodeFor('')).toBeNull()
    expect(yourLanguagesFromTags(['en-HK', 'zh-Hant-HK', 'en-US', 'zh-Hant-TW'])).toEqual([
      'en',
      'zh-Hant-HK',
      'zh-Hant-TW',
    ])
  })

  it('uses the chosen list, even an empty one, over the Mac languages', () => {
    expect(effectiveYourLanguages(null, ['en', 'zh-HK'])).toEqual(['en', 'zh-Hant-HK'])
    expect(effectiveYourLanguages(['ja'], ['en'])).toEqual(['ja'])
    expect(effectiveYourLanguages([], ['en'])).toEqual([])
  })

  it('offers every translation language, the Chinese ones named by the language spoken', () => {
    expect(YOUR_LANGUAGE_CODES).toEqual(TARGET_LANGUAGES.map((language) => language.value))
    expect(MAX_YOUR_LANGUAGES).toBe(12)
    expect(yourLanguageName('zh-Hant-HK', translate)).toBe('Cantonese (Hong Kong)')
    expect(yourLanguageName('zh-Hans', translate)).toBe('Mandarin (Simplified)')
    expect(yourLanguageName('zh-Hant-TW', translate)).toBe('Mandarin (Traditional)')
    expect(yourLanguageName('ja', translate)).toBe('日本語')
    expect(yourLanguageList(['en', 'ja', 'zh-Hant-HK'], translate)).toBe(
      'English, 日本語 and Cantonese (Hong Kong)',
    )
  })

  it('reads written languages as spoken ones for the suggested setups', () => {
    expect(spokenCodeFor('zh-Hant-HK')).toBe('yue')
    expect(spokenCodeFor('zh-HK')).toBe('yue')
    expect(spokenCodeFor('yue-Hant')).toBe('yue')
    expect(spokenCodeFor('zh-Hant-TW')).toBe('zh')
    expect(spokenCodeFor('zh-Hans')).toBe('zh')
    expect(spokenCodeFor('en-GB')).toBe('en')
    expect(spokenCodeFor('xx')).toBeNull()
    expect(spokenCodesOf(['en', 'zh-Hant-HK', 'zh-Hans', 'zh-Hant-TW'])).toEqual([
      'en',
      'yue',
      'zh',
    ])
    expect(setupFor(spokenCodesOf(['en', 'zh-Hant-HK']))?.id).toBe('cantonese')
    expect(setupFor(spokenCodesOf(['zh-Hans', 'zh-Hant-TW']))).toBeNull()
  })
})

describe('suggested setups (plan `language-model-suggestions`)', () => {
  it('has a setup only for combinations with Cantonese', () => {
    expect(setupFor(['en', 'yue'])?.id).toBe('cantonese')
    expect(setupFor(['yue'])?.speech).toBe(QWEN3_ASR_MODEL)
    expect(setupFor(['en', 'ja'])).toBeNull()
    expect(setupFor([])).toBeNull()
  })

  it('picks the setup models when this Mac is offered them', () => {
    expect(suggestionFor(['en', 'yue'], speechAll, aiAll)).toMatchObject({
      speech: QWEN3_ASR_MODEL,
      ai: QWEN35_AI_MODEL,
      aiFallback: false,
    })
    // Before the checks come back, the setup's own models are assumed.
    expect(suggestionFor(['yue'], null, null)).toMatchObject({
      speech: QWEN3_ASR_MODEL,
      ai: QWEN35_AI_MODEL,
    })
  })

  it('falls back to the faster AI model on a Mac with too little memory', () => {
    const small = hardwareCheck(['qwen3-1.7b'])
    expect(suggestionFor(['yue'], speechAll, small)).toMatchObject({
      ai: FASTER_AI_MODEL,
      aiFallback: true,
    })
    expect(suggestionFor(['yue'], hardwareCheck(['small']), hardwareCheck([]))).toMatchObject({
      speech: null,
      ai: null,
    })
  })

  it('offers on Home only the built-in models not in use yet', () => {
    const suggestion = suggestionFor(['en', 'yue'], speechAll, aiAll)
    expect(
      missingModels(suggestion, { speech: builtin('large-v3-turbo'), ai: builtin(null) }),
    ).toEqual([
      { kind: 'speech', id: QWEN3_ASR_MODEL },
      { kind: 'ai', id: QWEN35_AI_MODEL },
    ])
    expect(
      missingModels(suggestion, { speech: builtin(QWEN3_ASR_MODEL), ai: builtin('qwen3-4b') }),
    ).toEqual([{ kind: 'ai', id: QWEN35_AI_MODEL }])
    expect(
      missingModels(suggestion, {
        speech: builtin(QWEN3_ASR_MODEL),
        ai: builtin(QWEN35_AI_MODEL),
      }),
    ).toEqual([])
    // A service on the user's own server is left alone.
    expect(missingModels(suggestion, { speech: ownServer, ai: ownServer })).toEqual([])
    // A fallback AI model is never pushed.
    const fallback = suggestionFor(['yue'], speechAll, hardwareCheck(['qwen3-1.7b']))
    expect(
      missingModels(fallback, { speech: builtin(QWEN3_ASR_MODEL), ai: builtin(null) }),
    ).toEqual([])
    expect(missingModels(null, { speech: builtin(null), ai: builtin(null) })).toEqual([])
  })
})
