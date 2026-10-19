import { useMemo } from 'react'
import { useAppStore, type AppConfig } from '../stores/appStore'
import { useSpeechSetupStore } from '../stores/speechSetupStore'
import { useAiSetupStore } from '../stores/aiSetupStore'
import { dismissLanguageSetup, setSpokenLanguages } from '../lib/tauri'
import { effectiveSpokenLanguages } from '../lib/spokenLanguages'
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
 * Plan `language-model-suggestions`: the languages the user speaks, and the suggested setup for
 * them on this Mac (null when OraType has none for those languages). Reads both hardware checks.
 */
export function useLanguageSetup() {
  const chosen = useAppStore((s) => s.config.spoken_languages)
  const speechHardware = useSetupHardware(useSpeechSetupStore)
  const aiHardware = useSetupHardware(useAiSetupStore)
  const languages = useMemo(() => effectiveSpokenLanguages(chosen), [chosen])
  const suggestion = useMemo(
    () => suggestionFor(languages, speechHardware, aiHardware),
    [languages, speechHardware, aiHardware],
  )
  return { languages, chosen: chosen !== null, suggestion }
}

/** The model of a service's suggestion, if the user's languages have one that runs here. */
export function useSuggestedModel(service: EngineService): string | null {
  const { suggestion } = useLanguageSetup()
  if (!suggestion) return null
  return service.id === 'speech' ? suggestion.speech : suggestion.ai
}

/** Saves the spoken languages and puts the stored list in both configs (no Save bar). */
export async function saveSpokenLanguages(languages: string[]): Promise<void> {
  const saved = await setSpokenLanguages(languages)
  useAppStore.getState().applyPersistedConfigPatch({ spoken_languages: saved })
}

/** "Don't show again" on the Home card of a suggested setup. */
export async function hideLanguageSetup(setupId: string): Promise<void> {
  await dismissLanguageSetup(setupId)
  useAppStore.getState().applyPersistedConfigPatch({ dismissed_language_setup: setupId })
}
