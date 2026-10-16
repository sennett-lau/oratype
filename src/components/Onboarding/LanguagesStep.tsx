import { useTranslation } from 'react-i18next'
import { useLanguageSetup } from '../../hooks/useLanguageSetup'
import { AI_SERVICE, SPEECH_SERVICE } from '../Speech/services'
import { SetupSuggestionBox, SpokenLanguagesPicker } from '../Languages/SpokenLanguages'

/**
 * Plan `language-model-suggestions`: Onboarding → Your languages, between Voice input and Speech
 * recognition. It only looks up a suggested setup for the chosen languages; the next two steps
 * start with its models chosen, and the user downloads each one there.
 */
export function LanguagesStep() {
  const { t } = useTranslation()
  const { languages, chosen, suggestion } = useLanguageSetup()
  const sizes = { ...SPEECH_SERVICE.sizes, ...AI_SERVICE.sizes }
  return (
    <div className="flex flex-col gap-3" data-testid="languages-step">
      <div className="flex flex-col gap-1.5">
        <span id="spoken-languages-label" className="sr-only">
          {t('languageSetup.yourLanguages')}
        </span>
        <SpokenLanguagesPicker labelId="spoken-languages-label" />
        {!chosen && languages.length > 0 && (
          <span className="hardware-note">{t('languageSetup.fromMac')}</span>
        )}
      </div>
      <SetupSuggestionBox suggestion={suggestion} languages={languages} sizes={sizes} />
    </div>
  )
}
