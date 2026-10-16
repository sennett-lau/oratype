# Hands-free mode ("Hey Sam")

Say "Hey Sam" and then a request, with no shortcut. Typelite listens on the microphone for the
wake phrase with its own small pipeline: an in-memory ring buffer, its own voice gate, and a
wake check on short voiced segments with the whisper.cpp that it already ships and a small
model that the app downloads. After the wake phrase the pill records the request like the Ask
shortcut, stops at "Thanks, Sam" or after 3 s of quiet, and types the request into the focused
app ("ask …" asks instead), or asks it when the user chose that in Settings.

Status: building (2026-10-17)

## Goals

- Hands-free requests: Ask questions, translations ("translate … into French"), dictation
  ("type …", 打 …, 输入 …), and a hook for app commands ("open …").
- Nothing extra to install: no new runtime, engine or external project. The wake model is
  downloaded by the app's existing model manager.
- Privacy as for the shortcuts: audio stays in memory and on the Mac until the wake phrase.
  After the wake phrase, the request follows the user's speech and AI settings. The log gets
  wake events (score, timing) only, never transcripts or audio.
- Low cost while waiting: almost nothing during silence; a short check per voiced segment.

## Non-goals

- A visible "listening" idle state in the pill. The pill appears only after the wake phrase.
- A trained keyword model in this version (see [phase-2.md](phase-2.md)).
- Pausing when another app uses the microphone (optional; see Open questions).
- Windows and Linux.

## Key decisions

- **Own voice gate + Whisper on short segments, not an external wake-word engine.** The owner
  ruled out extra installs and vendored projects. Measured cost is small (about 37 ms and
  12 ms CPU per check; see [measurements.md](measurements.md)). Research:
  [wake-word](../../research/2026-10-09-wake-word/index.md).
- **Whisper base (60 MB, multilingual, q5_1) as the wake model.** Tiny missed 5 of 27 wake
  phrases; base missed 1. The model is downloaded from the same official repository as the
  speech models and is checked with SHA-256.
- **A separate whisper context for the wake model.** A wake check never unloads the user's
  speech model, and the two never wait for each other's model load.
- **Prompt with the name only ("Sam."), with a 512-frame encoder window.** With the full phrase as
  the prompt, Whisper left "Hey Sam" out of "Hey Sam, what's the weather" (8 of 8 misses), and
  it heard "Hey Pam" as "Hey Sam". 512 frames is half the time of the full window with the same
  accuracy.
- **Typelite's own fuzzy matcher.** It tolerates Whisper's spellings ("Hay Sam", "K. Sam", "He is
  Sam", "嘿 Sam", "Heysam") but needs the phrase at the start of the segment, so speech that
  only mentions "hey Sam" mid-sentence does not wake Typelite.
- **The hands-free run is an Ask run.** It reuses Ask's recording, voice check, intent detection
  and pill. One flag on the Ask state tells the stop path to apply hands-free routing first.
- **Typing is the default; Settings can make it Ask** (changed 2026-10-17 after first use).
  Hands-free mode is mostly used to type without the keyboard, and routing by meaning answered
  dictated questions. "ask …" / 問下 … asks and "translate …" translates. With Ask as the
  default, "type …" dictates, and "write …" dictates unless a draft object follows ("write an
  email" stays Ask's draft command).
- **"Thanks, Sam" ends a request; 3 s of quiet also does.** A spoken end keeps the hands off
  the keyboard, and the longer pause lets the user think mid-sentence. The end check reuses
  the wake model on the last 2.5 s before each pause.
- **The wake model hears a level-normalised segment.** A built-in Mac microphone is about
  20 dB quieter than the test clips; at that level the wake rate fell from 25 to 13 of 27.
  Raising the segment's peak to −3 dBFS, and joining a lone "Hey." with the next stretch,
  brought it back to 24 of 27 with no false wakes (measurements.md).
- **The listener keeps the microphone open while another run records, and ignores the audio.**
  CoreAudio allows two streams. Reopening the device after every run would make the indicator
  flicker and lose the first moment of the next wake phrase.

## Parts

| File | Covers |
| --- | --- |
| [listening.md](listening.md) | Capture, ring buffer, voice gate, pausing, releasing the microphone |
| [wake-check.md](wake-check.md) | Segments, the Whisper check, the fuzzy matcher, sensitivity |
| [routing.md](routing.md) | Ending the request ("Thanks, Sam", pauses) and routing it (type, ask, translate, open) |
| [settings.md](settings.md) | Settings switch, wake name, sensitivity, tray item, model download, strings |
| [measurements.md](measurements.md) | CPU, memory, latency, false accepts and rejects on this Mac |
| [phase-2.md](phase-2.md) | Proposal: an own small keyword model, if Whisper segments become too heavy |

## Open questions

- Real voices and rooms: the numbers are for synthetic `say` voices with mixed-in noise, now
  also at a built-in microphone's level. Collect a small real-voice set (with consent, kept
  outside the repository) before the plan is done.
- The pill shows Ask's recording state for every hands-free request, also when it will be
  typed. Should it show Dictate's look when typing is the default?
- Should "Hey Sam, what's …" in one breath keep the words after the wake phrase? Today the request
  starts when the pill appears, so the user pauses after "Hey Sam". Prepending the ring buffer's
  audio after the match would allow one breath.
- Pause automatically while another app (a call) uses the microphone? macOS reports this through
  `kAudioDevicePropertyDeviceIsRunningSomewhere`. Not built.
- Voice commands ("open Safari") depend on the `feature/voice-commands` branch. The hook in
  `hands_free::routing::run_voice_command` returns "not handled" until that feature is merged.
- Non-English, non-Chinese "type" words (French "tape", Spanish "escribe") are not routed yet.
