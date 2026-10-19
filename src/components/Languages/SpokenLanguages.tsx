import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'
import { SPOKEN_LANGUAGES, spokenLanguage, spokenLanguageList } from '../../lib/spokenLanguages'
import { formatSize } from '../../lib/speechSetup'
import { modelLabel, type SetupSuggestion } from '../../lib/languageSetups'
import { saveSpokenLanguages, useLanguageSetup } from '../../hooks/useLanguageSetup'

/**
 * Plan `language-model-suggestions`: the languages the user speaks, as tokens with a remove
 * button and a "+ Add language" menu, like the Translate languages. Every change is saved at once.
 */
export function SpokenLanguagesPicker({ labelId }: { labelId?: string }) {
  const { t } = useTranslation()
  const { languages } = useLanguageSetup()
  const [error, setError] = useState<string | null>(null)

  const save = (next: string[]) => {
    setError(null)
    saveSpokenLanguages(next).catch((err) => setError(String(err)))
  }
  const available = SPOKEN_LANGUAGES.filter((language) => !languages.includes(language.code))

  return (
    <div className="flex flex-col gap-1.5">
      <div className="spoken-langs" role="list" aria-labelledby={labelId}>
        {languages.map((code) => {
          const language = spokenLanguage(code)
          const hint = language?.hintKey ? t(language.hintKey) : null
          const name = language?.name ?? code
          return (
            <div key={code} role="listitem" className="lang-slot spoken-lang">
              <span className="lang-slot-name">
                {name}
                {hint && <small className="spoken-lang-hint">{hint}</small>}
              </span>
              <button
                type="button"
                className="lang-slot-remove"
                aria-label={t('languageSetup.remove', { language: hint ?? name })}
                onClick={() => save(languages.filter((item) => item !== code))}
              >
                <X size={11} />
              </button>
            </div>
          )
        })}
        {available.length > 0 && (
          <select
            className="lang-slot lang-slot-add spoken-lang-add"
            aria-label={t('languageSetup.addLabel')}
            value=""
            onChange={(event) => save([...languages, event.target.value])}
          >
            <option value="" disabled>
              {t('languageSetup.add')}
            </option>
            {available.map((language) => (
              <option key={language.code} value={language.code}>
                {language.hintKey ? `${language.name} (${t(language.hintKey)})` : language.name}
              </option>
            ))}
          </select>
        )}
      </div>
      {error && <span className="text-[12px] text-error">{error}</span>}
    </div>
  )
}

/**
 * The result of the languages step: the suggested setup for the chosen languages, or a note that
 * OraType has none for them (the standard models are used).
 */
export function SetupSuggestionBox({
  suggestion,
  languages,
  sizes,
}: {
  suggestion: SetupSuggestion | null
  languages: readonly string[]
  sizes: Record<string, number | undefined>
}) {
  const { t } = useTranslation()
  const list = spokenLanguageList(languages, t)
  if (languages.length === 0) {
    return (
      <div className="setup-suggestion setup-suggestion-none" data-testid="setup-suggestion">
        <span className="status-detail">{t('languageSetup.empty')}</span>
      </div>
    )
  }
  if (!suggestion) {
    return (
      <div className="setup-suggestion setup-suggestion-none" data-testid="setup-suggestion">
        <span className="setup-suggestion-title">
          {t('languageSetup.noneTitle', { languages: list })}
        </span>
        <span className="status-detail">{t('languageSetup.noneBody')}</span>
      </div>
    )
  }
  const rows: { label: string; id: string | null }[] = [
    { label: t('onboarding.steps.speechRecognition'), id: suggestion.speech },
    { label: t('onboarding.steps.aiPolish'), id: suggestion.ai },
  ]
  return (
    <div className="setup-suggestion" data-testid="setup-suggestion">
      <span className="setup-suggestion-title">
        {t('languageSetup.suggestedTitle', { languages: list })}
      </span>
      <div className="setup-suggestion-rows">
        {rows
          .filter((row) => row.id)
          .map((row) => {
            const size = sizes[row.id as string]
            return (
              <div key={row.label} className="contents">
                <span className="status-detail">{row.label}</span>
                <span className="text-[12.5px] text-text-primary">
                  {modelLabel(row.id as string)}
                </span>
                <span className="status-detail tabular-nums">{size ? formatSize(size) : ''}</span>
              </div>
            )
          })}
      </div>
      {suggestion.aiFallback && (
        <span className="status-detail">{t('languageSetup.aiFallback')}</span>
      )}
      <span className="status-detail">{t('languageSetup.nextSteps')}</span>
    </div>
  )
}
