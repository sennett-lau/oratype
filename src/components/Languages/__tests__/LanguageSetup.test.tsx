import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as tauri from '../../../lib/tauri'
import { useAppStore } from '../../../stores/appStore'
import { IDLE_SETUP_STATUS, useSpeechSetupStore } from '../../../stores/speechSetupStore'
import { useAiSetupStore } from '../../../stores/aiSetupStore'
import { hardwareCheck, installedBuiltin, serverPreset } from '../../../test-utils/speechHardware'
import { LanguagesStep } from '../../Onboarding/LanguagesStep'
import { LanguageSetupCard } from '../../HomePage/LanguageSetupCard'
import { BuiltinSettings } from '../../Speech/BuiltinSettings'
import { AI_SERVICE, SPEECH_SERVICE } from '../../Speech/services'

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
  vi.mocked(tauri.setYourLanguages).mockImplementation(async (languages) => languages)
  vi.mocked(tauri.dismissLanguageSetup).mockResolvedValue(undefined)
  window.location.hash = '#/'
})

const originalLanguages = Object.getOwnPropertyDescriptor(window.navigator, 'languages')

afterEach(() => {
  cleanup()
  if (originalLanguages) Object.defineProperty(window.navigator, 'languages', originalLanguages)
})

describe('Onboarding → Your languages (plan `language-model-suggestions`)', () => {
  it('shows the suggested setup for a combination with Cantonese', async () => {
    setConfig({ your_languages: ['en', 'zh-Hant-HK'] })
    render(<LanguagesStep />)

    const box = await screen.findByTestId('setup-suggestion')
    expect(
      within(box).getByText('Suggested setup for English and Cantonese (Hong Kong)'),
    ).toBeInTheDocument()
    expect(within(box).getByText('Qwen3-ASR 1.7B')).toBeInTheDocument()
    expect(within(box).getByText('Qwen 3.5 4B')).toBeInTheDocument()
    expect(within(box).getByText('2.5 GB')).toBeInTheDocument()
    expect(within(box).getByText('2.7 GB')).toBeInTheDocument()
    // The list is the user's own choice, so no "from your Mac" note.
    expect(screen.queryByText(/Filled in from your Mac/)).not.toBeInTheDocument()
  })

  it('says when there is no suggested setup for the languages', async () => {
    setConfig({ your_languages: ['en', 'ja'] })
    render(<LanguagesStep />)
    const box = await screen.findByTestId('setup-suggestion')
    expect(within(box).getByText('No suggested setup for English and 日本語')).toBeInTheDocument()
  })

  // Plan `languages-in-general`: Mandarin is not Cantonese, whichever characters.
  it('has no Cantonese setup for Mandarin', async () => {
    setConfig({ your_languages: ['zh-Hant-TW', 'zh-Hans'] })
    render(<LanguagesStep />)
    const box = await screen.findByTestId('setup-suggestion')
    expect(
      within(box).getByText(
        'No suggested setup for Mandarin (Traditional) and Mandarin (Simplified)',
      ),
    ).toBeInTheDocument()
  })

  it('saves each change at once, as written codes', async () => {
    setConfig({ your_languages: ['en', 'zh-Hant-HK'] })
    render(<LanguagesStep />)

    fireEvent.click(screen.getByRole('button', { name: 'Remove Cantonese (Hong Kong)' }))
    await waitFor(() => expect(tauri.setYourLanguages).toHaveBeenCalledWith(['en']))
    await waitFor(() => expect(useAppStore.getState().config.your_languages).toEqual(['en']))
    expect(useAppStore.getState().savedConfig?.your_languages).toEqual(['en'])

    const add = screen.getByRole('combobox', { name: 'Add a language you speak' })
    expect(within(add).getByRole('option', { name: 'Mandarin (Simplified)' })).toBeInTheDocument()
    fireEvent.change(add, { target: { value: 'zh-Hans' } })
    await waitFor(() => expect(tauri.setYourLanguages).toHaveBeenLastCalledWith(['en', 'zh-Hans']))
  })

  it('starts from the Mac languages until the user chooses', async () => {
    Object.defineProperty(window.navigator, 'languages', {
      value: ['en-GB', 'zh-HK'],
      configurable: true,
    })
    setConfig({ your_languages: null })
    render(<LanguagesStep />)
    expect(await screen.findByText(/Filled in from your Mac/)).toBeInTheDocument()
    expect(screen.getByText('Cantonese (Hong Kong)')).toBeInTheDocument()
    expect(await screen.findByTestId('setup-suggestion')).toHaveTextContent(
      'Suggested setup for English and Cantonese (Hong Kong)',
    )
  })
})

describe('Home → Suggested for your languages', () => {
  it('offers the models of the setup that are not in use, and opens Settings', async () => {
    setConfig({ your_languages: ['en', 'zh-Hant-HK'] })
    render(<LanguageSetupCard />)

    const card = await screen.findByTestId('language-setup-card')
    expect(
      within(card).getByText('Built-in models for English and Cantonese (Hong Kong)'),
    ).toBeInTheDocument()
    expect(
      within(card).getByText(
        'Qwen3-ASR 1.7B for speech recognition and Qwen 3.5 4B for AI polish. They run on this Mac; 5.3 GB to download.',
      ),
    ).toBeInTheDocument()
    fireEvent.click(within(card).getByRole('button', { name: 'Set up' }))
    expect(window.location.hash).toBe('#/settings?pane=stt')
  })

  it('hides for good with "Don’t show again"', async () => {
    setConfig({ your_languages: ['zh-Hant-HK'] })
    render(<LanguageSetupCard />)
    const card = await screen.findByTestId('language-setup-card')
    fireEvent.click(within(card).getByRole('button', { name: 'Don’t show again' }))

    await waitFor(() => expect(tauri.dismissLanguageSetup).toHaveBeenCalledWith('cantonese'))
    await waitFor(() => expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument())
  })

  it('stays away without a setup, when dismissed, or when the user has their own server', async () => {
    setConfig({ your_languages: ['en'] })
    const { unmount } = render(<LanguageSetupCard />)
    await waitFor(() => expect(tauri.getAiHardware).toHaveBeenCalled())
    expect(screen.queryByTestId('language-setup-card')).not.toBeInTheDocument()
    unmount()

    setConfig({ your_languages: ['zh-Hant-HK'], dismissed_language_setup: 'cantonese' })
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
      your_languages: ['zh-Hant-HK'],
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

// Plan `languages-in-general`: the languages are chosen in Settings → General; the built-in
// pages keep only the suggestion.
describe('Settings → Built-in speech and AI polish', () => {
  it.each([
    [SPEECH_SERVICE, 'Speech recognition: Qwen3-ASR 1.7B.'],
    [AI_SERVICE, 'AI polish: Qwen 3.5 4B.'],
  ])('has no language picker, and its banner opens General (%#)', async (service, detail) => {
    setConfig({ your_languages: ['en', 'zh-Hant-HK'] })
    window.location.hash = `#/settings?pane=${service.id === 'speech' ? 'stt' : 'llm'}`
    render(<BuiltinSettings service={service} />)

    const banner = await screen.findByTestId('setup-banner')
    expect(
      within(banner).getByText('Suggested setup for English and Cantonese (Hong Kong)'),
    ).toBeInTheDocument()
    expect(within(banner).getByText(detail)).toBeInTheDocument()
    expect(screen.queryByText('Your languages')).not.toBeInTheDocument()
    expect(
      screen.queryByRole('combobox', { name: 'Add a language you speak' }),
    ).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: /^Remove/ })).not.toBeInTheDocument()

    fireEvent.click(within(banner).getByRole('button', { name: 'Change your languages' }))
    expect(window.location.hash).toBe('#/settings?pane=general')
  })

  it('shows no banner when the languages have no suggested setup', async () => {
    setConfig({ your_languages: ['en', 'ja'] })
    render(<BuiltinSettings service={SPEECH_SERVICE} />)
    await waitFor(() => expect(tauri.getSpeechHardware).toHaveBeenCalled())
    expect(screen.getByTestId('builtin-settings')).toBeInTheDocument()
    expect(screen.queryByTestId('setup-banner')).not.toBeInTheDocument()
    expect(screen.queryByText('Your languages')).not.toBeInTheDocument()
  })
})
