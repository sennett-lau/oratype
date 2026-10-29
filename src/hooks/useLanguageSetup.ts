import { useMemo } from 'react'
import { useAppStore, type AppConfig } from '../stores/appStore'
import { useSpeechSetupStore } from '../stores/speechSetupStore'
import { useAiSetupStore } from '../stores/aiSetupStore'
import { dismissLanguageSetup, setYourLanguages } from '../lib/tauri'
import { effectiveYourLanguages, spokenCodesOf } from '../lib/yourLanguages'
import { suggestionFor, type CurrentEngine } from '../lib/languageSetups'
import { activePresetOf, type EngineService } from '../components/Speech/services'
import { useSetupHardware } from './useSpeechSetup'

/** The engine a service uses now (see `CurrentEngine`). */
export function currentEngine(service: EngineService, config: AppConfig): CurrentEngine {
  const preset = activePresetOf(service, config)
  if (!preset || !service.isBuiltin(preset)) return { builtin: false, model: null }
  return { builtin: true, model: preset.model_file && preset.verified_at ? preset.model : null }
}

/**
 * Plan `language-model-suggestions`: Your languages (plan `languages-in-general`), and the
 * suggested setup for them on this Mac (null when OraType has none for those languages). The
 * setups are by spoken language, so Cantonese (Hong Kong) counts as Cantonese. Reads both
 * hardware checks.
 */
export function useLanguageSetup() {
  const chosen = useAppStore((s) => s.config.your_languages)
  const speechHardware = useSetupHardware(useSpeechSetupStore)
  const aiHardware = useSetupHardware(useAiSetupStore)
  const languages = useMemo(() => effectiveYourLanguages(chosen), [chosen])
  const suggestion = useMemo(
    () => suggestionFor(spokenCodesOf(languages), speechHardware, aiHardware),
    [languages, speechHardware, aiHardware],
  )
  return { languages, chosen: chosen != null, suggestion }
}

/** The model of a service's suggestion, if the user's languages have one that runs here. */
export function useSuggestedModel(service: EngineService): string | null {
  const { suggestion } = useLanguageSetup()
  if (!suggestion) return null
  return service.id === 'speech' ? suggestion.speech : suggestion.ai
}

/** Saves Your languages and puts the stored list in both configs (no Save bar). */
export async function saveYourLanguages(languages: string[]): Promise<void> {
  const saved = await setYourLanguages(languages)
  useAppStore.getState().applyPersistedConfigPatch({ your_languages: saved ?? languages })
}

/** "Don't show again" on the Home card of a suggested setup. */
export async function hideLanguageSetup(setupId: string): Promise<void> {
  await dismissLanguageSetup(setupId)
  useAppStore.getState().applyPersistedConfigPatch({ dismissed_language_setup: setupId })
}
