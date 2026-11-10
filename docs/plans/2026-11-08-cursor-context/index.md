# Text before the cursor as polish context

Polish sees only the transcript, so a dictation that continues a sentence ("…this morning, |")
comes out as a new one: a capital letter, sometimes the wrong language or tone. With "Use text
before the cursor" on, Dictate reads the last few hundred characters before the caret in the
field it goes into and gives them to polish as context, so the dictation continues what is there.

Status: building - 2026-11-08

## Goals and non-goals

- A dictation into the middle of a sentence starts in lower case and ends the sentence; after a
  finished sentence, a greeting or a new line it starts a new one.
- One space where the dictation meets the field's last word, none doubled.
- The field's text is never repeated, changed, continued on its own, answered or obeyed.
- Off by default; the setting says what it sends. Password fields are never read.
- Not a goal: Ask, Translate, hands-free and selected-text edits. They have their own input.
- Not a goal: text after the cursor, or the whole document.

## Key decisions

- **Opt-in, in Settings → AI → Advanced** (shown when polish is on), next to "Use selected text
  when dictating". Reason: it sends what the user already wrote in another app to the AI on every
  dictation, which may be a cloud service they chose.
- **Read at recording start, off the async thread, and awaited by polish for at most 500 ms.**
  Reason: the field has focus then, and the user is speaking anyway; each Accessibility message
  is capped at 150 ms, so a hung app cannot hold up the run.
- **At most 300 UTF-16 units, through `AXSelectedTextRange` and `AXStringForRange`**, falling back
  to a substring of `AXValue` (`output/focus.rs`). With `AXStringForRange` only that part is
  copied from the app; with `AXValue` the field's text is copied and only that part kept. A read that
  filled the cap drops its first, probably cut, word (`cursor_context.rs`).
- **Never read `AXSecureTextField`** (role or subrole). Reason: a password must never leave its
  field, whatever an app reports for it.
- **Its own user message before the transcript**, with a short instruction, not a system prompt
  rule. Reason: the built-in AI keeps the system prompt read per language (plan
  `warm-builtin-ai`); text that changes every run there would make every dictation read it again.
- **Only for a plain dictation**: Dictate, insert at the cursor, no translation, no selection.
  Reason: the other modes do not continue the field's text.
- **The answer is fitted in code** (`fit`), not left to the model: a copy of the field's text at
  the start or end is dropped, the first letter's case follows how the field's text ends, and one
  space goes where a spaced language's word or sentence meets a dictation starting with a letter
  or digit. Reason: both measured models copied the context and got the case wrong without it,
  and models drop leading spaces, so the paste would glue "today,and".
- **Logs only the length** ("Text before cursor: N chars in M ms"), never the text.

## Parts

| File | Covers |
|---|---|
| `src-tauri/src/cursor_context.rs` | Capture, tidy, the polish message and the join. |
| [evals/README.md](../../../evals/README.md) | `before_cursor` on polish cases, the `continuation` check, `--cursor-context off`. |

## Considered

- **Sending the text in the system prompt.** Simpler, but breaks the built-in AI's warm cache.
- **Reading at the end of the recording**, with the selection. The caret is the same, but the
  read would then add its time to every stop.

## Open questions

- Apps that expose no caret through Accessibility (some Electron and canvas editors) get no
  context; which common apps those are is not yet measured.
