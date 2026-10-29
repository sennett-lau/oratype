import { useState, type KeyboardEvent } from 'react'
import { useTranslation } from 'react-i18next'
import { Plus, X } from 'lucide-react'
import {
  MAX_TRANSLATION_TARGETS,
  TARGET_LANGUAGES,
  targetLanguageLabel,
} from '../../../lib/constants'
import type { LibraryStatus } from '../../../lib/tauri'
import {
  languageSettings,
  useAppStore,
  type AppConfig,
  type TranslationConfig,
} from '../../../stores/appStore'
import { Toggle } from '../shared/Toggle'
import { saveTranslationLanguage } from '../translationLanguages'
import { needsUpdateDecision, usePresetDetail } from './languageLibrary'

interface LanguageRowsProps {
  config: AppConfig
  status: LibraryStatus
  /** Changes the language list (order, add, remove): an unsaved Settings edit. */
  onChange: (translation: TranslationConfig) => void
  /** Opens a language's sheet. */
  onEdit: (code: string) => void
}

/**
 * Plan `language-prompt-library`: Settings → Translate as one row per language, in the saved
 * order (the order the Switch language key and the pill follow). Each row: a drag handle, its
 * number, the name with an "Update" tag when a decision is needed, a line saying where its
 * instructions come from, an on/off switch (saved at once, like the sheet), Edit and remove.
 * "+ Add language" is the last row while fewer than three are chosen. Plan
 * `languages-in-general`: Settings → General → Your languages reuses the row and the Add row.
 */
export function LanguageRows({ config, status, onChange, onEdit }: LanguageRowsProps) {
  const { t } = useTranslation()
  const [dragIndex, setDragIndex] = useState<number | null>(null)
  const translation = config.translation
  const targets = translation.targets
  const canRemove = targets.length > 1
  const canAdd = targets.length < MAX_TRANSLATION_TARGETS
  const available = TARGET_LANGUAGES.filter((language) => !targets.includes(language.value))

  const move = (from: number, to: number) => {
    if (from === to || to < 0 || to >= targets.length) return
    const next = [...targets]
    const [code] = next.splice(from, 1)
    next.splice(to, 0, code)
    onChange({ ...translation, targets: next })
  }

  const remove = (code: string) => {
    if (!canRemove) return
    const index = targets.indexOf(code)
    const next = targets.filter((target) => target !== code)
    const activeTarget =
      code === translation.active_target
        ? next[Math.min(index, next.length - 1)]
        : translation.active_target
    onChange({ ...translation, targets: next, active_target: activeTarget })
  }

  const add = (code: string) => {
    if (!code || targets.includes(code) || !canAdd) return
    onChange({ ...translation, targets: [...targets, code] })
    onEdit(code)
  }

  return (
    <div className="language-rows" role="list" aria-label={t('translate.targetsLabel')}>
      {targets.map((code, index) => (
        <LanguageRow
          key={code}
          code={code}
          config={config}
          status={status}
          canRemove={canRemove}
          onEdit={() => onEdit(code)}
          onRemove={() => remove(code)}
          order={{
            index,
            dragging: dragIndex === index,
            onMove: (to) => move(index, to),
            onDragStart: () => setDragIndex(index),
            onDragEnd: () => setDragIndex(null),
            onDrop: () => {
              if (dragIndex !== null) move(dragIndex, index)
              setDragIndex(null)
            },
          }}
        />
      ))}
      {canAdd && (
        <LanguageAddRow
          codes={available.map((language) => language.value)}
          max={MAX_TRANSLATION_TARGETS}
          nameOf={(code) => targetLanguageLabel(code, t)}
          onAdd={add}
        />
      )}
    </div>
  )
}

/** "+ Add language" with its limit; pressing it turns the row into the language menu. */
export function LanguageAddRow({
  codes,
  max,
  nameOf,
  onAdd,
}: {
  /** The languages that can still be added, in menu order. */
  codes: string[]
  max: number
  nameOf: (code: string) => string
  onAdd: (code: string) => void
}) {
  const { t } = useTranslation()
  const [adding, setAdding] = useState(false)
  if (adding) {
    return (
      <div className="language-add-row">
        <select
          autoFocus
          defaultValue=""
          aria-label={t('translate.addLanguage')}
          onChange={(event) => {
            setAdding(false)
            onAdd(event.target.value)
          }}
          onBlur={() => setAdding(false)}
          className="popup"
        >
          <option value="" disabled>
            {t('translate.chooseLanguage')}
          </option>
          {codes.map((code) => (
            <option key={code} value={code}>
              {nameOf(code)}
            </option>
          ))}
        </select>
      </div>
    )
  }
  return (
    <button type="button" onClick={() => setAdding(true)} className="language-add-row">
      <Plus size={13} aria-hidden="true" />
      {t('translate.addLanguage')}
      <span className="language-add-limit">{t('translate.language.upTo', { count: max })}</span>
    </button>
  )
}

/** A row's place in an ordered list (Translate): its drag handle and number. */
interface RowOrder {
  index: number
  dragging: boolean
  onMove: (to: number) => void
  onDragStart: () => void
  onDragEnd: () => void
  onDrop: () => void
}

interface LanguageRowProps {
  code: string
  /** The name shown; the Translate list's name by default. */
  name?: string
  config: AppConfig
  status: LibraryStatus
  canRemove: boolean
  onEdit: () => void
  onRemove: () => void
  /** Without it (Your languages) the row has no drag handle and no number. */
  order?: RowOrder
}

/**
 * One language: its name, where its instructions come from, its on/off switch, Edit and remove.
 * The settings are the language's own, shared by Your languages and Translate.
 */
export function LanguageRow({
  code,
  name: shownName,
  config,
  status,
  canRemove,
  onEdit,
  onRemove,
  order,
}: LanguageRowProps) {
  const { t } = useTranslation()
  const settings = languageSettings(config, code)
  const name = shownName ?? targetLanguageLabel(code, t)
  const { detail, missing } = usePresetDetail(code, settings.library_preset)
  const presetName = detail?.name ?? settings.library_preset?.id ?? ''
  const needsDecision = needsUpdateDecision(settings, status)
  const [saving, setSaving] = useState(false)

  const source = (() => {
    if (!settings.enabled) return t('translate.language.sourceOff')
    const preset = settings.library_preset
    if (preset && settings.instructions !== null)
      return t('translate.language.sourceEdited', { name: presetName })
    if (settings.instructions !== null) return t('translate.language.sourceBuiltinEdited')
    if (preset && missing)
      return t('translate.language.sourceMissing', { name: presetName, version: preset.version })
    if (preset)
      return t(
        settings.auto_update
          ? 'translate.language.sourcePresetAuto'
          : 'translate.language.sourcePreset',
        { name: presetName, version: preset.version },
      )
    return t('translate.language.sourceBuiltin')
  })()

  const toggle = async (enabled: boolean) => {
    if (saving) return
    setSaving(true)
    try {
      const stored = useAppStore.getState().savedConfig ?? useAppStore.getState().config
      await saveTranslationLanguage(code, { ...languageSettings(stored, code), enabled })
    } catch (error) {
      console.error('[settings] failed to save the language switch', error)
    } finally {
      setSaving(false)
    }
  }

  const onHandleKey = (event: KeyboardEvent<HTMLButtonElement>) => {
    if (!order) return
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      order.onMove(order.index - 1)
    } else if (event.key === 'ArrowDown') {
      event.preventDefault()
      order.onMove(order.index + 1)
    }
  }

  return (
    <div
      role="listitem"
      data-testid={`language-row-${code}`}
      className={`language-row ${settings.enabled ? '' : 'language-row-off'} ${order?.dragging ? 'language-row-dragging' : ''}`}
      onDragOver={order ? (event) => event.preventDefault() : undefined}
      onDrop={
        order
          ? (event) => {
              event.preventDefault()
              order.onDrop()
            }
          : undefined
      }
    >
      {order && (
        <>
          <button
            type="button"
            draggable
            onDragStart={(event) => {
              event.dataTransfer?.setData('text/plain', code)
              order.onDragStart()
            }}
            onDragEnd={order.onDragEnd}
            onKeyDown={onHandleKey}
            aria-label={t('translate.language.moveLabel', { language: name })}
            title={t('translate.language.dragHint')}
            className="language-grip"
          >
            ⋮⋮
          </button>
          <span className="language-number" aria-hidden="true">
            {order.index + 1}
          </span>
        </>
      )}
      <div className="language-info">
        <span className="language-name">
          <span className="truncate">{name}</span>
          {needsDecision && <span className="tag">{t('translate.language.updateTag')}</span>}
        </span>
        <span className="language-source">{source}</span>
      </div>
      <div className="language-actions">
        <Toggle
          checked={settings.enabled}
          disabled={saving}
          onChange={toggle}
          label={t('translate.language.switchLabel', { language: name })}
          hideLabel
        />
        <button
          type="button"
          onClick={onEdit}
          aria-label={t('translate.language.editLabel', { language: name })}
          className="btn-secondary px-2.5 py-1 text-[12px]"
        >
          {t('translate.language.edit')}
        </button>
        {canRemove && (
          <button
            type="button"
            onClick={onRemove}
            aria-label={`${t('translate.remove')} ${name}`}
            title={t('translate.remove')}
            className="btn-icon"
          >
            <X size={13} />
          </button>
        )}
      </div>
    </div>
  )
}
