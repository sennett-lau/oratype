import { useState, useEffect, useCallback } from 'react'

/** Main-window pages: the three sidebar tabs plus About, pinned at the bottom of the sidebar. */
export type Route = 'home' | 'settings' | 'dictionary' | 'about'

export function parseHash(): Route {
  const hash = window.location.hash.replace('#/', '')
  if (hash === 'dictionary' || hash === 'about') return hash
  if (hash === 'settings' || hash.startsWith('settings?')) return 'settings'
  return 'home'
}

export function useRoute() {
  const [route, setRoute] = useState<Route>(parseHash)

  useEffect(() => {
    const onHashChange = () => setRoute(parseHash())
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  const navigate = useCallback((r: Route) => {
    window.location.hash = r === 'home' ? '#/' : `#/${r}`
  }, [])

  return { route, navigate }
}

/** Open a Settings section directly, e.g. from a Home row. */
export function settingsPaneHash(pane: string): string {
  return `#/settings?pane=${pane}`
}

/**
 * Opens a Settings section from inside Settings (a link in one pane to another). A tab click does
 * not change the address, so it may already name the pane; the change is then announced anyway.
 */
export function openSettingsPane(pane: string) {
  const hash = settingsPaneHash(pane)
  if (window.location.hash === hash) window.dispatchEvent(new Event('hashchange'))
  else window.location.hash = hash
}
