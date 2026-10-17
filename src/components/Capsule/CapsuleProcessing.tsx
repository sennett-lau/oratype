import { useTranslation } from 'react-i18next'
import { CapsuleWorking } from './CapsuleWorking'

/**
 * Transcribing: the speech server is turning the recording into text. A hands-free request that
 * will be typed shows it too (plan `hands-free-mode`); that Ask run is cancelled with Escape, so
 * it has no cancel button.
 */
export function CapsuleProcessing({ cancellable = true }: { cancellable?: boolean }) {
  const { t } = useTranslation()
  return (
    <CapsuleWorking
      label={t('capsule.transcribing')}
      cancelLabel={cancellable ? t('capsule.cancelProcessing') : undefined}
    />
  )
}
