import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { X } from 'lucide-react'
import {
  MAX_YOUR_LANGUAGES,
  YOUR_LANGUAGE_CODES,
  yourLanguageList,
  yourLanguageName,
} from '../../lib/yourLanguages'
import { formatSize } from '../../lib/speechSetup'
import { modelLabel, type SetupSuggestion } from '../../lib/languageSetups'
import { saveYourLanguages, useLanguageSetup } from '../../hooks/useLanguageSetup'

/**
 * Plan `language-model-suggestions`: the onboarding picker of Your languages, as tokens with a
 * remove button and a "+ Add language" menu. Every change is saved at once. Plan
 * `languages-in-general`: the codes are written languages; Settings → General shows the same list
 * as rows with their presets.
 */
export function YourLanguagesPicker({ labelId }: { labelId?: string }) {
  const { t } = useTranslation()
  const { languages } = useLanguageSetup()
  const [error, setError] = useState<string | null>(null)

  const save = (next: string[]) => {
    setError(null)
    saveYourLanguages(next).catch((err) => setError(String(err)))
  }
  const available = YOUR_LANGUAGE_CODES.filter((code) => !languages.includes(code))

  return (
    <div className="flex flex-col gap-1.5">
      <div className="spoken-langs" role="list" aria-labelledby={labelId}>
        {languages.map((code) => {
          const name = yourLanguageName(code, t)
          return (
            <div key={code} role="listitem" className="lang-slot spoken-lang">
              <span className="lang-slot-name">{name}</span>
              <button
                type="button"
                className="lang-slot-remove"
                aria-label={t('languageSetup.remove', { language: name })}
                onClick={() => save(languages.filter((item) => item !== code))}
              >
                <X size={11} />
              </button>
            </div>
          )
        })}
        {available.length > 0 && languages.length < MAX_YOUR_LANGUAGES && (
          <select
            className="lang-slot lang-slot-add spoken-lang-add"
            aria-label={t('languageSetup.addLabel')}
            value=""
            onChange={(event) => save([...languages, event.target.value])}
          >
            <option value="" disabled>
              {t('languageSetup.add')}
            </option>
            {available.map((code) => (
              <option key={code} value={code}>
                {yourLanguageName(code, t)}
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
  const list = yourLanguageList(languages, t)
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
