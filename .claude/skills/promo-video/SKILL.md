---
name: promo-video
description: Make the short promo video for a Typelite release (10–15 s, 1920×1080) from the real app UI, in the standard folder output/promo-<version>/. Use when preparing a release, or when someone asks for a promo, launch or "what's new" video for a version.
---

# Make a release promo video

One short film per release that shows what is new, made from the app's real UI. Everything
lives in **`output/promo-<version>/`** (git-ignored), so every release looks the same on disk:

| Path | What it is |
| --- | --- |
| `film.html` | The film: shots, captions and timing, drawn by `render(t)` |
| `stills/<shot>.png` | Stills of the real app (dark, 2× or 4×), from the capture harness |
| `music.wav` | The soundtrack, synthesised for the film's length |
| `typelite-<version>-promo.mp4` | **The video** (the one file to share) |
| `sheet.png` | One frame per second on one image, to check the film at a glance |
| `preview/` | Single frames from `--preview` (optional, safe to delete) |

## The format

- **1920×1080, 30 fps**, H.264 (`yuv420p`, CRF 16) with AAC 192 kbit/s, **10–15 s**.
- **Three to five shots:** a title card (the version and one headline), one to three shots of
  the new feature on real stills, and the end card (name, free and open source, the site
  `typelite.sennettlau.me`).
- **Real UI only.** Stills come from the app's own components in the capture harness
  (`site/capture-app`), never a drawing of a screen that does not exist. Numbers in captions
  come from the release's measurements; claim nothing that was not measured or shipped.
- **Two languages per caption:** the headline in the language of the release's main audience,
  the other below it in grey. Default English first with Traditional Chinese (Hong Kong) under
  it; a Cantonese-led release (1.1.3) puts Cantonese first.
- **Look:** dark stills on the near-black background with the teal → violet gradient of the
  pill (the template's styles). One slow push-in per still; captions fade and lift in; the film
  fades in from and out to black.
- **Sound:** `promo-music.py` only (synthesised, no samples, so no licence to track). No voice
  unless asked.
- No private machine names, addresses or personal data in any still or caption; the harness
  mocks hold only made-up data.

## Steps

1. **What is new.** Read the version's What's New entry (`src/lib/whatsNew.ts`, strings in
   `src/i18n/locales/en.json` and `zh-Hant.json`) and pick the headline feature, plus at most one
   more. Write the storyboard as the shots' comments in `film.html`: shot, seconds, caption in
   both languages.
2. **Stills.** Use the shots in `PROMO_SHOTS` (`site/scripts/capture-promo-stills.mjs`). For a
   feature with no shot yet, add a screen to the harness (`site/capture-app/mocks.ts`: the
   `Screen` type, `SCREENS`, the config and command mocks it needs; `app.tsx` for store state)
   and a shot to `PROMO_SHOTS`. Then, from the repository root (after `npm ci`):

   ```sh
   node site/scripts/capture-promo-stills.mjs <version> <shot> [<shot> ...]
   ```

   Look at every still: the harness prints warnings for unmocked commands, and a missing mock
   shows as an empty or broken pane.
3. **Film.** Copy `.claude/skills/promo-video/film-template.html` to
   `output/promo-<version>/film.html`, fill in the `{{…}}` and write the shots. Keep `render(t)`
   pure (no timers or CSS animations): the recorder steps `t` itself. Check single frames:

   ```sh
   node site/scripts/record-film.mjs <version> --preview 1,4,8,12
   ```

   and look at `preview/t*.png` (text not clipped, stills sharp, nothing behind a caption).
4. **Music**, as long as `window.DURATION`:

   ```sh
   python3 site/scripts/promo-music.py <version> <seconds>
   ```

5. **Record:**

   ```sh
   node site/scripts/record-film.mjs <version>
   ```

   It renders every frame, encodes `typelite-<version>-promo.mp4` with the music, writes
   `sheet.png` and deletes the frames. Look at `sheet.png`, and check the length with
   `ffprobe -v error -show_entries format=duration -of csv=p=0 <mp4>`.
6. **Hand over:** reveal the video in Finder (`open -R output/promo-<version>/typelite-<version>-promo.mp4`)
   and give the path. Nothing in `output/` is committed; changes to the harness or the scripts
   go in a pull request.

Needs Chromium (`/Applications/Chromium.app`, or set `CHROMIUM`), ffmpeg and Python 3.
