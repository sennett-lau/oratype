import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../../../i18n'
import { useAppStore } from '../../../stores/appStore'
import { CapsuleRecording } from '../CapsuleRecording'
import { CapsuleProcessing } from '../CapsuleProcessing'

vi.mock('../../../lib/tauri', () => ({
  abortAskDictation: vi.fn().mockResolvedValue(undefined),
  abortRecording: vi.fn().mockResolvedValue(undefined),
}))

vi.mock('../Waveform', () => ({
  Waveform: () => <div data-testid="waveform" />,
}))

vi.mock('../TranslatePillLanguage', () => ({
  TranslatePillLanguage: () => <div data-testid="translate-language" />,
}))

import { abortAskDictation, abortRecording } from '../../../lib/tauri'

beforeEach(async () => {
  await i18n.changeLanguage('en')
  useAppStore.setState(useAppStore.getInitialState())
})

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
})

// Plan `hands-free-mode`: a hands-free request that will be typed shows the Dictate pill.
describe('Hands-free request that will be typed', () => {
  it('cancels the hands-free run, not a dictation', () => {
    render(<CapsuleRecording handsFree />)
    fireEvent.click(screen.getByRole('button', { name: 'Cancel recording' }))
    expect(abortAskDictation).toHaveBeenCalledTimes(1)
    expect(abortRecording).not.toHaveBeenCalled()
  })

  it('never shows a Translate language left over from an earlier run', () => {
    useAppStore.setState({ activeVoiceMode: 'translate' })
    const { rerender } = render(<CapsuleRecording handsFree />)
    expect(screen.queryByTestId('translate-language')).toBeNull()
    rerender(<CapsuleRecording />)
    expect(screen.getByTestId('translate-language')).toBeDefined()
  })

  it('shows "Transcribing" without a cancel button while the request is read', () => {
    const { rerender } = render(<CapsuleProcessing cancellable={false} />)
    expect(screen.getByText('Transcribing')).toBeDefined()
    expect(screen.queryByRole('button')).toBeNull()
    rerender(<CapsuleProcessing />)
    expect(screen.getByRole('button')).toBeDefined()
  })
})
