import { useEffect } from 'react'
import { isWindowsPlatform, useAppStore } from '../stores/appStore'
import type { Theme } from '../stores/appStore'
import { syncMainWindowBackdrop } from '../lib/tauri'

/**
 * Tells macOS / Windows native window chrome which appearance this window uses.
 */
function applyWindowTheme(theme: Theme) {
  import('@tauri-apps/api/window')
    .then(({ getCurrentWindow }) => getCurrentWindow().setTheme(theme === 'system' ? null : theme))
    .catch(() => {})
}

function isMainWindow(): boolean {
  const hash = window.location.hash
  return hash !== '#capsule' && hash !== '#ask'
}

export function useTheme() {
  const theme = useAppStore((s) => s.config.theme)

  useEffect(() => {
    const root = document.documentElement
    let cancelled = false

    function resolveIsDark(t: Theme): boolean {
      if (t === 'dark') return true
      if (t === 'light') return false
      return window.matchMedia('(prefers-color-scheme: dark)').matches
    }

    function applyBackdrop(isDark: boolean) {
      if (!isWindowsPlatform() || !isMainWindow()) {
        delete root.dataset.windowsBackdrop
        return
      }
      if (!root.dataset.windowsBackdrop) {
        root.dataset.windowsBackdrop = 'solid'
      }
      syncMainWindowBackdrop(isDark)
        .then((mode) => {
          if (!cancelled) {
            root.dataset.windowsBackdrop = mode === 'mica' ? 'mica' : 'solid'
          }
        })
        .catch(() => {
          if (!cancelled) {
            root.dataset.windowsBackdrop = 'solid'
          }
        })
    }

    const initialDark = resolveIsDark(theme)
    root.classList.toggle('dark', initialDark)
    applyWindowTheme(theme)
    applyBackdrop(initialDark)

    if (theme === 'system') {
      const mq = window.matchMedia('(prefers-color-scheme: dark)')
      const handler = (e: MediaQueryListEvent) => {
        root.classList.toggle('dark', e.matches)
        applyBackdrop(e.matches)
      }
      mq.addEventListener('change', handler)
      return () => {
        cancelled = true
        mq.removeEventListener('change', handler)
      }
    }

    return () => {
      cancelled = true
    }
  }, [theme])

  return theme
}
