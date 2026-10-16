import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useAppStore } from '../../stores/appStore'
import { settingsPaneHash } from '../../lib/router'
import { formatSize } from '../../lib/speechSetup'
import { missingModels, modelLabel } from '../../lib/languageSetups'
import { currentEngine, hideLanguageSetup, useLanguageSetup } from '../../hooks/useLanguageSetup'
import { AI_SERVICE, SPEECH_SERVICE } from '../Speech/services'
import { spokenLanguageList } from '../../lib/spokenLanguages'
import { Group, Row } from '../ui/Group'

/**
 * Plan `language-model-suggestions`: "Suggested for your languages" on Home, styled like Finish
 * setup. It shows while a model of the user's suggested setup is not in use on a Built-in engine
 * (for example after skipping it in onboarding). "Set up" opens the Settings pane, as Finish setup
 * does, so nothing downloads from Home; "Don't show again" hides it for that setup.
 */
export function LanguageSetupCard() {
  const { t } = useTranslation()
  const config = useAppStore((s) => s.config)
  const { languages, suggestion } = useLanguageSetup()
  const [hiding, setHiding] = useState(false)

  if (!suggestion || config.dismissed_language_setup === suggestion.setup.id) return null
  const missing = missingModels(suggestion, {
    speech: currentEngine(SPEECH_SERVICE, config),
    ai: currentEngine(AI_SERVICE, config),
  })
  if (missing.length === 0) return null

  const sizes: Record<string, number | undefined> = { ...SPEECH_SERVICE.sizes, ...AI_SERVICE.sizes }
  const total = missing.reduce((sum, model) => sum + (sizes[model.id] ?? 0), 0)
  const speech = missing.find((model) => model.kind === 'speech')
  const ai = missing.find((model) => model.kind === 'ai')
  const bodyKey =
    speech && ai
      ? 'languageSetup.homeBodyBoth'
      : speech
        ? 'languageSetup.homeBodySpeech'
        : 'languageSetup.homeBodyAi'

  const hide = () => {
    setHiding(true)
    hideLanguageSetup(suggestion.setup.id).catch((error) => {
      console.error('[language setup] hide failed', error)
      setHiding(false)
    })
  }

  return (
    <div className="mb-[22px]" data-testid="language-setup-card">
      <Group label={t('languageSetup.homeGroup')}>
        <Row
          label={t('languageSetup.homeTitle', { languages: spokenLanguageList(languages, t) })}
          help={t(bodyKey, {
            speech: speech ? modelLabel(speech.id) : '',
            ai: ai ? modelLabel(ai.id) : '',
            size: formatSize(total),
          })}
        >
          <button type="button" onClick={hide} disabled={hiding} className="btn-secondary">
            {t('languageSetup.dontShowAgain')}
          </button>
          <button
            type="button"
            onClick={() => {
              window.location.hash = settingsPaneHash(missing[0].kind === 'speech' ? 'stt' : 'llm')
            }}
            className="btn-accent"
          >
            {t('home.setup.setUp')}
          </button>
        </Row>
      </Group>
    </div>
  )
}
