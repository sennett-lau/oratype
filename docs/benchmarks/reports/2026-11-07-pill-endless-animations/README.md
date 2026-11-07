# Pill: endless animations piled up after every dictation

A memory and CPU fix. After about a week of use the pill's web page held over a gigabyte, and
while the pill showed it grew by about 20 MB a second and spent a large part of every frame in
JavaScript. The recording dot's framer-motion loop (`repeat: Infinity`) kept running after the
dot unmounted, so each dictation left one more endless animation behind.

## Provenance

| Field | Value |
|---|---|
| Date | 2026-11-07 |
| Kind | Memory / CPU fix (frontend) |
| Revision | `main` at `d92084e` (before) and this PR (after) |
| Hardware | Apple Silicon Mac, macOS 26, Retina display plus external monitors |
| App | Installed OraType 1.2.1, open 8 days, about 2,250 dictations |
| framer-motion | 11.18.2 |

## What was seen in the installed app

Measured with `footprint -p <pid>` (the number Activity Monitor shows) on the pill's
`com.apple.WebKit.WebContent` process:

| Process | Footprint |
|---|---:|
| Main window page | 122 MB |
| **Pill page** | **992 MB to 1.48 GB** |
| Ask page | 23 MB |

- Almost all of it was "WebKit malloc" (JavaScript objects), and most was swapped out.
- It did not grow while the pill was hidden. During one recording, sampled every 0.5 s, it grew
  from 1,016 MB to 1,300 MB in 15 s (about 19 MB/s), then fell back slowly as the garbage
  collector caught up.
- `sample <pid> 5` during a recording: 39% of the page's main thread was inside
  `requestAnimationFrame` callbacks, in one function that looped over many items; 7.6% was
  `fmod` (JavaScript `%` on doubles, which framer-motion uses for each repeating animation's
  progress).

## Finding the cause

Each step ruled something out:

| Test | Result |
|---|---|
| JS heap of the pill page in headless Chromium, 20 simulated dictations | +40 KB per run, DOM node count flat |
| Headless WebKit, 70 runs | levels off at about 120 MB |
| Volume events, the waveform and aurora loops switched off | no change |
| Real WKWebView, one Tauri path at a time: 36,000 IPC calls, 14,000 events, 8,300 resizes, 1,200 show/hide | +2 to +4 MB each; resizes level off |

The probe that found it ran the real pill page (production bundle) in the real capsule window,
made a non-activating panel as the app does, while a Rust thread sent the events the pipeline
sends (states, 33 volume events a second, transcript, polish chunks, insert result). An
initialization script counted `requestAnimationFrame` callbacks per second and the time spent
in each. Three loops run while recording: the waveform, the aurora glow and framer-motion's
frame loop. framer's loop runs only while it has an unfinished animation.

| Sequence (4 runs each, 5 s recording) | framer loop during recording |
|---|---|
| recording → idle | never |
| recording → transcribing → idle | from the second run on, every run |
| recording → transcribing (2.5 s, all transitions done) → idle | from the second run on, every run |
| same, recording dot's `animate` removed | never |

Going straight from recording to idle keeps `CapsuleRecording` mounted (the hidden pill keeps
showing its last state), so its dot never unmounts. Any working state unmounts it, and its
endless animation stayed in framer's loop. `CapsulePreparing`'s dot and the settings save
spinner (`DirtyBar`) use the same pattern.

## Change

- The recording dot and the preparing dot are CSS animations (`.pill-dot-pulse`,
  `.pill-dot-breathe` in `globals.css`), turned off under reduced motion like the aurora. The
  save spinner uses Tailwind's `animate-spin`, like the other spinners. A CSS animation ends with
  its element.
- A test fails if any source file uses framer-motion's `repeat: Infinity`.

Same look: the same keyframes, durations and easing; with reduced motion the dots stay still,
as before.

## Results

60 simulated dictations (preparing, 3.9 s recording, transcribing, polishing, pasting, idle),
framer's loop in the steady part of each recording:

| Run | Before: calls/s | Before: JS ms/s | After: calls/s | After: JS ms/s |
|---:|---:|---:|---:|---:|
| 1 | 60 | 2 | 0 | 0 |
| 10 | 60 | 1 | 0 | 0 |
| 30 | 60 | 10 | 0 | 0 |
| 50 | 60 | 28 | 0 | 0 |
| 60 | 59 | 18 | 0 | 0 |

Before, the cost per second grows with the number of dictations (noisy); after, framer does no
work while recording. Over only 60 runs the page's memory is dominated by garbage-collector
timing (peaks of 133 MB before and 110 MB after), so it is not used as the measure here; the
installed app's gigabyte came from about 2,250 runs.

## Validation and limitations

- `npx vitest run` (815 tests), `npx tsc --noEmit`, `npx eslint src/`, `npx prettier --check src`.
- The probes were temporary examples and are not committed.
- Not yet measured: an installed build with this fix over several days. Until it ships,
  quitting and reopening OraType frees the memory.
- Not investigated: whether framer-motion leaves the animation running in every browser or only
  in WKWebView; the fix does not depend on it.
