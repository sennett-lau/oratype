# Measurements

Cost and accuracy of the wake pipeline on the target Mac. Part of [hands-free-mode](index.md).
The raw results are in the research entry
([wake-word/results.json](../../research/2026-10-09-wake-word/results.json)).

## Method

- Machine: Apple M1 Pro, 32 GB, macOS 26.5. whisper.cpp through whisper-rs 0.16 with Metal and
  a release build.
- Audio: `benchmarks/hands-free/make_fixtures.sh` synthesises 76 clips with macOS `say` at
  16 kHz:
  - 27 wake clips: 8 English voices (US, UK, AU, IE, IN, ZA) × "Hey Sam." / "Hey, Sam!" /
    "Hey Sam, what's the weather like today?", plus "嘿 Sam" in Mandarin, Taiwanese and
    Cantonese voices;
  - 47 other clips: "Hey Pam", "Same here", "Hey man …", "I saw Sam yesterday …", "Thank you",
    "Hey Siri", "Hey Tom …", and Chinese and Cantonese sentences;
  - 72 s of continuous background talk in English and Mandarin that mentions "Sam" and "Hey".
- Harness: `cargo run --release --example benchmark_wake -- <model> <dir>`. Each clip runs
  through the real listener code (gate, segmenter, Whisper check, matcher), with 1 s of −65 dBFS
  room noise before and after it and noise mixed under it. CPU time comes from `getrusage`
  (all threads), memory is peak RSS, and latency is the wall time of one check.

## Results (shipped settings: base q5_1, prompt "Sam.", 512-frame window, Normal)

| What | Result |
| --- | --- |
| Idle (60 s of room noise) | 0 checks; 2.5 ms CPU per minute (0.004% of one core) |
| Background talk (72 s) | 9 checks, 0 false wakes; 0.18% of one core on average |
| Wake check latency | p50 37 ms, p90 51 ms (one outlier 173 ms, the first check after load) |
| CPU per check | p50 12 ms (most of the work runs on the GPU) |
| Model load | 0.1–0.3 s; peak RSS of the process 168–184 MB (from 9 MB) |
| Wake phrases accepted | 26 / 27 (false reject 3.7%) |
| Other speech accepted | 0 / 47 (false accept 0%) |

From the end of "Hey Sam" to the start of the Ask run: the 300 ms gate hangover plus about 40 ms
for the check, so the pill appears about 0.35 s after the user stops speaking. The one miss was
the Mandarin TTS voice saying "Sam", which every setting heard as "Peace him".

## Variants

| Variant | Wake accepted | Other accepted | Check p50 |
| --- | --- | --- | --- |
| **base, "Sam." prompt, ctx 512, Normal (shipped)** | 26/27 | 0/47 | 37 ms |
| same, Low | 26/27 | 0/47 | 36 ms |
| same, High (final rules) | 26/27 | 0/47 | 38 ms |
| High with one consonant allowed in "Sam" (rejected) | 26/27 | 17/47 | 36 ms |
| base, ctx 256 | 24/27 | 0/47 | 28 ms |
| base, full 30 s window | 26/27 | 0/47 | 78 ms |
| base, no prompt | 23/27 | 0/47 | 39 ms |
| base, prompt "Hey Sam." | 18/27 | 0/47 (1/47 with an earlier matcher) | 37 ms |
| tiny q5_1, "Sam." prompt, ctx 512 | 22/27 | 0/47 | 28 ms |

## At a built-in microphone's level (2026-10-17)

In first use on the target Mac the wake phrase was often missed. A real voice on the built-in
microphone peaks at −23 to −27 dBFS (the voice check's log lines), about 20 dB under the `say`
clips (−2 to −4 dBFS). `benchmark_wake --gain-db -22` plays the clips that much quieter, with the
room noise still at −65 dBFS. The fixtures now also hold 18 end-phrase clips ("Thanks, Sam.",
"See you at six. Thank you, Sam.", 多謝 Sam, 谢谢 Sam) and 18 that must not end a request
("I'll ask Sam about it tomorrow.", 多謝晒你哋嘅幫忙。), 112 clips in all. Same machine, Normal.

| Version | Level | Wake accepted | Other accepted | End phrases heard | False ends |
| --- | --- | --- | --- | --- | --- |
| first build | as recorded | 25/27 | 0/47 | | |
| first build | −22 dB | 13/27 | 0/47 | | |
| first build, peak raised to −3 dBFS | −22 dB | 20/27 | 0/47 | | |
| **current** (peak raised, lone "Hey." joined) | as recorded | 24/27 | 0/47 | 18/18 | 0/18 |
| **current** | −22 dB | 24/27 | 0/47 | 16/18 | 0/18 |
| **current** | −30 dB | 6/27 | 0/47 | 0/18 | 0/18 |

- At −22 dB the first build heard "Hey, sir", "Hey there" and "Peace, Sam", and split "Hey, Sam!"
  into "Hey." and "Sam."; raising the level fixed the first, joining a lone greeting the second.
- At full level the current build misses one more "Hey Sam, what's the weather like today?"
  said in one breath (an open question in [index.md](index.md)).
- The end check needs the last 2.5 s, not only the last stretch: at −22 dB "Thanks," and "Sam"
  are two stretches (3/18 heard with the last stretch only). Decoding 多謝 Sam as English never
  matched; the second, Chinese decode hears both Chinese clips.
- −30 dB (a voice further away than arm's length) is out of reach for base; check p50 stayed at
  30–33 ms and background talk at 0 false wakes.

## What these numbers do not cover

- **Real voices, accents, distance and noise.** `say` voices are clean and consistent. Real
  false-reject rates will be higher, and a real-voice set is an open question in
  [index.md](index.md).
- **Real conversation.** TTS talk has few pauses (9 checks in 72 s). People pause more, so expect
  roughly one check per phrase, about 1 check per second while someone talks nearby: about 12 ms
  CPU and 37 ms GPU each, or about 1–2% of one core while they talk.
- **The capture itself.** The CoreAudio input stream runs whenever the mode is on. It was not
  measured here, and it is the same stream a recording uses.
- **False wakes over hours of audio.** The negative set is small. Track wake events in the log
  (`Hands-free: wake phrase heard …`) during real use.
