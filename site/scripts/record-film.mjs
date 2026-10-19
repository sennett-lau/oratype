// Records a release's promo film into an MP4 (see .claude/skills/promo-video).
//
//   node site/scripts/record-film.mjs <version>                  # the whole film
//   node site/scripts/record-film.mjs <version> --preview 1,4.5  # only these seconds
//
// Reads output/promo-<version>/film.html, which defines:
//   window.DURATION  the film's length in seconds
//   window.render(t) draws the frame at t seconds (pure: the same t gives the same frame)
//   window.ready     a promise that resolves once its images have decoded
// Each frame is rendered and screenshot at 1920×1080 and 30 fps, so the film never depends on
// real-time playback. Then ffmpeg encodes oratype-<version>-promo.mp4 with music.wav from the
// same folder (made by site/scripts/promo-music.py), writes sheet.png (one frame per second, to
// check the film at a glance) and removes the frames. A preview writes preview/t<seconds>.png.
//
// Needs Chromium (or CHROMIUM) and ffmpeg.

import { execFileSync } from 'node:child_process'
import { existsSync } from 'node:fs'
import { mkdir, rm } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import { launch } from './lib/chromium.mjs'

const repoRoot = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const FPS = 30
const WIDTH = 1920
const HEIGHT = 1080

const [version, flag, list] = process.argv.slice(2)
if (!version || !/^\d+\.\d+\.\d+$/.test(version)) {
  console.error('Usage: node site/scripts/record-film.mjs <version> [--preview 1,4.5]')
  process.exit(1)
}
const dir = join(repoRoot, 'output', `promo-${version}`)
const film = join(dir, 'film.html')
if (!existsSync(film)) {
  console.error(`${film} is missing; start from .claude/skills/promo-video/film-template.html`)
  process.exit(1)
}
const preview = flag === '--preview' ? list.split(',').map(Number) : null

const browser = await launch()
let frames = 0
try {
  const page = await browser.newPage({ width: WIDTH, height: HEIGHT, scale: 1, dark: true })
  await page.goto(pathToFileURL(film).href)
  await page.evaluate('window.ready')
  const duration = Number(await page.evaluate('window.DURATION'))
  if (!(duration > 0)) throw new Error('film.html must set window.DURATION (seconds)')
  const times = preview ?? Array.from({ length: Math.round(duration * FPS) }, (_, i) => i / FPS)
  const outDir = join(dir, preview ? 'preview' : 'frames')
  await rm(outDir, { recursive: true, force: true })
  await mkdir(outDir, { recursive: true })
  for (const [i, t] of times.entries()) {
    // Wait until every visible image of the new frame has decoded.
    await page.evaluate(
      `(render(${t}), Promise.all([...document.images].filter(i => i.offsetParent).map(i => i.decode().catch(() => {}))))`,
    )
    const name = preview ? `t${t}.png` : `${String(i).padStart(4, '0')}.png`
    await page.screenshot(join(outDir, name))
  }
  frames = times.length
  console.log(`${frames} frame(s) in ${outDir}`)
} finally {
  await browser.close()
}

if (!preview) {
  const video = join(dir, `oratype-${version}-promo.mp4`)
  const music = join(dir, 'music.wav')
  const audio = existsSync(music) ? ['-i', music, '-c:a', 'aac', '-b:a', '192k', '-shortest'] : []
  if (!audio.length) console.warn('No music.wav: the film is silent (site/scripts/promo-music.py)')
  execFileSync(
    'ffmpeg',
    [
      '-loglevel',
      'error',
      '-y',
      '-framerate',
      String(FPS),
      '-i',
      join(dir, 'frames/%04d.png'),
      ...audio.slice(0, 2),
      '-c:v',
      'libx264',
      '-pix_fmt',
      'yuv420p',
      '-crf',
      '16',
      '-preset',
      'slow',
      '-tune',
      'stillimage',
      ...audio.slice(2),
      '-movflags',
      '+faststart',
      video,
    ],
    { stdio: 'inherit' },
  )
  // One frame per second, five across.
  const rows = Math.ceil(frames / FPS / 5)
  execFileSync(
    'ffmpeg',
    [
      '-loglevel',
      'error',
      '-y',
      '-i',
      video,
      '-vf',
      `fps=1,scale=480:-1,tile=5x${rows}:padding=8:color=black`,
      '-frames:v',
      '1',
      join(dir, 'sheet.png'),
    ],
    { stdio: 'inherit' },
  )
  await rm(join(dir, 'frames'), { recursive: true, force: true })
  console.log(video)
}
