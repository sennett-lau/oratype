# Typelite's own keystrokes are not shortcuts

Typelite sends ⌘C to read a highlight in apps that do not report it through Accessibility, and
⌘V to paste. Both go through System Events, so the shortcut listener sees them like the user's
keys. A shortcut on a bare modifier (Dictate on Left Command) fired on their ⌘. While Ask was
starting, that "Dictate" press meant "stop Ask", so Ask stopped before it recorded anything and
the pill ended as "no speech". This plan keeps Typelite's own keystrokes out of shortcut
matching.

Status: done - 2026-10-17.

## Goals and non-goals

- A keystroke Typelite sends never starts, stops or cancels a run.
- The user's keys keep working while Typelite sends one: a key released in that moment still
  counts, so nothing is left held.
- Not a goal: changing when a bare-modifier shortcut fires for the user's own key presses. A
  bare Left Command still fires when it goes down, also as part of the user's own ⌘C; see Open
  questions.

## Key decisions

| Decision | Reason |
|---|---|
| A mark (`native_hotkey::OwnKeystroke`) lives while the `osascript` call that sends the keystroke runs, and 250 ms after | The events reach the tap shortly after the call returns. The sender's process id does not help: System Events posts them, not Typelite. |
| While marked, the tap matches only key releases; new presses reach the app untouched | Ignoring presses keeps the synthetic ⌘ from firing anything. Matching releases means a key the user lets go of in that moment (End held to talk) is not left stuck. |
| The ⌘C that reads a highlight (`selection.rs`) and the ⌘V that pastes (`output/clipboard.rs`) are marked | They are the keystrokes Typelite sends through System Events on macOS. |

## Parts

None; the decisions above are the plan. Code: `native_hotkey.rs` (`OwnKeystroke`,
`TapLogic::process_mac_event_during`), `selection.rs`, `output/clipboard.rs`.

## Open questions

- Should a bare Command, Control or Option shortcut fire only when tapped alone (released
  without another key)? Today it fires when it goes down, so a user with Dictate on Left Command
  also starts Dictate with every ⌘ shortcut they type. Firing on release would change how hold
  mode feels, so it needs its own decision.
