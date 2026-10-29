import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { openUrl } from '@tauri-apps/plugin-opener'
import { useAppStore } from '../../stores/appStore'
import { saveYourLanguages } from '../../hooks/useLanguageSetup'
import {
  MAX_YOUR_LANGUAGES,
  YOUR_LANGUAGE_CODES,
  macLanguageTags,
  yourLanguageList,
  yourLanguageName,
  yourLanguagesFromTags,
} from '../../lib/yourLanguages'
import { Group, Row } from '../ui/Group'
import { LanguageAddRow, LanguageRow } from './languages/LanguageRows'
import { LANGUAGE_PRESETS_GUIDE_URL, useLibraryStatus } from './languages/languageLibrary'
import { LanguageSheet } from './languages/LanguageSheet'

/**
 * Plan `languages-in-general`: Settings → General → Your languages, the languages the user speaks
 * and dictates in. Dictate and Ask pick the one spoken from this list and follow its preset,
 * with every speech and AI service. One row per language, as in Translate but without an order:
 * where its instructions come from, its switch, Edit (the same sheet) and remove. A language's
 * settings are shared with Translate. The list itself is saved at once, not by the Save bar.
 * Until the user has chosen, a button offers the Mac's preferred languages.
 */
export function YourLanguagesGroup() {
  const { t } = useTranslation()
  const config = useAppStore((s) => s.config)
  const libraryStatus = useLibraryStatus()
  const [editing, setEditing] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)
  const chosen = config.your_languages
  const languages = chosen ?? []
  const macLanguages = chosen == null ? yourLanguagesFromTags(macLanguageTags()) : []

  const save = (next: string[]) => {
    setError(null)
    saveYourLanguages(next).catch((err) => setError(String(err)))
  }

  const add = (code: string) => {
    if (!code || languages.includes(code) || languages.length >= MAX_YOUR_LANGUAGES) return
    save([...languages, code])
    setEditing(code)
  }

  return (
    <>
      <Group
        label={t('languageSetup.yourLanguages')}
        actions={
          <button
            type="button"
            onClick={() =>
              openUrl(LANGUAGE_PRESETS_GUIDE_URL).catch((err) =>
                console.error('[settings] failed to open the guide', err),
              )
            }
            className="link-button normal-case tracking-normal"
          >
            {t('translate.language.aboutPresets')}
          </button>
        }
      >
        <div
          className="language-rows"
          role="list"
          aria-label={t('languageSetup.yourLanguages')}
          data-testid="your-languages"
        >
          {languages.map((code) => (
            <LanguageRow
              key={code}
              code={code}
              name={yourLanguageName(code, t)}
              config={config}
              status={libraryStatus}
              canRemove
              onEdit={() => setEditing(code)}
              onRemove={() => save(languages.filter((item) => item !== code))}
            />
          ))}
          {languages.length < MAX_YOUR_LANGUAGES && (
            <LanguageAddRow
              codes={YOUR_LANGUAGE_CODES.filter((code) => !languages.includes(code))}
              max={MAX_YOUR_LANGUAGES}
              nameOf={(code) => yourLanguageName(code, t)}
              onAdd={add}
            />
          )}
        </div>
        <Row help={t('languageSetup.generalHelp')}>
          {macLanguages.length > 0 && (
            <button type="button" onClick={() => save(macLanguages)} className="btn-secondary">
              {t('languageSetup.useMac', { languages: yourLanguageList(macLanguages, t) })}
            </button>
          )}
        </Row>
        {error && (
          <Row>
            <span className="text-[12px] text-error">{error}</span>
          </Row>
        )}
      </Group>

      {editing && (
        <LanguageSheet
          code={editing}
          name={yourLanguageName(editing, t)}
          status={libraryStatus}
          onClose={() => setEditing(null)}
        />
      )}
    </>
  )
}
