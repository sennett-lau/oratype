import { cleanup, renderHook, waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useTheme } from '../useTheme'
import { useAppStore } from '../../stores/appStore'
import * as tauri from '../../lib/tauri'

describe('useTheme Windows backdrop sync', () => {
  const originalPlatformDescriptor = Object.getOwnPropertyDescriptor(window.navigator, 'platform')

  function setNavigatorPlatform(value: string) {
    Object.defineProperty(window.navigator, 'platform', {
      value,
      configurable: true,
    })
  }

  beforeEach(() => {
    useAppStore.setState(useAppStore.getInitialState(), true)
    document.documentElement.className = ''
    delete document.documentElement.dataset.windowsBackdrop
    window.location.hash = '#/'
    vi.stubGlobal(
      'matchMedia',
      vi.fn().mockImplementation((query: string) => ({
        matches: false,
        media: query,
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
      })),
    )
  })

  afterEach(() => {
    cleanup()
    vi.restoreAllMocks()
    vi.unstubAllGlobals()
    window.location.hash = ''
    delete document.documentElement.dataset.windowsBackdrop
    document.documentElement.className = ''
    if (originalPlatformDescriptor) {
      Object.defineProperty(window.navigator, 'platform', originalPlatformDescriptor)
    }
  })

  it('syncs dark backdrop on Windows main window and applies returned mica mode', async () => {
    setNavigatorPlatform('Win32')
    const spy = vi.spyOn(tauri, 'syncMainWindowBackdrop').mockResolvedValue('mica')
    useAppStore.setState((s) => ({ config: { ...s.config, theme: 'dark' } }))

    renderHook(() => useTheme())

    await waitFor(() => {
      expect(spy).toHaveBeenCalledWith(true)
      expect(document.documentElement.dataset.windowsBackdrop).toBe('mica')
    })
    expect(document.documentElement.classList.contains('dark')).toBe(true)
  })

  it('falls back to solid when backend returns solid on Windows 10', async () => {
    setNavigatorPlatform('Win32')
    vi.spyOn(tauri, 'syncMainWindowBackdrop').mockResolvedValue('solid')
    useAppStore.setState((s) => ({ config: { ...s.config, theme: 'light' } }))

    renderHook(() => useTheme())

    await waitFor(() => {
      expect(document.documentElement.dataset.windowsBackdrop).toBe('solid')
    })
  })

  it('keeps capsule, ask, and macOS windows untouched without data-windows-backdrop', () => {
    const spy = vi.spyOn(tauri, 'syncMainWindowBackdrop').mockResolvedValue('mica')

    setNavigatorPlatform('Win32')
    window.location.hash = '#capsule'
    renderHook(() => useTheme())
    expect(spy).not.toHaveBeenCalled()
    expect(document.documentElement.dataset.windowsBackdrop).toBeUndefined()

    window.location.hash = '#ask'
    renderHook(() => useTheme())
    expect(spy).not.toHaveBeenCalled()
    expect(document.documentElement.dataset.windowsBackdrop).toBeUndefined()

    window.location.hash = '#/'
    setNavigatorPlatform('MacIntel')
    renderHook(() => useTheme())
    expect(spy).not.toHaveBeenCalled()
    expect(document.documentElement.dataset.windowsBackdrop).toBeUndefined()
  })
})
