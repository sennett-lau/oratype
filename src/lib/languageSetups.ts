import type { SpeechHardwareCheck } from './tauri'

/** Plan `language-model-suggestions`: built-in model ids (match the backend's tables). */
export const QWEN3_ASR_MODEL = 'qwen3-asr-1.7b'
export const QWEN35_AI_MODEL = 'qwen3.5-4b'
/** The AI model used when the Mac cannot run a setup's AI model (under 16 GB). */
export const FASTER_AI_MODEL = 'qwen3-1.7b'

/** Product names of the built-in models; not translated. */
const MODEL_LABELS: Record<string, string> = {
  'large-v3-turbo': 'Whisper large-v3-turbo',
  small: 'Whisper small',
  [QWEN3_ASR_MODEL]: 'Qwen3-ASR 1.7B',
  'qwen3-4b': 'Qwen3 4B Instruct',
  [FASTER_AI_MODEL]: 'Qwen3 1.7B',
  [QWEN35_AI_MODEL]: 'Qwen 3.5 4B',
}

export function modelLabel(id: string): string {
  return MODEL_LABELS[id] ?? id
}

/**
 * A suggested setup: one speech and one AI model for a combination of languages. Only
 * combinations listed here get a suggestion; each needs results in `evals/` (see the plan's
 * catalogue.md). OraType shows no per-language quality labels.
 */
export interface LanguageSetup {
  /** Stable id, stored when the user hides the Home card. */
  id: string
  matches: (languages: readonly string[]) => boolean
  speech: string
  ai: string
}

export const LANGUAGE_SETUPS: LanguageSetup[] = [
  {
    // Whisper writes Cantonese speech as standard written Chinese; Qwen3-ASR keeps it as said.
    id: 'cantonese',
    matches: (languages) => languages.includes('yue'),
    speech: QWEN3_ASR_MODEL,
    ai: QWEN35_AI_MODEL,
  },
]

/** The setup for these languages, or null when OraType has none for them. */
export function setupFor(languages: readonly string[]): LanguageSetup | null {
  return LANGUAGE_SETUPS.find((setup) => setup.matches(languages)) ?? null
}

/** What a setup means on this Mac: the models to pick, given what the Mac is offered. */
export interface SetupSuggestion {
  setup: LanguageSetup
  /** The speech model to pick, or null when this Mac is not offered it. */
  speech: string | null
  /** The AI model to pick: the setup's, or the faster one when the Mac has too little memory. */
  ai: string | null
  /** True when `ai` is the faster model instead of the setup's. */
  aiFallback: boolean
}

function offered(check: SpeechHardwareCheck | null, id: string): boolean {
  return Boolean(check?.offer.models.some((model) => model.id === id))
}

/**
 * The suggestion for these languages on this Mac, or null without a setup. While a hardware
 * check is still loading, the setup's own model is assumed.
 */
export function suggestionFor(
  languages: readonly string[],
  speechCheck: SpeechHardwareCheck | null,
  aiCheck: SpeechHardwareCheck | null,
): SetupSuggestion | null {
  const setup = setupFor(languages)
  if (!setup) return null
  const speech = !speechCheck || offered(speechCheck, setup.speech) ? setup.speech : null
  let ai: string | null = setup.ai
  let aiFallback = false
  if (aiCheck && !offered(aiCheck, setup.ai)) {
    aiFallback = true
    ai = offered(aiCheck, FASTER_AI_MODEL) ? FASTER_AI_MODEL : null
  }
  return { setup, speech, ai, aiFallback }
}

/**
 * The engine a service uses now: Built-in with its tested model (null until one is), or the
 * user's own server or API key.
 */
export interface CurrentEngine {
  builtin: boolean
  model: string | null
}

/**
 * The models of the suggestion that are not in use yet: what the Home card offers. A service on
 * the user's own server is left alone, and a fallback AI model is never pushed. Empty when
 * nothing is missing (or cannot run here).
 */
export function missingModels(
  suggestion: SetupSuggestion | null,
  current: { speech: CurrentEngine; ai: CurrentEngine },
): { kind: 'speech' | 'ai'; id: string }[] {
  if (!suggestion) return []
  const missing: { kind: 'speech' | 'ai'; id: string }[] = []
  if (suggestion.speech && current.speech.builtin && suggestion.speech !== current.speech.model) {
    missing.push({ kind: 'speech', id: suggestion.speech })
  }
  if (
    suggestion.ai &&
    !suggestion.aiFallback &&
    current.ai.builtin &&
    suggestion.ai !== current.ai.model
  ) {
    missing.push({ kind: 'ai', id: suggestion.ai })
  }
  return missing
}
