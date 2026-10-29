# Keep Built-in AI warm

Built-in AI (llama.cpp's `llama-server` on the Mac) is fast once it has read its instructions,
about 4,000 tokens with a language preset. It is slow when it has to read them again: the first
dictation after the server starts (5–11 s), and, with Qwen 3.5 4B, every switch to another
language (about 11 s), because that model cannot reuse the shared start of the instructions. This
plan keeps the instructions read: a warm-up when the server starts, and "Fast language switching",
one request slot per language.

Status: building - 2026-10-29

## Goals and non-goals

- The first dictation after the app starts, or after the model or settings change, is as fast as
  any other.
- Switching between the user's languages costs under a second instead of 5–15 s.
- The user can trade that speed for memory: "Fast language switching" is a setting, on by default.
- Not a goal: servers the user runs (an OpenAI-compatible URL or an API key). OraType does not
  start them and cannot choose their slots.
- Not a goal: Ask, Translate and selected-text edits. Their instructions differ from Dictate's;
  they use whichever slot the server picks.
- Not a goal: changing the model catalog. Qwen3 4B Instruct stays the general model; it scores
  higher than Qwen 3.5 4B on Mandarin (88% vs 79%).

## Key decisions

- **Warm up after every start in the background** (app start, model change, slot change, after
  setup's test). Each language with a slot gets one polish request with its real instructions,
  built by the same code as a dictation (`polish_request` in `pipeline.rs`), asking for one token.
  Reason: the first dictation then reads only its transcript (0.5–1.1 s instead of 2.5–11 s).
- **The warm-up uses an app OraType does not know** (the General context). Reason: the app's part
  of the instructions comes last, inside the last ~512 tokens, where llama-server keeps a
  checkpoint; a dictation in any app then re-reads only that part (about 2 s on Qwen 3.5 4B).
- **No warm-up for a start on demand** (a dictation that finds no server). Reason: the dictation
  reads the instructions itself; a warm-up then would only compete with it.
- **One slot per language, at most three** (`--parallel N`, each slot with the full 8192-token
  context). The router's languages, in order, get slots 0, 1, 2. A dictation the router places in
  one of them sends `id_slot`. Reason: a slot keeps its own language's instructions, so a switch
  costs 0.9 s instead of 11 s (Qwen 3.5 4B). Three keeps the memory bounded.
- **"Fast language switching" is on by default**, in Settings → AI polish → Built-in AI, shown
  only when Built-in AI is in use. Its help text gives the cost: about 1–2 GB more memory per
  extra language. Reason: most Macs that run a 4B model have the memory, and the user asked
  for speed first; turning it off gives one slot, as before.
- **A different slot count restarts the server** (the setting or the language list changed).
  Reason: llama-server fixes its slots at start.
- **The server's per-request timings go to the log**: prompt tokens read and how fast, tokens
  written and how fast, and the slot's size at the end. Reason: they show whether a request found
  its instructions read (a few dozen tokens) or read them again (thousands).

## Considered

- Moving the language section to the end of the instructions, after the checkpoint, so one slot
  serves every language. Untested, and it changes what the model reads for every service.
- Re-warming after every settings change. A save happens often; most saves do not change the
  instructions, and a stale cache costs one slow dictation.
- A slot for Ask and Translate. More memory for requests that are rarer than dictation.

## Parts

| File | What it covers |
|---|---|
| [Report](../../benchmarks/reports/2026-10-29-warm-builtin-ai/README.md) | A scripted 30-dictation session per model, slots on and off, the warm-up, app switches and memory, on an M1 Pro |

## Open questions

- 16 GB Macs: three slots of Qwen3 4B add about 4.4 GB. If that pushes them into swap, cap the
  slots by the Mac's memory.
- The warm-up runs once per start; a dictionary or preset change makes the next dictation read
  the instructions again. Whether that is noticeable in real use.
