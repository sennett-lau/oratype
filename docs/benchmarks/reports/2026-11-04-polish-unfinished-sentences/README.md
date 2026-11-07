# Polish completes unfinished sentences

A quality check, measured with `evals/`. A user of 1.2.1 dictated "The message should include"
and got that sentence followed by a numbered list of points they never said.

## Provenance

| Field | Value |
|---|---|
| Date | 2026-11-04 |
| Kind | Quality check (cases and a code guard; a prompt change was tried and dropped) |
| Revision | `main` at `6eaa07a` (after the [answers fix](../2026-11-03-polish-answers-questions/README.md)) and this PR |
| Hardware | Apple Silicon Mac, macOS 26; an OpenAI-compatible llama-server on a separate PC (CUDA) |
| Built-in AI | llama-server, `Qwen3.5-4B-Q4_K_M.gguf` (the built-in default), default settings |
| Server AI | llama-server, `Huihui-Qwen3.5-4B-abliterated.Q4_K_M.gguf`, `--jinja --reasoning off` |
| Data | 15 new cases tagged `unfinished` in all six languages (9 dev, 6 holdout), 3 samples per case |

## What happens

The model reads a sentence that stops mid-way ("the main things we need to fix before launch
are", 封email入面要包括) as an invitation and adds a list: "1. Performance optimization
2. Bug fixes 3. Security patches", "1. 主題 2. 收信人 3. 日期 4. 正文".

| Model, `main` | Completed (of 45 samples) | Reached the paste | Pass |
|---|---:|---:|---:|
| Built-in Qwen3.5-4B | 0 | 0 | 43 |
| Server model | 25 | 0 | 39 |

1.2.1 checks only for a change of language after polish, so the list was pasted. The answer check added on `main` by the
answers fix (`output_guard::answered_instead`: a numbered list made mostly of words never spoken)
catches every one of the 25 and pastes the transcript instead. The server's failures left are a
kept "Um" (in that pasted transcript) and Simplified Chinese written in Traditional characters.

## Change

- `evals/`: the 15 `unfinished` cases, and the eval results keep the model's text that the guard
  replaced (`rejected`), so a report can show what was caught.
- The answer check also reads bulleted lists (`- `, `* `, `• `), not only numbered ones: an
  invented bulleted list of short items was too short for the length check.

## Tried and dropped: a prompt rule

A rule in rule 10 ("a sentence left unfinished stays unfinished: never finish it, never add a
list of items after it") with one example cut the server's completions from 25 to 8 of 45. But
the built-in model, which never completed these sentences, got more literal. With the first
wording it kept self-corrections and spoken numbers (23 regressions against the baseline); with
a narrower wording, on the 14 cases that regressed, rerun 6 times each, it passed 44 of 84
against 49 of 84 for `main` (en-h02 "no actually" correction 6 → 3, zh-023 6 → 3). Since the
code check already stops every completion from reaching the paste, the prompt is unchanged.

## Reproduce

```sh
npm run eval -- polish --split all --tag unfinished --n 3 --llama-model <Qwen3.5-4B-Q4_K_M.gguf>
npm run eval -- polish --split all --tag unfinished --n 3 --ai-url <server>/v1 --ai-model <name>
```
