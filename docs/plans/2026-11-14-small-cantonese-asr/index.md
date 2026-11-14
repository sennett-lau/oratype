# Qwen3-ASR-0.6B-HK-Cantonese for Macs with less memory

The OraType team's fine-tune of Qwen3-ASR 0.6B for Hong Kong Cantonese with English mixed in
([`joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF`](https://huggingface.co/joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF),
Apache-2.0) becomes a second built-in Qwen3-ASR model, **Qwen3-ASR-0.6B-HK-Cantonese**, next to
Qwen3-ASR-1.7B-HK-Cantonese (plan [`hk-cantonese-asr`](../2026-11-10-hk-cantonese-asr/index.md)).
A Mac with less than 16 GB of memory is suggested the smaller one; the larger one stays the
suggestion everywhere else.

Status: building - 2026-11-14.

## Goals and non-goals

- Cantonese dictation that fits an 8 GB Mac beside the built-in AI model and the user's apps,
  with Hong Kong characters and English words kept, as the 1.7B does.
- Nobody who already uses the 1.7B is asked to switch, whatever their memory.
- Not a goal: replacing the 1.7B. It is a little more accurate on Cantonese and stays the
  suggestion on 16 GB and more.
- Not a goal: a smaller model for other languages.

## Key decisions

- **Q6_K at revision 9b67dd0** with the fine-tune's own Q8_0 projector (709 MB in all). Reason:
  it passed as many of OraType's 41 speech clips as the 1.7B (34) at about half the time and
  1 GB less memory; Q4_K_M missed one more English clip for 5–8% more speed, Q8_0 is slower and
  missed one more Cantonese clip
  ([report](../../benchmarks/reports/2026-11-14-hk-cantonese-0.6b/README.md)).
- **Its own model id, `qwen3-asr-0.6b`.** Reason: it is a different model, not a new version of
  the 1.7B, so it gets its own card, download, test and delete; the 1.7B's id and its
  update path from earlier files stay as they are.
- **Both models are offered on every Apple Silicon Mac** (8 GB or more, room on the disk).
  Reason: the user can still choose the 1.7B on 8 GB; only the suggestion depends on memory.
- **Suggestion: the 0.6B under 16 GB, or when the 1.7B is not offered** (for example not
  enough disk for it). Reason: 16 GB is where the built-in AI already switches to its faster
  model, and both the speech server and the AI server share the memory.
- **Either model counts as the Cantonese setup's speech model.** Reason: Home's "suggested
  setup" card must not push the 0.6B on someone who chose the 1.7B, or the reverse.
- **No change to recognition code.** Both run in the same second `llama-server` with the same
  arguments, write the same `language X<asr_text>` answer, and already write Hong Kong
  Traditional, so the text path is shared.

This changes one decision of plan `hk-cantonese-asr`: the suggestion and the guides no longer
name only the 1.7B for Cantonese. Its reason for rejecting a second model (two Qwen3-ASR models
for the same use, one writing Simplified) does not apply: these two are for different Macs and
both write Hong Kong characters.

## Considered

- Raising the 1.7B to 16 GB so 8 GB Macs see only the 0.6B: hides a model some 8 GB users
  already run, and takes the choice away.
- Replacing the 1.7B with the 0.6B: one Cantonese clip fewer and 1–2 points more character
  error on the model card's test sets, for speed most 16 GB Macs do not need.
- Q4_K_M as the smallest download (611 MB): see above.

## Parts

| File | What it covers |
|---|---|
| [Report](../../benchmarks/reports/2026-11-14-hk-cantonese-0.6b/README.md) | The 0.6B's three files against the 1.7B and the stock 0.6B: clips, speed, memory |

## Open questions

- Real voices on an 8 GB Mac: the clips are synthetic and were measured on a 32 GB M1 Pro.
- Mandarin: like the 1.7B it writes Mandarin speech in Traditional characters, and OraType does
  not convert text that is already Traditional. Users whose list has both Cantonese and
  Simplified Mandarin get this model; watch for reports.
