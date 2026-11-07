import { describe, expect, it } from 'vitest'

// framer-motion's `repeat: Infinity` kept running after its element unmounted: the pill's
// recording dot left one more endless animation behind every dictation, and after a week the
// pill's page spent a third of each frame on them and used over a gigabyte. Endless loops are
// CSS animations instead (`.pill-dot-pulse`, Tailwind's `animate-spin`), which stop with
// their element.
const sources = import.meta.glob<string>(['../../../**/*.{ts,tsx}', '!**/__tests__/**'], {
  query: '?raw',
  import: 'default',
  eager: true,
})

describe('endless animations', () => {
  it('finds the app sources', () => {
    expect(Object.keys(sources).length).toBeGreaterThan(50)
  })

  it('never uses framer-motion repeat: Infinity', () => {
    const offenders = Object.entries(sources)
      .filter(([, text]) => /repeat:\s*Infinity/.test(text))
      .map(([path]) => path)
    expect(offenders).toEqual([])
  })
})
