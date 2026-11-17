import { useEffect, useId, useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  getBuiltinBackendInfo,
  type BuiltinBackendInfo,
  type LlamaDeviceInfo,
} from '../../lib/tauri'
import { useAppStore } from '../../stores/appStore'
import { Group, Row } from '../ui/Group'

function formatDeviceOption(device: LlamaDeviceInfo): string {
  const gb = Math.max(1, Math.round(device.totalMib / 1024))
  return `${device.id} · ${device.description} (${gb} GB)`
}

/**
 * The engine, the device in use and the Auto / GPU / CPU choice for a built-in service. Windows
 * only: a PC can have several GPUs or none, while a Mac always runs on its own GPU (Metal), so
 * there is nothing to choose there.
 */
export function BuiltinBackendCard({ kind }: { kind: 'ai' | 'speech' }) {
  const { t } = useTranslation()
  const selectId = useId()
  const config = useAppStore((s) => s.config)
  const updateConfig = useAppStore((s) => s.updateConfig)
  const choice =
    kind === 'ai'
      ? (config.builtin_ai_backend ?? 'auto')
      : (config.builtin_speech_backend ?? 'auto')
  const [info, setInfo] = useState<BuiltinBackendInfo | null>(null)

  useEffect(() => {
    let cancelled = false
    Promise.resolve(getBuiltinBackendInfo?.())
      .then((result) => {
        if (!cancelled && result) {
          setInfo(result)
        }
      })
      .catch(() => {})
    return () => {
      cancelled = true
    }
  }, [choice])

  if (!info || info.platform !== 'windows') {
    return null
  }

  const engineLabel =
    kind === 'ai' ? info.llamaEngine : `${info.whisperEngine} / ${info.llamaEngine}`
  const activeDevice =
    kind === 'ai'
      ? info.activeAiDevice
      : info.activeSpeechLlamaDevice === info.activeWhisperDevice
        ? info.activeSpeechLlamaDevice
        : `${info.whisperEngine}: ${info.activeWhisperDevice} · ${info.llamaEngine}: ${info.activeSpeechLlamaDevice}`

  const handleChange = (next: string) => {
    if (kind === 'ai') {
      updateConfig({ builtin_ai_backend: next })
    } else {
      updateConfig({ builtin_speech_backend: next })
    }
  }

  const autoLabel = info.autoDeviceId
    ? `${t('builtinBackend.auto')} (${info.autoDeviceId})`
    : t('builtinBackend.auto')

  return (
    <Group label={t('builtinBackend.group')}>
      <Row label={t('builtinBackend.engine')}>
        <span className="status-detail" data-testid={`builtin-${kind}-engine`}>
          {engineLabel}
        </span>
      </Row>
      <Row label={t('builtinBackend.activeDevice')}>
        <span className="status-detail" data-testid={`builtin-${kind}-device`}>
          {activeDevice}
        </span>
      </Row>
      <Row
        label={t('builtinBackend.preference')}
        help={t('builtinBackend.preferenceHelp')}
        htmlFor={selectId}
      >
        <select
          id={selectId}
          aria-label={t('builtinBackend.preference')}
          className="popup"
          value={choice}
          onChange={(e) => handleChange(e.target.value)}
        >
          <option value="auto">{autoLabel}</option>
          {info.devices.map((device) => (
            <option key={device.id} value={device.id}>
              {formatDeviceOption(device)}
            </option>
          ))}
          <option value="cpu">{t('builtinBackend.cpuOnly')}</option>
        </select>
      </Row>
    </Group>
  )
}
