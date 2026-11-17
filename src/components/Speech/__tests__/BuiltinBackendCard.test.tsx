import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/react'
import { BuiltinBackendCard } from '../BuiltinBackendCard'
import * as tauri from '../../../lib/tauri'
import { useAppStore } from '../../../stores/appStore'

vi.mock('../../../lib/tauri')

vi.mock('react-i18next', async () => {
  const { translate } = await import('../../../test-utils/i18nMock')
  return { useTranslation: () => ({ t: translate }) }
})

const WINDOWS_INFO: tauri.BuiltinBackendInfo = {
  platform: 'windows',
  llamaEngine: 'llama.cpp b11177 (Vulkan)',
  whisperEngine: 'whisper.cpp (Vulkan)',
  devices: [
    {
      id: 'Vulkan0',
      description: 'AMD Radeon(TM) Graphics',
      totalMib: 512,
      freeMib: 450,
    },
    {
      id: 'Vulkan1',
      description: 'NVIDIA GeForce RTX 4090',
      totalMib: 24142,
      freeMib: 23400,
    },
  ],
  autoDeviceId: 'Vulkan1',
  activeAiDevice: 'Vulkan1 · NVIDIA GeForce RTX 4090 (24 GB)',
  activeSpeechLlamaDevice: 'Vulkan1 · NVIDIA GeForce RTX 4090 (24 GB)',
  activeWhisperDevice: 'Vulkan1 · NVIDIA GeForce RTX 4090 (24 GB)',
}

describe('BuiltinBackendCard', () => {
  const updateConfig = vi.fn().mockResolvedValue(undefined)

  afterEach(() => cleanup())

  beforeEach(() => {
    vi.clearAllMocks()
    useAppStore.setState({
      updateConfig,
      config: {
        ...useAppStore.getInitialState().config,
        builtin_ai_backend: 'auto',
        builtin_speech_backend: 'auto',
      },
    } as any)
    vi.mocked(tauri.getBuiltinBackendInfo).mockResolvedValue(WINDOWS_INFO)
  })

  it('displays llama engine and RTX 4090 active device for AI and updates config on change', async () => {
    render(<BuiltinBackendCard kind="ai" />)

    expect(await screen.findByTestId('builtin-ai-engine')).toHaveTextContent(
      'llama.cpp b11177 (Vulkan)',
    )
    expect(screen.getByTestId('builtin-ai-device')).toHaveTextContent(
      'Vulkan1 · NVIDIA GeForce RTX 4090 (24 GB)',
    )

    const select = screen.getByRole('combobox', { name: 'Backend' }) as HTMLSelectElement
    expect(select.value).toBe('auto')
    expect(screen.getByRole('option', { name: /NVIDIA GeForce RTX 4090/ })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'CPU only' })).toBeInTheDocument()

    fireEvent.change(select, { target: { value: 'cpu' } })
    expect(updateConfig).toHaveBeenCalledWith({ builtin_ai_backend: 'cpu' })
  })

  it('shows nothing on a Mac, which has no device to choose', async () => {
    vi.mocked(tauri.getBuiltinBackendInfo).mockResolvedValue({
      ...WINDOWS_INFO,
      platform: 'macos',
      llamaEngine: 'llama.cpp b11177 (Metal)',
      whisperEngine: 'whisper.cpp (Metal)',
      devices: [],
      autoDeviceId: null,
      activeAiDevice: 'Apple Metal GPU',
      activeSpeechLlamaDevice: 'Apple Metal GPU',
      activeWhisperDevice: 'Apple Metal GPU',
    })
    const { container } = render(<BuiltinBackendCard kind="speech" />)

    await waitFor(() => expect(tauri.getBuiltinBackendInfo).toHaveBeenCalled())
    await Promise.resolve()
    expect(container).toBeEmptyDOMElement()
  })

  it('displays whisper and llama engines for Speech and switches between Vulkan GPU and CPU', async () => {
    render(<BuiltinBackendCard kind="speech" />)

    expect(await screen.findByTestId('builtin-speech-engine')).toHaveTextContent(
      'whisper.cpp (Vulkan) / llama.cpp b11177 (Vulkan)',
    )
    expect(screen.getByTestId('builtin-speech-device')).toHaveTextContent(
      'Vulkan1 · NVIDIA GeForce RTX 4090 (24 GB)',
    )

    const select = screen.getByRole('combobox', { name: 'Backend' }) as HTMLSelectElement
    fireEvent.change(select, { target: { value: 'Vulkan1' } })
    expect(updateConfig).toHaveBeenCalledWith({ builtin_speech_backend: 'Vulkan1' })
  })
})
