// Stills of the real app UI for a release's promo video (see .claude/skills/promo-video).
//
// The same harness as the README screenshots (site/capture-app): the app's own React components
// with a mocked Tauri backend, in headless Chromium. Promo stills are dark and twice the size the
// film shows them, so a slow push-in stays sharp.
//
//   node site/scripts/capture-promo-stills.mjs <version> [shot ...]
//   node site/scripts/capture-promo-stills.mjs 1.2.0 hands-free pill-recording
//
// Writes output/promo-<version>/stills/<shot>.png. Without shots, every shot in PROMO_SHOTS is
// captured. A local tool: run `npm ci` at the repository root first, and have Chromium installed
// (or set CHROMIUM).

import { mkdir } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { launch } from './lib/chromium.mjs'
import { optimise, startServer } from './capture-app-screens.mjs'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..')

/**
 * Every still a promo can use. `screen` is a harness screen (site/capture-app/mocks.ts),
 * `state` the pill state for the `pill` screen, and `scrollTo` an element the settings page is
 * scrolled to (its top lands 64 px below the window's top). Add a shot here when a release needs
 * a new one, and its screen to the harness.
 */
export const PROMO_SHOTS = {
  home: { screen: 'home', width: 1000, height: 820, scale: 2 },
  'hands-free': {
    screen: 'promo-hands-free',
    width: 1000,
    height: 820,
    scale: 2,
    scrollTo: '[role="switch"][aria-label="Hands-free mode"]',
  },
  'fast-switching': {
    screen: 'promo-fast-switching',
    width: 1000,
    height: 820,
    scale: 2,
    scrollTo: '[role="switch"][aria-label="Fast language switching"]',
  },
  'your-languages': {
    screen: 'promo-your-languages',
    width: 1000,
    height: 820,
    scale: 2,
    scrollTo: '[data-testid="your-languages"]',
  },
  'pill-recording': { screen: 'pill', state: 'recording', width: 500, height: 150, scale: 4 },
  'pill-transcribing': { screen: 'pill', state: 'transcribing', width: 500, height: 150, scale: 4 },
  'pill-polishing': { screen: 'pill', state: 'polishing', width: 500, height: 150, scale: 4 },
  'pill-ask': { screen: 'pill', state: 'ask_recording', width: 500, height: 150, scale: 4 },
}

const sleep = (ms) => new Promise((done) => setTimeout(done, ms))

/** Scrolls the nearest scrolling parent of `selector` so the element sits near the top. */
function scrollIntoPlace(selector) {
  return `(() => {
    const target = document.querySelector(${JSON.stringify(selector)})
    let scroller = target?.parentElement
    while (scroller && scroller.scrollHeight <= scroller.clientHeight) scroller = scroller.parentElement
    if (!target || !scroller) return false
    const top = target.getBoundingClientRect().top - scroller.getBoundingClientRect().top
    scroller.scrollTop += top - 64
    return true
  })()`
}

export async function capturePromoStills({ browser, version, shots }) {
  const outDir = join(repoRoot, 'output', `promo-${version}`, 'stills')
  await mkdir(outDir, { recursive: true })
  const { server, url } = await startServer()
  const written = []
  try {
    for (const name of shots) {
      const shot = PROMO_SHOTS[name]
      if (!shot)
        throw new Error(`Unknown shot "${name}"; known: ${Object.keys(PROMO_SHOTS).join(', ')}`)
      const page = await browser.newPage({
        width: shot.width,
        height: shot.height,
        scale: shot.scale,
        dark: true,
      })
      const state = shot.state ? `&state=${shot.state}` : ''
      await page.goto(`${url}?screen=${shot.screen}&theme=dark${state}`)
      await page.waitFor('window.__captureReady === true', 60000)
      if (shot.scrollTo) {
        const scrolled = await page.evaluate(scrollIntoPlace(shot.scrollTo))
        if (!scrolled) console.warn(`[${name}] ${shot.scrollTo} not found; not scrolled`)
        await sleep(400)
      }
      const file = join(outDir, `${name}.png`)
      await page.screenshot(file, { selector: '#stage' })
      optimise(file)
      written.push(file)
      const logs = await page.evaluate('window.__captureLogs ?? []')
      if (logs.length) console.warn(`[${name}]\n  ${logs.join('\n  ')}`)
      page.close()
    }
  } finally {
    await server.close()
  }
  return written
}

// Run directly: node site/scripts/capture-promo-stills.mjs <version> [shot ...]
if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  const [version, ...requested] = process.argv.slice(2)
  if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
    console.error('Usage: node site/scripts/capture-promo-stills.mjs <version> [shot ...]')
    console.error(`Shots: ${Object.keys(PROMO_SHOTS).join(', ')}`)
    process.exit(1)
  }
  const browser = await launch()
  try {
    const shots = requested.length ? requested : Object.keys(PROMO_SHOTS)
    for (const file of await capturePromoStills({ browser, version, shots })) console.log(file)
  } finally {
    await browser.close()
  }
}
