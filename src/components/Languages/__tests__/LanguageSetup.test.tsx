import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as tauri from '../../../lib/tauri'
import { useAppStore } from '../../../stores/appStore'
import { IDLE_SETUP_STATUS, useSpeechSetupStore } from '../../../stores/speechSetupStore'
import { useAiSetupStore } from '../../../stores/aiSetupStore'
import { hardwareCheck, installedBuiltin, serverPreset } from '../../../test-utils/speechHardware'
import { LanguagesStep } from '../../Onboarding/LanguagesStep'
import { LanguageSetupCard } from '../../HomePage/LanguageSetupCard'

vi.mock('../../../lib/tauri')

vi.mock('react-i18next', async () => {
  const { translate } = await import('../../../test-utils/i18nMock')
  return { useTranslation: () => ({ t: translate }) }
})

function setConfig(partial: Partial<ReturnType<typeof useAppStore.getState>['config']>) {
  const next = { ...useAppStore.getState().config, ...partial }
  useAppStore.getState().setConfig(next)
  useAppStore.getState().setSavedConfig(next)
}

beforeEach(() => {
  useAppStore.setState(useAppStore.getInitialState())
  useSpeechSetupStore.setState({ status: IDLE_SETUP_STATUS, models: null, hardware: null })
  useAiSetupStore.setState({ status: IDLE_SETUP_STATUS, models: null, hardware: null })
  vi.clearAllMocks()
  vi.mocked(tauri.getSpeechHardware).mockResolvedValue(
    hardwareCheck(['large-v3-turbo', 'small', 'qwen3-asr-1.7b']),
  )
  vi.mocked(tauri.getAiHardware).mockResolvedValue(
    hardwareCheck(['qwen3-4b', 'qwen3-1.7b', 'qwen3.5-4b'], { serverAvailable: true }),
  )
  vi.mocked(tauri.setSpokenLanguages).mockImplementation(async (languages) => languages)
  vi.mocked(tauri.dismissLanguageSetup).mockResolvedValue(undefined)
  window.location.hash = '#/'
})

afterEach(() => cleanup())

describe('Onboarding → Your languages (plan `language-model-suggestions`)', () => {
  it('shows the suggested setup for a combination with Cantonese', async () => {
    setConfig({ spoken_languages: ['en', 'yue'] })
    render(<LanguagesStep />)

    const box = await screen.findByTestId('setup-suggestion')
    expect(within(box).getByText('Suggested setup for English and Cantonese')).toBeInTheDocument()
    expect(within(box).getByText('Qwen3-ASR 1.7B')).toBeInTheDocument()
    expect(within(box).getByText('Qwen 3.5 4B')).toBeInTheDocument()
    expect(within(box).getByText('2.5 GB')).toBeInTheDocument()
    expect(within(box).getByText('2.7 GB')).toBeInTheDocument()
    // The list is the user's own choice, so no "from your Mac" note.
    expect(screen.queryByText(/Filled in from your Mac/)).not.toBeInTheDocument()
  })

  it('says when there is no suggested setup for the languages', async () => {
    setConfig({ spoken_languages: ['en', 'ja'] })
    render(<LanguagesStep />)
    const box = await screen.findByTestId('setup-suggestion')
    expect(within(box).getByText('No suggested setup for English and 日本語')).toBeInTheDocument()
  })

  it('saves each change at once', async () => {
    setConfig({ spoken_languages: ['en', 'yue'] })
    render(<LanguagesStep />)

    fireEvent.click(screen.getByRole('button', { name: 'Remove Cantonese' }))
    await waitFor(() => expect(tauri.setSpokenLanguages).toHaveBeenCalledWith(['en']))
    await waitFor(() => expect(useAppStore.getState().config.spoken_languages).toEqual(['en']))
    expect(useAppStore.getState().savedConfig?.spoken_languages).toEqual(['en'])

    fireEvent.change(screen.getByRole('combobox', { name: 'Add a language you speak' }), {
      target: { value: 'zh' },
    })
    await waitFor(() => expect(tauri.setSpokenLanguages).toHaveBeenLastCalledWith(['en', 'zh']))
  })

  it('starts from the Mac languages until the user chooses', async () => {
    setConfig({ spoken_languages: null })
    render(<LanguagesStep />)
    expect(await screen.findByText(/Filled in from your Mac/)).toBeInTheDocument()
  })
})

describe('Home → Suggested for your languages', () => {
  it('offers the models of the setup that are not in use, and opens Settings', async () => {
    setConfig({ spoken_languages: ['en', 'yue'] })
    render(<LanguageSetupCard />)

    const card = await screen.findByTestId('language-setup-card')
    expect(within(card).getByText('Built-in models for English and Cantonese')).toBeInTheDocument()
    expect(
      within(card).getByText(
        'Qwen3-ASR 1.7B for speech recognition and Qwen 3.5 4B for AI polish. They run on this Mac; 5.3 GB to download.',
      ),
    ).toBeInTheDocument()
    fireEvent.click(within(card).getByRole('button', { name: 'Set up' }))
    expect(window.location.hash).toBe('#/settings?pane=stt')
  })

  it('hides for good with "Don’t show again"', async () => {
    setConfig({ spoken_languages: ['yue'] })
    render(<LanguageSetupCard />)
    const card = await screen.findByTestId('language-setup-card')
    fireEvent.click(within(card).getByRole('button', { name: 'Don’t show again' }))

    await waitFor(() => expect(tauri.dismissLanguageSetup).toHaveBeenCalledWith('cantonese'))
    await waitFor(() => expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument())
  })

  it('stays away without a setup, when dismissed, or when the user has their own server', async () => {
    setConfig({ spoken_languages: ['en'] })
    const { unmount } = render(<LanguageSetupCard />)
    await waitFor(() => expect(tauri.getAiHardware).toHaveBeenCalled())
    expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument()
    unmount()

    setConfig({ spoken_languages: ['yue'], dismissed_language_setup: 'cantonese' })
    render(<LanguageSetupCard />)
    expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument()
    cleanup()

    const config = useAppStore.getState().config
    setConfig({
      dismissed_language_setup: null,
      speech_presets: [...config.speech_presets, serverPreset('pc', 'PC', 'http://pc:8180/v1')],
      active_speech_preset_id: 'pc',
      ai_presets: config.ai_presets.map((preset) => ({
        ...preset,
        kind: 'openai_compatible' as const,
      })),
    })
    render(<LanguageSetupCard />)
    expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument()
  })

  it('names only the missing model when the other is in use', async () => {
    setConfig({
      spoken_languages: ['yue'],
      speech_presets: [installedBuiltin('qwen3-asr-1.7b')],
    })
    render(<LanguageSetupCard />)
    const card = await screen.findByTestId('language-setup-card')
    expect(
      within(card).getByText('Qwen 3.5 4B for AI polish. It runs on this Mac; 2.7 GB to download.'),
    ).toBeInTheDocument()
    fireEvent.click(within(card).getByRole('button', { name: 'Set up' }))
    expect(window.location.hash).toBe('#/settings?pane=llm')
  })
})
