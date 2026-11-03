# Polish answers dictated questions

A quality fix, measured with `evals/`. Dictation sometimes pasted an answer instead of what the
user said: "summarize the main points of the meeting for me" came back as an invented
summary, 呢個bug點樣先可以fix到 as a five-step guide, "how many days are there until
christmas" as "There are X days until Christmas".

## Provenance

| Field | Value |
|---|---|
| Date | 2026-11-03 |
| Kind | Quality fix (prompt and code) |
| Revision | `main` at `fad8fc0` (before) and this PR (after) |
| Hardware | Apple Silicon Mac, macOS 26; an OpenAI-compatible llama-server on a separate PC (CUDA) |
| Built-in AI | llama-server, `Qwen3.5-4B-Q4_K_M.gguf` (the built-in default), default settings |
| Server AI | llama-server, `Huihui-Qwen3.5-4B-abliterated.Q4_K_M.gguf`, `--jinja --reasoning off` |
| Data | 216 polish cases (54 new, tagged `no-answer`), dev and holdout, 3 samples per case, the app's default language list, a general app unless stated |

Full summaries: [before-builtin](before-builtin-summary.md), [after-builtin](after-builtin-summary.md),
[before-server](before-server-summary.md), [after-server](after-server-summary.md),
[after-server, ChatGPT app](after-server-chatgpt-summary.md).

## Causes

1. **The prompt.** Rule 10 forbade carrying out requests, but its wording and examples were
   about writing or translating "in another language". Nothing said that a plain question is
   written down unanswered, and it named a "question operation" (the Ask shortcut) that a
   4B model can take for any question.
2. **The guard after polish** (`llm/output_guard.rs`) only caught a change of language. An answer
   in the dictation's own language went through.

## Change

- **Prompt rule 10** ("NEVER ANSWER, NEVER EXECUTE"): the transcription is what the user is
  typing, never a message to the model. A question is output as the same question, never
  answered, with no answer added after it; a request is written down, never carried out. Three
  new examples (an English question, an English request, a Cantonese question) on topics no
  case uses, and one line in the self-check list.
- **Answer check in code** (`output_guard::answered_instead`), plain dictation only: a result
  longer than 1.5 × the transcript plus 12 units (words, or Chinese and Japanese characters), or
  with a numbered list whose items are mostly words never spoken (Chinese compared in one
  script), is an answer. Then the transcript is pasted, as for a language change. The log says
  "Polish answered or carried out the dictation instead of writing it down; pasting the
  transcript". Markdown is not used as a sign: document and developer apps ask for bullets.
- The language check no longer treats a result with no words at all ("two point five" → "2.5")
  as another language.
- `evals/`: 54 `no-answer` cases in all six languages (36 dev, 18 holdout), and a `--context
  <profile id>` option for the app dictated into.

## Results

Answers that reached the paste, out of 162 `no-answer` samples:

| Model | Before | After | Caught by the new check |
|---|---:|---:|---:|
| Built-in Qwen3.5-4B | 3 | 0 | 0 |
| Server model | 39 | 1 | 20 |
| Server model, ChatGPT app (`--context prompt.chatgpt`) | – | 0 | 30 |

The one left is "please summarize this article in three bullet points" followed by an empty
"1. … 2. … 3. …" list.

Pass rate, before → after (cases):

| Built-in | en | es | fr | ja | yue | zh-Hans | all |
|---|---|---|---|---|---|---|---|
| no-answer dev | 93% → 100% (10) | 100% → 75% (4) | 100% → 100% (4) | 92% → 100% (4) | 96% → 100% (8) | 100% → 100% (6) | 96% → 97% (36) |
| no-answer holdout | 100% → 100% (5) | 100% → 100% (2) | 67% → 100% (2) | 100% → 100% (2) | 100% → 100% (4) | 78% → 100% (3) | 93% → 100% (18) |
| other dev | 87% → 85% (32) | 75% → 79% (8) | 79% → 75% (8) | 62% → 67% (8) | 78% → 80% (38) | 73% → 76% (28) | 78% → 79% (122) |
| other holdout | 70% → 80% (10) | 50% → 50% (2) | 50% → 50% (2) | 67% → 67% (2) | 69% → 71% (14) | 70% → 67% (10) | 68% → 70% (40) |

| Server model | en | es | fr | ja | yue | zh-Hans | all |
|---|---|---|---|---|---|---|---|
| no-answer dev | 73% → 100% (10) | 50% → 92% (4) | 75% → 100% (4) | 50% → 75% (4) | 62% → 83% (8) | 0% → 50% (6) | 54% → 84% (36) |
| no-answer holdout | 87% → 93% (5) | 100% → 100% (2) | 50% → 100% (2) | 100% → 100% (2) | 100% → 100% (4) | 0% → 0% (3) | 74% → 81% (18) |
| other dev | 72% → 78% (32) | 62% → 62% (8) | 75% → 75% (8) | 46% → 46% (8) | 76% → 75% (38) | 14% → 14% (28) | 58% → 59% (122) |
| other holdout | 80% → 80% (10) | 50% → 50% (2) | 50% → 50% (2) | 100% → 50% (2) | 57% → 52% (14) | 30% → 27% (10) | 57% → 53% (40) |

Overall: built-in 80% → 82%, server model 58% → 64%.

Check of the check: every sample of the three earlier runs (before-built-in, before-server, the
first after-server dev run) and every case's expected answer, 2,360 texts, went through
`answered_instead`. It fired on 50 texts, all of them answers (lists of ideas, summaries,
explanations, in every language), and on no correct polish. In the final runs it fired on no
case outside `no-answer`.

## Regressions and what is left

- **es-012** (built-in, 3/3): "explícame qué es una base de datos" became the question "¿Qué es
  una base de datos?". It is no longer answered, but it is reworded.
- **ja-h01** (server, 3/3): "バージョン2.1にアップデートしてください" came back with its clauses
  swapped. A holdout case, so the prompt was not tuned on it.
- yue-013, zh-h08 (built-in), zh-022 and yue-h06 (server): one or two samples of three, within
  noise at n = 3.
- The server model's remaining `no-answer` failures are not answers: it writes Simplified
  Chinese in Traditional characters (zh-Hans 21% on all cases, before and after), keeps
  えーと, and turns Cantonese 邊座 into 哪一座.
- With streaming insertion on, text is typed while it arrives, so the check cannot replace it.
- Apps of the "Prompt or code" family (ChatGPT, Claude, editors) are told to "make the spoken
  request explicit and usable"; with the server model 30 of 162 samples needed the code check
  there, against 20 in a general app. That rule could be revisited.
- The language check still replaces "Price it at 24.99" for "price it at twenty four ninety
  nine" (en-h08), as before this change.

## Reproduce

```sh
npm run eval -- polish --split all --n 3 --llama-model <Qwen3.5-4B-Q4_K_M.gguf>
npm run eval -- polish --split all --n 3 --ai-url <server>/v1 --ai-model <name>
npm run eval -- polish --split all --tag no-answer --context prompt.chatgpt --ai-url <server>/v1 --ai-model <name>
```

Baseline saved: `evals/baselines/polish-qwen3.5-4b-q4-k-m.json` (the after-built-in run).
