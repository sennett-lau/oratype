# Settings and tray

How the user turns Hands-free mode on and adjusts it. Part of [hands-free-mode](index.md).

## Settings → General → Hands-free

A group after Recording (`HandsFreeSettings.tsx`). It lives in General because it is another way
to start a recording, next to the shortcuts and the microphone.

- **Hands-free mode** switch, off by default. Help text: say "Hey {name}", then the request. The
  microphone stays open, so macOS shows its indicator, and nothing is sent before the wake
  phrase.
- When on:
  - a status line: listening for "Hey {name}", the wake model is missing (60 MB) with a
    **Download** button, download progress with **Cancel**, or the microphone could not be
    opened;
  - **Wake name**: the name after "Hey", default "Sam", at most 32 characters. An empty name
    falls back to "Sam". A typed "Hey Jarvis" is the same as "Jarvis";
  - **Sensitivity**: Low / Normal / High (see [wake-check.md](wake-check.md));
  - **After "Hey {name}"**: *Type what I say* (default) / *Ask anything*, with help that names
    the words for the other one ("ask …" / "type …"), "translate …", and how a request ends
    ("Thanks, {name}", or 3 s of quiet). See [routing.md](routing.md).

The missing-model line says why there is a download: the wake phrase is heard on the Mac by a
small model, so nothing leaves the Mac before it.

Config: `hands_free: { enabled, wake_name, sensitivity, action }` in `AppConfig` (serde
defaults, so older settings load with the feature off and typing as the action). Saving the
settings applies them at once: the listener starts, restarts or stops. `update_config` puts
`hands_free` and `input_device` in its change patch for this; without them a new wake name,
sensitivity or microphone took effect only after a restart.

## Wake model download

The commands `get_hands_free_status`, `download_hands_free_model` and
`cancel_hands_free_model_download` reuse the model setup that built-in speech and AI use. The
download resumes, checks free space and SHA-256, and sends progress as `hands-free:setup`. The
file sits in the same models folder as the speech models, and the speech model list does not
show it. Listening starts by itself when the download finishes.

## Tray

The menu-bar menu has a **Hands-free Mode** check item under Start Recording. It flips the
setting, saves it, tells open windows (`config:patch`) and applies it.

## Strings

All new strings are in `en.json` with translations in zh, zh-Hant, es, fr, de and ja (the
locale parity test checks this). The tray label is in `tray.rs` for the same languages. Locales
whose own "type" verb is not routed yet show the English words ("type …").
