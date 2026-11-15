// Plan `language-evals`: loads and validates the evaluation dataset in evals/.
import { createHash } from 'node:crypto'
import { existsSync, readdirSync, readFileSync, statSync } from 'node:fs'
import { dirname, join, relative } from 'node:path'
import { fileURLToPath } from 'node:url'

import { contains, scorePolish } from './score.mjs'

export const REPO = join(dirname(fileURLToPath(import.meta.url)), '..', '..')
export const EVALS = join(REPO, 'evals')

export const POLISH_TAGS = [
  'filler',
  'self-correction',
  'numbers',
  'punctuation',
  'newline',
  'list',
  'dash',
  'keep-meaning',
  'script',
  'dialect',
  'mixed-language',
  'no-answer',
  'unfinished',
  'over-punctuation',
  'cursor-context',
]
export const SPEECH_TAGS = [
  'clean',
  'numbers',
  'names',
  'dialect',
  'mixed-language',
  'noisy',
  'accent',
  'dictionary',
]
export const SPLITS = ['dev', 'holdout']
export const LICENCES = ['CC0-1.0', 'CC-BY-4.0']
export const SPEECH_SOURCES = ['synthetic-say', 'recording', 'external']
const ID = /^[a-z0-9]+(-[a-z0-9]+)*$/
const POLISH_KEYS = {
  required: ['id', 'input', 'expected', 'tags', 'split'],
  optional: ['accept', 'must_contain', 'must_not_contain', 'notes', 'max_error', 'before_cursor'],
}
const SPEECH_KEYS = {
  required: ['id', 'audio', 'text', 'tags', 'source', 'licence', 'speaker', 'split'],
  optional: [
    'accept',
    'notes',
    'max_error',
    'say_voice',
    'say_text',
    'dictionary',
    'terms',
    'must_not_contain',
    'fetch',
  ],
}
// A clip downloaded by `npm run eval -- fetch` instead of committed: where it comes from.
const FETCH_KEYS = ['url', 'member', 'sha256']
const SHA256 = /^[0-9a-f]{64}$/
export const EXTERNAL_AUDIO = 'audio/external/'

export function loadLanguages(root = EVALS) {
  return JSON.parse(readFileSync(join(root, 'languages.json'), 'utf8'))
}

/** Parses a JSON Lines file; returns { rows, errors }. */
export function readJsonl(path) {
  const rows = []
  const errors = []
  const lines = readFileSync(path, 'utf8').split('\n')
  lines.forEach((line, index) => {
    if (!line.trim()) return
    try {
      rows.push({ ...JSON.parse(line), _file: path, _line: index + 1 })
    } catch (error) {
      errors.push(`${path}:${index + 1}: not JSON (${error.message})`)
    }
  })
  return { rows, errors }
}

const langDirs = (dir) =>
  existsSync(dir) ? readdirSync(dir).filter((d) => statSync(join(dir, d)).isDirectory()) : []

/** Polish cases for the languages and split asked for (`all` for both splits). */
export function loadPolish({ root = EVALS, langs, split = 'dev' } = {}) {
  const cases = []
  for (const lang of langDirs(join(root, 'polish'))) {
    if (langs?.length && !langs.includes(lang)) continue
    for (const s of SPLITS) {
      if (split !== 'all' && split !== s) continue
      const path = join(root, 'polish', lang, `${s}.jsonl`)
      if (existsSync(path)) cases.push(...readJsonl(path).rows.map((r) => ({ ...r, lang })))
    }
  }
  return cases
}

/** Speech clips for the languages and split asked for, with absolute audio paths. */
export function loadSpeech({ root = EVALS, langs, split = 'dev' } = {}) {
  const clips = []
  for (const lang of langDirs(join(root, 'speech'))) {
    if (langs?.length && !langs.includes(lang)) continue
    const path = join(root, 'speech', lang, 'manifest.jsonl')
    if (!existsSync(path)) continue
    for (const row of readJsonl(path).rows) {
      if (split !== 'all' && row.split !== split) continue
      clips.push({ ...row, lang, audio_path: join(root, 'speech', lang, row.audio) })
    }
  }
  return clips
}

/** Reads a WAV header; returns null when it is 16 kHz mono 16-bit PCM, or what is wrong. */
export function wavProblem(path) {
  const buf = readFileSync(path)
  if (
    buf.length < 44 ||
    buf.toString('ascii', 0, 4) !== 'RIFF' ||
    buf.toString('ascii', 8, 12) !== 'WAVE'
  ) {
    return 'not a WAV file'
  }
  let i = 12
  while (i + 8 <= buf.length) {
    const id = buf.toString('ascii', i, i + 4)
    const size = buf.readUInt32LE(i + 4)
    if (id === 'fmt ') {
      const channels = buf.readUInt16LE(i + 10)
      const rate = buf.readUInt32LE(i + 12)
      const bits = buf.readUInt16LE(i + 22)
      return channels === 1 && rate === 16000 && bits === 16
        ? null
        : `${rate} Hz, ${channels} channel(s), ${bits}-bit; want 16000 Hz mono 16-bit`
    }
    i += 8 + size + (size & 1)
  }
  return 'no fmt chunk'
}

const isStringList = (value) =>
  Array.isArray(value) && value.every((v) => typeof v === 'string' && v)

export const sha256 = (path) => createHash('sha256').update(readFileSync(path)).digest('hex')

/**
 * A fetched clip (`fetch`): an external set too large for git, downloaded at a pinned URL. Its
 * audio lives in audio/external/ (git-ignored) and must match `sha256` once downloaded.
 */
function checkFetch(row, where, errors) {
  const f = row.fetch
  if (!row.audio.startsWith(EXTERNAL_AUDIO))
    errors.push(`${where}: a clip with "fetch" lives in ${EXTERNAL_AUDIO}`)
  if (row.source !== 'external') errors.push(`${where}: a fetched clip needs source "external"`)
  if (!f || typeof f !== 'object' || Array.isArray(f)) {
    errors.push(`${where}: a clip in ${EXTERNAL_AUDIO} needs "fetch" (url, sha256)`)
    return
  }
  for (const key of Object.keys(f)) {
    if (!FETCH_KEYS.includes(key)) errors.push(`${where}: unknown fetch key "${key}"`)
  }
  if (typeof f.url !== 'string' || !/^https:\/\/\S+$/.test(f.url))
    errors.push(`${where}: fetch.url must be an https URL`)
  if (
    f.member !== undefined &&
    (typeof f.member !== 'string' ||
      !f.member ||
      f.member.startsWith('/') ||
      f.member.includes('..'))
  )
    errors.push(`${where}: fetch.member must be a relative path inside the archive`)
  if (!SHA256.test(f.sha256 ?? ''))
    errors.push(`${where}: fetch.sha256 must be 64 lowercase hex digits`)
}

function checkKeys(row, keys, where, errors) {
  for (const key of keys.required) {
    if (row[key] === undefined) errors.push(`${where}: missing "${key}"`)
  }
  for (const key of Object.keys(row)) {
    if (key.startsWith('_') || key === 'lang') continue
    if (!keys.required.includes(key) && !keys.optional.includes(key)) {
      errors.push(`${where}: unknown key "${key}"`)
    }
  }
}

/**
 * Validates the whole dataset. Returns a list of problems (empty when valid). Besides the
 * schema, every polish case's expected text and accepted alternatives must pass the case's
 * own checks, so a case cannot ask for something its answer does not do.
 */
export function validate(root = EVALS) {
  const errors = []
  let languages
  try {
    languages = loadLanguages(root)
  } catch (error) {
    return [`evals/languages.json: ${error.message}`]
  }
  for (const [code, lang] of Object.entries(languages)) {
    if (!['word', 'char'].includes(lang.unit))
      errors.push(`languages.json ${code}: unit must be word or char`)
    if (!isStringList(lang.fillers ?? []))
      errors.push(`languages.json ${code}: fillers must be strings`)
  }
  const ids = new Map()
  const seen = (id, where) => {
    if (ids.has(id)) errors.push(`${where}: id "${id}" also used at ${ids.get(id)}`)
    else ids.set(id, where)
  }

  for (const lang of langDirs(join(root, 'polish'))) {
    if (!languages[lang]) errors.push(`polish/${lang}: not in languages.json`)
    for (const file of readdirSync(join(root, 'polish', lang))) {
      const split = file.replace(/\.jsonl$/, '')
      if (!file.endsWith('.jsonl') || !SPLITS.includes(split)) {
        errors.push(`polish/${lang}/${file}: only dev.jsonl and holdout.jsonl belong here`)
        continue
      }
      const { rows, errors: parseErrors } = readJsonl(join(root, 'polish', lang, file))
      errors.push(...parseErrors.map((e) => relative(root, e)))
      for (const row of rows) {
        const where = `polish/${lang}/${file}:${row._line}`
        checkKeys(row, POLISH_KEYS, where, errors)
        if (!ID.test(row.id ?? '')) errors.push(`${where}: id must be lowercase-kebab`)
        else seen(row.id, where)
        for (const key of ['input', 'expected']) {
          if (typeof row[key] !== 'string' || !row[key].trim())
            errors.push(`${where}: "${key}" must be text`)
        }
        if (row.split !== split) errors.push(`${where}: split "${row.split}" but file is ${file}`)
        if (!isStringList(row.tags) || !row.tags.length)
          errors.push(`${where}: tags must be a non-empty list`)
        for (const tag of row.tags ?? []) {
          if (!POLISH_TAGS.includes(tag)) errors.push(`${where}: unknown tag "${tag}"`)
        }
        for (const key of ['accept', 'must_contain', 'must_not_contain']) {
          if (row[key] !== undefined && !isStringList(row[key]))
            errors.push(`${where}: "${key}" must be a list of text`)
        }
        // Plan `cursor-context`: the field's text before the cursor, given to polish as context.
        const hasBefore = typeof row.before_cursor === 'string' && row.before_cursor.trim() !== ''
        if (row.before_cursor !== undefined && !hasBefore)
          errors.push(`${where}: before_cursor must be text`)
        if (hasBefore !== Boolean(row.tags?.includes('cursor-context')))
          errors.push(`${where}: before_cursor and the "cursor-context" tag go together`)
        if (row.max_error !== undefined && !(row.max_error > 0 && row.max_error <= 1)) {
          errors.push(`${where}: max_error must be in (0, 1]`)
        }
        if (typeof row.expected === 'string' && isStringList(row.tags)) {
          for (const reference of [row.expected, ...(isStringList(row.accept) ? row.accept : [])]) {
            const own = scorePolish(
              { ...row, accept: [], expected: reference },
              reference,
              languages[lang],
            )
            if (!own?.pass) {
              const why = own.failed.map(([n, d]) => `${n}: ${d}`).join('; ')
              errors.push(`${where}: its own answer ${JSON.stringify(reference)} fails (${why})`)
            }
          }
        }
      }
    }
  }

  for (const lang of langDirs(join(root, 'speech'))) {
    if (!languages[lang]) errors.push(`speech/${lang}: not in languages.json`)
    const path = join(root, 'speech', lang, 'manifest.jsonl')
    if (!existsSync(path)) {
      errors.push(`speech/${lang}: missing manifest.jsonl`)
      continue
    }
    const { rows, errors: parseErrors } = readJsonl(path)
    errors.push(...parseErrors.map((e) => relative(root, e)))
    for (const row of rows) {
      const where = `speech/${lang}/manifest.jsonl:${row._line}`
      checkKeys(row, SPEECH_KEYS, where, errors)
      if (!ID.test(row.id ?? '')) errors.push(`${where}: id must be lowercase-kebab`)
      else seen(row.id, where)
      if (!SPLITS.includes(row.split)) errors.push(`${where}: split must be dev or holdout`)
      if (!LICENCES.includes(row.licence))
        errors.push(`${where}: licence must be one of ${LICENCES.join(', ')}`)
      if (!SPEECH_SOURCES.includes(row.source))
        errors.push(`${where}: source must be one of ${SPEECH_SOURCES.join(', ')}`)
      for (const tag of row.tags ?? []) {
        if (!SPEECH_TAGS.includes(tag)) errors.push(`${where}: unknown tag "${tag}"`)
      }
      // Plan `dictionary-to-speech`: terms are dictionary words the clip says; must_not_contain
      // are dictionary words it does not say, which the recognizer must not force in.
      for (const key of ['dictionary', 'terms', 'must_not_contain']) {
        if (row[key] !== undefined && !(isStringList(row[key]) && row[key].length))
          errors.push(`${where}: ${key} must be a non-empty list of strings`)
      }
      if ((row.terms || row.must_not_contain) && !row.dictionary)
        errors.push(`${where}: terms and must_not_contain need a dictionary`)
      if (row.dictionary && !row.tags?.includes('dictionary'))
        errors.push(`${where}: a clip with a dictionary needs the "dictionary" tag`)
      for (const term of isStringList(row.terms) ? row.terms : []) {
        if (typeof row.text === 'string' && !row.text.includes(term))
          errors.push(`${where}: term "${term}" is not in text`)
      }
      for (const word of isStringList(row.must_not_contain) ? row.must_not_contain : []) {
        if (typeof row.text === 'string' && contains(row.text, word))
          errors.push(`${where}: must_not_contain "${word}" is in text`)
      }
      if (
        typeof row.audio !== 'string' ||
        !/^audio\/[\w./-]+\.wav$/.test(row.audio) ||
        row.audio.includes('..')
      ) {
        errors.push(`${where}: audio must be a .wav path under audio/`)
        continue
      }
      const synthetic = row.source === 'synthetic-say'
      if (synthetic) {
        if (!row.say_voice) errors.push(`${where}: a synthetic-say clip needs "say_voice"`)
        if (!row.audio.startsWith('audio/synthetic/'))
          errors.push(`${where}: synthetic clips live in audio/synthetic/`)
      }
      const fetched = row.fetch !== undefined || row.audio.startsWith(EXTERNAL_AUDIO)
      if (fetched) checkFetch(row, where, errors)
      const audio = join(root, 'speech', lang, row.audio)
      if (existsSync(audio)) {
        const problem = wavProblem(audio)
        if (problem) errors.push(`${where}: ${row.audio}: ${problem}`)
        else if (
          fetched &&
          SHA256.test(row.fetch?.sha256 ?? '') &&
          sha256(audio) !== row.fetch.sha256
        )
          errors.push(
            `${where}: ${row.audio} differs from fetch.sha256 (npm run eval -- fetch --force)`,
          )
      } else if (!synthetic && !fetched) {
        errors.push(`${where}: ${row.audio} does not exist`)
      }
    }
  }
  return errors
}
