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
  SPOKEN_LANGUAGES,
  effectiveSpokenLanguages,
  spokenCodeFor,
  spokenLanguagesFromTags,
} from '../spokenLanguages'
import { hardwareCheck } from '../../test-utils/speechHardware'

const speechAll = hardwareCheck(['large-v3-turbo', 'small', QWEN3_ASR_MODEL])
const aiAll = hardwareCheck(['qwen3-4b', 'qwen3-1.7b', QWEN35_AI_MODEL])
const builtin = (model: string | null) => ({ builtin: true, model })
const ownServer = { builtin: false, model: null }

describe('spoken languages (plan `language-model-suggestions`)', () => {
  it('reads the Mac languages as spoken languages', () => {
    expect(spokenCodeFor('zh-Hant-HK')).toBe('yue')
    expect(spokenCodeFor('zh-HK')).toBe('yue')
    expect(spokenCodeFor('zh_Hant_MO')).toBe('yue')
    expect(spokenCodeFor('yue-Hant')).toBe('yue')
    expect(spokenCodeFor('zh-Hant-TW')).toBe('zh')
    expect(spokenCodeFor('zh-Hans-CN')).toBe('zh')
    expect(spokenCodeFor('en-GB')).toBe('en')
    expect(spokenCodeFor('ja-JP')).toBe('ja')
    expect(spokenCodeFor('xx')).toBeNull()
    expect(spokenLanguagesFromTags(['en-HK', 'zh-Hant-HK', 'en-US', 'zh-Hant-TW'])).toEqual([
      'en',
      'yue',
      'zh',
    ])
  })

  it('uses the chosen list, even an empty one, over the Mac languages', () => {
    expect(effectiveSpokenLanguages(null, ['en', 'zh-HK'])).toEqual(['en', 'yue'])
    expect(effectiveSpokenLanguages(['ja'], ['en'])).toEqual(['ja'])
    expect(effectiveSpokenLanguages([], ['en'])).toEqual([])
  })

  it('lists each language once, the Chinese ones with a hint', () => {
    const codes = SPOKEN_LANGUAGES.map((language) => language.code)
    expect(new Set(codes).size).toBe(codes.length)
    expect(codes.slice(0, 3)).toEqual(['en', 'yue', 'zh'])
    expect(codes.some((code) => code.startsWith('zh-'))).toBe(false)
    expect(SPOKEN_LANGUAGES.filter((language) => language.hintKey).map((l) => l.code)).toEqual([
      'yue',
      'zh',
    ])
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
