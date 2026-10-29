import { afterEach, describe, expect, it, vi } from 'vitest'
import { openSettingsPane, parseHash, settingsPaneHash } from '../router'

afterEach(() => {
  window.location.hash = ''
})

describe('desktop route parsing', () => {
  it('keeps direct recovery links inside Settings', () => {
    window.location.hash = '#/settings?pane=stt'
    expect(parseHash()).toBe('settings')
    window.location.hash = '#/settings?pane=llm'
    expect(parseHash()).toBe('settings')
  })

  it('routes the Dictionary tab and the About page', () => {
    window.location.hash = '#/dictionary'
    expect(parseHash()).toBe('dictionary')
    window.location.hash = '#/about'
    expect(parseHash()).toBe('about')
  })

  it('sends the removed History route to Home', () => {
    window.location.hash = '#/history'
    expect(parseHash()).toBe('home')
    window.location.hash = '#/'
    expect(parseHash()).toBe('home')
  })

  it('builds links to a Settings section', () => {
    expect(settingsPaneHash('general')).toBe('#/settings?pane=general')
  })

  // Plan `languages-in-general`: Translate and the built-in pages link to General. A tab click
  // does not change the address, so it may already name the pane.
  it('opens a Settings section even when the address already names it', () => {
    const changed = vi.fn()
    window.addEventListener('hashchange', changed)
    window.location.hash = '#/settings?pane=general'
    changed.mockClear()

    openSettingsPane('general')
    expect(window.location.hash).toBe('#/settings?pane=general')
    expect(changed).toHaveBeenCalledTimes(1)
    window.removeEventListener('hashchange', changed)
  })
})
