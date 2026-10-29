import { cleanup, fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import * as tauri from '../../../lib/tauri'
import type { PresetDetail } from '../../../lib/tauri'
import { useAppStore, type AppConfig } from '../../../stores/appStore'
import { YourLanguagesGroup } from '../YourLanguagesGroup'
import { resetTranslationDefaultsCache } from '../translationLanguages'

vi.mock('../../../lib/tauri')
vi.mock('@tauri-apps/plugin-opener', () => ({ openUrl: vi.fn().mockResolvedValue(undefined) }))
vi.mock('react-i18next', async () => {
  const { translate } = await import('../../../test-utils/i18nMock')
  return { useTranslation: () => ({ t: translate, i18n: { language: 'en' } }) }
})

const SHA = 'a'.repeat(64)

const CANTONESE: PresetDetail = {
  id: 'cantonese-hong-kong',
  name: 'Cantonese (Hong Kong) 廣東話',
  tier: 'official',
  summary: 'Colloquial written Cantonese.',
  version: 4,
  sha256: SHA,
  authors: ['sennett-lau'],
  model_hint: null,
  text: 'Written Cantonese…',
  variant: null,
  detect_codes: ['yue', 'zh'],
  hints: ['嘅'],
  require_hint: true,
  applies_to: ['polish', 'translate'],
}

/** Your languages and a Translate list that differs from them. */
function setup(yourLanguages: string[] | null) {
  const base = useAppStore.getState().config
  const config: AppConfig = {
    ...base,
    your_languages: yourLanguages,
    translation: {
      targets: ['ja'],
      active_target: 'ja',
      languages: {
        'zh-Hant-HK': {
          instructions: null,
          library_preset: { id: 'cantonese-hong-kong', version: 4, sha256: SHA },
        },
      },
    },
  }
  useAppStore.setState({ config, savedConfig: config })
}

function row(code: string) {
  return screen.getByTestId(`language-row-${code}`)
}

const originalLanguages = Object.getOwnPropertyDescriptor(window.navigator, 'languages')

beforeEach(() => {
  useAppStore.setState(useAppStore.getInitialState())
  resetTranslationDefaultsCache()
  vi.mocked(tauri.setYourLanguages).mockImplementation(async (languages) => languages)
  vi.mocked(tauri.loadLanguagePreset).mockResolvedValue(CANTONESE)
  vi.mocked(tauri.getLanguageLibraryStatus).mockResolvedValue({ latest: {}, updates: {} })
  vi.mocked(tauri.getTranslationLanguageDefaults).mockResolvedValue({})
  vi.mocked(tauri.updateConfig).mockResolvedValue(undefined)
})

afterEach(() => {
  cleanup()
  vi.clearAllMocks()
  if (originalLanguages) Object.defineProperty(window.navigator, 'languages', originalLanguages)
})

// Plan `languages-in-general`: Settings → General → Your languages.
describe('Settings → General → Your languages', () => {
  it('shows each language by its spoken name with its preset, in no particular order', async () => {
    setup(['en', 'zh-Hant-HK'])
    render(<YourLanguagesGroup />)

    const group = screen.getByRole('region', { name: 'Your languages' })
    const list = within(group).getByRole('list', { name: 'Your languages' })
    expect(within(list).getAllByRole('listitem')).toHaveLength(2)
    expect(within(row('zh-Hant-HK')).getByText('Cantonese (Hong Kong)')).toBeInTheDocument()
    await waitFor(() =>
      expect(
        within(row('zh-Hant-HK')).getByText('Cantonese (Hong Kong) 廣東話 preset · v4'),
      ).toBeInTheDocument(),
    )
    expect(within(row('en')).getByText('Built-in instructions')).toBeInTheDocument()
    // Not the Translate list: no drag handle, no number, and Japanese is not shown.
    expect(screen.queryByRole('button', { name: /^Move / })).not.toBeInTheDocument()
    expect(within(list).queryByText('1')).not.toBeInTheDocument()
    expect(screen.queryByTestId('language-row-ja')).not.toBeInTheDocument()
    expect(within(group).getByText('up to 12')).toBeInTheDocument()
    expect(
      within(group).getByText(/OraType picks the language you spoke from this list/),
    ).toBeInTheDocument()
  })

  it('adds a language at once and opens its sheet to choose a preset', async () => {
    setup(['en'])
    render(<YourLanguagesGroup />)

    fireEvent.click(screen.getByRole('button', { name: /Add language/ }))
    const menu = screen.getByRole('combobox', { name: 'Add language' })
    expect(within(menu).getByRole('option', { name: 'Cantonese (Hong Kong)' })).toBeInTheDocument()
    expect(within(menu).queryByRole('option', { name: 'English' })).not.toBeInTheDocument()
    fireEvent.change(menu, { target: { value: 'zh-Hant-HK' } })

    await waitFor(() => expect(tauri.setYourLanguages).toHaveBeenCalledWith(['en', 'zh-Hant-HK']))
    await waitFor(() =>
      expect(useAppStore.getState().savedConfig?.your_languages).toEqual(['en', 'zh-Hant-HK']),
    )
    // Saved at once: nothing for the Save bar, and Translate's list is unchanged.
    expect(tauri.updateConfig).not.toHaveBeenCalled()
    expect(useAppStore.getState().config.translation.targets).toEqual(['ja'])
    expect(screen.getByRole('dialog', { name: 'Cantonese (Hong Kong)' })).toBeInTheDocument()
  })

  it('removes any language, even the last one', async () => {
    setup(['zh-Hant-HK'])
    render(<YourLanguagesGroup />)

    fireEvent.click(screen.getByRole('button', { name: 'Remove Cantonese (Hong Kong)' }))
    await waitFor(() => expect(tauri.setYourLanguages).toHaveBeenCalledWith([]))
    await waitFor(() => expect(screen.queryAllByRole('listitem')).toHaveLength(0))
  })

  it('edits the preset shared with Translate', async () => {
    setup(['zh-Hant-HK'])
    render(<YourLanguagesGroup />)

    fireEvent.click(screen.getByRole('button', { name: 'Edit Cantonese (Hong Kong)' }))
    expect(screen.getByRole('dialog', { name: 'Cantonese (Hong Kong)' })).toBeInTheDocument()

    // The switch writes the language's own settings, the ones Translate uses too.
    fireEvent.click(
      within(row('zh-Hant-HK')).getByRole('switch', {
        name: 'Use instructions for Cantonese (Hong Kong)',
      }),
    )
    await waitFor(() => expect(tauri.updateConfig).toHaveBeenCalled())
    const saved = vi.mocked(tauri.updateConfig).mock.calls[0][0]
    expect(saved.translation.languages?.['zh-Hant-HK']?.enabled).toBe(false)
    expect(saved.translation.languages?.['zh-Hant-HK']?.library_preset?.id).toBe(
      'cantonese-hong-kong',
    )
  })

  it('offers the Mac languages until the user has chosen', async () => {
    Object.defineProperty(window.navigator, 'languages', {
      value: ['en-HK', 'zh-Hant-HK', 'en-US'],
      configurable: true,
    })
    setup(null)
    render(<YourLanguagesGroup />)

    expect(screen.queryAllByRole('listitem')).toHaveLength(0)
    fireEvent.click(screen.getByRole('button', { name: 'Add English and Cantonese (Hong Kong)' }))
    await waitFor(() => expect(tauri.setYourLanguages).toHaveBeenCalledWith(['en', 'zh-Hant-HK']))
    await waitFor(() => expect(screen.getAllByRole('listitem')).toHaveLength(2))
    expect(screen.queryByRole('button', { name: /^Add English/ })).not.toBeInTheDocument()
  })

  it('does not offer the Mac languages once the user has chosen, even none', () => {
    setup([])
    render(<YourLanguagesGroup />)
    expect(screen.queryByRole('button', { name: /^Add English/ })).not.toBeInTheDocument()
    expect(screen.getByRole('button', { name: /Add language/ })).toBeInTheDocument()
  })
})
