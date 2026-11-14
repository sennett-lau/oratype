// Plan `language-evals`: downloads the external speech clips that are too large for git. A clip
// with `fetch` names a pinned URL (a WAV, or a .tar.gz holding it as `member`) and the sha256 of
// the clip after conversion to 16 kHz mono 16-bit WAV. Downloads are cached in output/evals/cache/.
import { createHash } from 'node:crypto'
import { spawnSync } from 'node:child_process'
import {
  createWriteStream,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, dirname, join } from 'node:path'
import { Readable } from 'node:stream'
import { pipeline } from 'node:stream/promises'

const hash = (data) => createHash('sha256').update(data).digest('hex')

/**
 * Converts a WAV file (16-bit PCM or 32-bit float, any channel count) at 16 kHz into 16 kHz mono
 * 16-bit PCM, the format the evaluations use. Channels are averaged; float samples are clipped to
 * [-1, 1] and rounded. Throws for other formats or rates (resampling is not done here).
 */
export function toWav16(buf) {
  if (
    buf.length < 12 ||
    buf.toString('ascii', 0, 4) !== 'RIFF' ||
    buf.toString('ascii', 8, 12) !== 'WAVE'
  )
    throw new Error('not a WAV file')
  let fmt = null
  let data = null
  let i = 12
  while (i + 8 <= buf.length) {
    const id = buf.toString('ascii', i, i + 4)
    const size = buf.readUInt32LE(i + 4)
    const body = buf.subarray(i + 8, Math.min(buf.length, i + 8 + size))
    if (id === 'fmt ') {
      let format = body.readUInt16LE(0)
      // WAVE_FORMAT_EXTENSIBLE: the real format is the first two bytes of the sub-format GUID.
      if (format === 0xfffe && body.length >= 26) format = body.readUInt16LE(24)
      fmt = {
        format,
        channels: body.readUInt16LE(2),
        rate: body.readUInt32LE(4),
        bits: body.readUInt16LE(14),
      }
    } else if (id === 'data') {
      data = body
    }
    i += 8 + size + (size & 1)
  }
  if (!fmt || !data) throw new Error('no fmt or data chunk')
  if (fmt.rate !== 16000) throw new Error(`${fmt.rate} Hz; only 16 kHz sources are supported`)
  const pcm16 = fmt.format === 1 && fmt.bits === 16
  const float32 = fmt.format === 3 && fmt.bits === 32
  if (!pcm16 && !float32) throw new Error(`unsupported WAV format ${fmt.format}, ${fmt.bits}-bit`)
  const width = fmt.bits / 8
  const frames = Math.floor(data.length / (width * fmt.channels))
  const out = Buffer.alloc(44 + frames * 2)
  for (let f = 0; f < frames; f += 1) {
    let sum = 0
    for (let c = 0; c < fmt.channels; c += 1) {
      const at = (f * fmt.channels + c) * width
      sum += pcm16 ? data.readInt16LE(at) / 32768 : data.readFloatLE(at)
    }
    const v = Math.max(-1, Math.min(1, sum / fmt.channels))
    out.writeInt16LE(Math.max(-32768, Math.min(32767, Math.round(v * 32767))), 44 + f * 2)
  }
  out.write('RIFF', 0, 'ascii')
  out.writeUInt32LE(36 + frames * 2, 4)
  out.write('WAVEfmt ', 8, 'ascii')
  out.writeUInt32LE(16, 16)
  out.writeUInt16LE(1, 20) // PCM
  out.writeUInt16LE(1, 22) // mono
  out.writeUInt32LE(16000, 24)
  out.writeUInt32LE(32000, 28) // bytes per second
  out.writeUInt16LE(2, 32) // block align
  out.writeUInt16LE(16, 34)
  out.write('data', 36, 'ascii')
  out.writeUInt32LE(frames * 2, 40)
  return out
}

async function download(url, path, log) {
  mkdirSync(dirname(path), { recursive: true })
  log(`evals: downloading ${url}`)
  const response = await fetch(url)
  if (!response.ok || !response.body)
    throw new Error(`download failed (${response.status}): ${url}`)
  const part = `${path}.part`
  await pipeline(Readable.fromWeb(response.body), createWriteStream(part))
  renameSync(part, path)
}

/**
 * Downloads and converts the clips that are missing (all of them with `force`), checking each
 * against its sha256. Returns the number of clips written.
 */
export async function fetchClips(clips, { cacheDir, force = false, log = console.log } = {}) {
  const todo = clips.filter((c) => c.fetch && (force || !existsSync(c.audio_path)))
  const byUrl = new Map()
  for (const clip of todo) byUrl.set(clip.fetch.url, [...(byUrl.get(clip.fetch.url) ?? []), clip])
  let written = 0
  for (const [url, group] of byUrl) {
    const cached = join(cacheDir, `${hash(url).slice(0, 12)}-${basename(new URL(url).pathname)}`)
    if (!existsSync(cached)) await download(url, cached, log)
    const members = [...new Set(group.map((c) => c.fetch.member).filter(Boolean))]
    // One pass over the archive extracts every member this group needs.
    const dir = members.length ? mkdtempSync(join(tmpdir(), 'oratype-evals-fetch-')) : null
    try {
      if (dir) {
        const result = spawnSync('tar', ['-xzf', cached, '-C', dir, ...members], {
          encoding: 'utf8',
        })
        if (result.status !== 0)
          throw new Error(`tar failed on ${basename(cached)}: ${result.stderr.trim()}`)
      }
      for (const clip of group) {
        const source = clip.fetch.member ? join(dir, clip.fetch.member) : cached
        const wav = toWav16(readFileSync(source))
        const got = hash(wav)
        if (got !== clip.fetch.sha256)
          throw new Error(`${clip.id}: sha256 ${got} does not match the manifest; source changed?`)
        mkdirSync(dirname(clip.audio_path), { recursive: true })
        writeFileSync(clip.audio_path, wav)
        written += 1
      }
    } finally {
      if (dir) rmSync(dir, { recursive: true, force: true })
    }
  }
  return written
}
