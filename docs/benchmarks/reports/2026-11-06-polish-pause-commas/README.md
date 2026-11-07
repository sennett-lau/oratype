# Polish copies the recognizer's pause commas

A quality fix, measured with `evals/`. Polish often over-punctuated and cut one thought into
pieces: "我想問下你，聽日得唔得閒，一齊食個晏？", "明日の午後に、資料を、送ります".

## Provenance

| Field | Value |
|---|---|
| Date | 2026-11-06 |
| Kind | Quality fix (code; two prompt changes tried and dropped) |
| Revision | `main` at `6eaa07a` (before) and this PR (after) |
| Hardware | Apple Silicon Mac, macOS 26; an OpenAI-compatible llama-server on a separate PC (CUDA) |
| Built-in AI | llama-server, `Qwen3.5-4B-Q4_K_M.gguf` (the built-in default), default settings |
| Server AI | llama-server, `Huihui-Qwen3.5-4B-abliterated.Q4_K_M.gguf`, `--jinja --reasoning off` |
| Data | 18 new `over-punctuation` cases in six languages (11 dev, 7 holdout) and 3 list-comma cases; all 237 cases, 3 samples each |

## Cause

Qwen3-ASR and Whisper write a Chinese or Japanese comma (，、) at every pause. Polish copies
them. On `main`, the new cases whose transcript had no commas mostly passed; the failures were
nearly all transcripts with pause commas, kept as they were.

## Change

`llm/pause_commas.rs`: the text sent to polish has its ， and 、 removed (sentence ends 。？！ stay).
The model then places commas by grammar. The transcript itself is unchanged where it is pasted
as is (polish off, or the dictation guard falls back to it). ASCII commas are untouched.

## Results

`over-punctuation` cases, samples passed (of 54):

| Model | `main` | This PR |
|---|---:|---:|
| Built-in Qwen3.5-4B | 41 | 54 |
| Server model | 30 | 38 |

The server's failures left are Simplified Chinese written in Traditional characters, and English
commas ("I was thinking, maybe …"), which this change does not touch.

No other case's transcript has ， or 、, so the text the model receives for them is identical
and their differences between runs are sampling noise (built-in 83% overall, server 65%). Three
new cases check that a list comma survives (牛奶、鸡蛋、面包; りんご、バナナ、みかん; 護照，機票):
both models wrote 、 between the items in 36 of 36 samples (the server's zh-038 fails only on
its known Traditional script).

## Tried and dropped: prompt changes

1. Rule 1 rewritten ("punctuate by grammar, not by pauses", no comma after an opening phrase or
   topic, remove pause commas): dev cases built-in 26 → 27 of 33, server 17 → 16. No effect.
2. The same plus two examples with pause commas removed: server 17 → 20, but the built-in model
   began writing Simplified Chinese in Traditional characters and dropping numbered lists
   (zh-Hans 79% → 59% over all cases, 31 regressions). The prompt is unchanged.

## Reproduce

```sh
npm run eval -- polish --split all --n 3 --llama-model <Qwen3.5-4B-Q4_K_M.gguf>
npm run eval -- polish --split all --n 3 --ai-url <server>/v1 --ai-model <name>
npm run eval -- polish --split all --tag over-punctuation --n 3 --llama-model <gguf>
```
