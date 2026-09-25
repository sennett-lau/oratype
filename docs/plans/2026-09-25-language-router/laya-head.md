# Laya in Typelite

How Laya works, how Typelite runs it without Python, and what it measured. Part of
[language-router](index.md).

## What Laya is

Laya (Convai Innovations, Apache-2.0) is a "System 1" decision model: it reads a text and a
typed question (`choice`, `score`, or yes/no, which Laya calls `noul`) and returns one
probability per option in a single forward pass. It never generates text. There are three
checkpoints: `convaiinnovations/laya` (ModernBERT-large, English), `laya-typed-decisions`, and
**`convaiinnovations/laya-multilingual`** (mmBERT-base, 307 M backbone + 15 M head, 100+
languages), the one used here. The `laya` pip package (0.3.20) holds the reference code
(`laya/common.py`, `laya/agent.py`).

## Files

| File | Size | Source (pinned commit) |
|---|---|---|
| `laya-multilingual-Q8_0.gguf` | 341 MB | `fr0stbit3/laya-multilingual-gguf` @ `65c0eef`, community conversion |
| `laya-multilingual-head.safetensors` | 60 MB | same repository |
| `laya-multilingual-tokenizer.json` | 34 MB | `convaiinnovations/laya-multilingual` @ `e4e9ddf`, `tokenizer/tokenizer.json` |

SHA-256 values are in `src-tauri/src/router/models.rs`. The head file's 36 tensors are
bit-identical to the non-encoder tensors of the original `model.safetensors` (checked), and its
`laya.config` metadata equals the original `rl_agent_config.json`: `max_len` 1024,
`head_max_len` 256, `temperature` [1, 1, 1], no per-option-count temperatures.

Head tensors (all F32, `d` = 768): `type_emb.weight` [3, d]; per layer N in 0..2
`head.layers.N.self_attn.in_proj_weight` [3d, d], `in_proj_bias` [3d],
`self_attn.out_proj.weight` [d, d], `.bias` [d], `norm1`/`norm2` `.weight`/`.bias` [d],
`linear1.weight` [3072, d], `.bias` [3072], `linear2.weight` [d, 3072], `.bias` [d];
`scorer.0` (LayerNorm) [d]; `scorer.1` [d, d] + [d]; `scorer.3` [1, d] + [1];
`act_head.*` (unused here); `temperature` [3] (unused; the config's values count).

## The sequence

Each question is its own sequence, and the question and its options go **through the
backbone together with the text**:

```text
<bos> "{type} question: {instructions}" <eos> <mask> " {option 0}" <mask> " {option 1}" ... <eos> {text} <eos>
```

- `{type}` is `choice`, `score` or `noul`. Options read `key` or `key: description` (choice),
  `level i: description` (score), and `false: …` / `true: …` with the defaults "no, the
  statement does not hold" / "yes, the statement holds" (yes/no).
- Every option is cut to 48 tokens. If the options leave fewer than 16 of the 256 head tokens,
  each option is cut to `max(4, (256 − 16) / n)`. The question gets what is left, at least 8.
- The text fills the rest up to 1024 tokens (cut at the end). `<mask>` in any text is replaced
  by a space. No padding: one sequence per request.
- Tokenizer: mmBERT's 256k BPE. Added tokens (`<eos>`, `\n`, tab runs, ...) are cut out of the
  raw text first; each other piece gets spaces → `▁`, a leading `▁`, a split before every `▁`,
  then BPE with byte fallback. Details in `router/tokenizer.rs`.

## The head

With `h` = the backbone's last hidden states (after its final norm), one row per token:

```text
x = h + type_emb[type]                               (every row)
2 x:  x = x + MHA(LN1(x))        12 heads of 64, softmax(q·k / 8), no mask
      x = x + W2 · ReLU(W1 · LN2(x) + b1) + b2       3072 wide
z_i = w3 · GELU(W · LN(x[marker_i]) + b) + b3        one logit per option (exact-erf GELU)
p   = softmax(z / T)                                 T = 1.0, kept within 0.5 to 5.0
```

LayerNorm eps is 1e-5 everywhere. The answer is `argmax p`; Laya's calibrated confidence is
`max p` (its entropy-based "confidence" is not calibrated). Laya-multilingual ships
uncalibrated (T = 1); its model card reports over-confidence and advises refitting T per
question type and option count.

## Rust design

```text
router/tokenizer.rs   tokenizer.json → ids (no special tokens)
router/head.rs        Question, build_sequence, LayaHead::logits, probabilities
router/embeddings.rs  second llama-server (ServerManager::with_spec) + POST /embedding client
router/laya.rs        LayaModel (tokenizer + head), LayaClassifier::classify, LayaRouter
router/chars.rs       CharRouter (no model)
router/models.rs      the three files for the shared download code
router/mod.rs         Language, LanguageGuess, LanguageRouter trait
```

- `LayaClassifier::classify(text, question) -> Vec<(option, probability)>`: build the
  sequence, `POST {origin}/embedding {"content": [ids]}` (the server adds no tokens to an id
  input and returns one 768-wide row per id), then the head on a blocking thread.
- The last head layer computes keys and values for all rows but queries, attention output and
  feed-forward only for the option markers. Matrix products call `cblas_sgemm` in Apple's
  Accelerate framework on macOS (a system framework, no new library); elsewhere plain Rust
  loops split over up to 8 threads. A test checks the two agree.
- `llm::builtin::ServerManager` gained `with_spec(launcher, label, args)`; `ServerManager::new`
  is the Built-in AI server with its arguments unchanged. The Laya server runs with
  `--embeddings --pooling none`, context, batch and micro-batch 1024 (a bidirectional model
  needs the whole input in one micro-batch), on 127.0.0.1 with its own random key and pid file.

## Does llama.cpp match the reference?

llama.cpp b11177 loads the GGUF as `modern-bert` (sliding window 128, every third layer
global, RoPE 160000 for both). Its per-token hidden states were compared with PyTorch
(`transformers` 5.17, F32) on 22 sequences of 62 to 575 tokens, and the reference head was run
on them:

| GGUF | min cosine (any token) | median cosine | largest probability change |
|---|---|---|---|
| F16 (629 MB) | 0.9996 | 1.0000 | 0.002 |
| Q8_0 (341 MB) | 0.932 | 0.9998 | 0.028 |
| Q4_K_M (249 MB) | 0.05 | 0.98 | 0.165 |

The 575-token sequence (longer than the sliding window) matches as well as the short ones, so
the sliding-window attention is right. Q8_0 is the one to download. Through Typelite's own
path (Rust tokenizer, Q8_0 server, Rust head) the largest difference from the `laya` package
on the fixtures was 0.028. The Rust tokenizer and sequence builder give the same ids as the
package on every fixture, and the Rust head on PyTorch's hidden states gives its logits
within 1e-3.

## Measured routing (zero-shot)

18 sentences (4 English, 6 Mandarin of which 2 Traditional, 6 Cantonese of which 2
code-switched, 1 code-switched Mandarin, 1 French). Wordings tried with the `laya` package:

| Question | Right |
|---|---|
| "Which language is this text written in?" english / mandarin (with description) / cantonese / other | 7 / 18 (everything "mandarin") |
| same, plain option names | 5 / 18 (everything "english") |
| "Which language is the speaker using?" plain keys | 6 / 18 |
| "Classify the language of the dictated text." with descriptions (`laya::language_question`) | 10 / 18 |

Yes/no forms ("The text is written in English.") gave 0.09 for English text and 0.66 for
Chinese text. No wording ever picked Cantonese for a Cantonese sentence, and the
two-option "Cantonese or Mandarin?" question stayed near 0.5 for both. The model card itself
routes between its checkpoints by script, before the forward pass, because the model's
confidence "gives no warning".

End to end in Typelite (`laya_routes_english_mandarin_and_cantonese`, Q8_0, M1 Pro):

| Router | Right | Time per sentence |
|---|---|---|
| Laya | 9 / 18 (2 of 4 English, 7 of 7 Mandarin, 0 of 6 Cantonese, 0 of 1 other) | median 62 ms, max 103 ms |
| Characters | 18 / 18 | microseconds |

Release build. Per sentence: tokenizing and the backbone in llama-server (Metal) plus the JSON
answer (one 768-wide row per token) take most of it; the head is about 10 to 40 ms with
Accelerate on macOS (250 ms with the portable loops, 4.5 s in a debug build). Laya server
start: about 9 s (Metal shaders already cached); head and tokenizer load: 0.8 s. The server's
resident memory is about 0.5 GB; Typelite itself holds the 60 MB head and the tokenizer tables.

## Considered

- **ONNX export** (`receptron/laya-onnx`) with an ONNX runtime: a second ML runtime in the app.
- **Linking llama.cpp or ggml into Typelite** to run backbone and head in-process: duplicate
  ggml symbols with whisper.cpp.
- **Running the head inside llama.cpp**: not supported for this architecture; the head is
  small enough for plain Rust.
- **The `tokenizers` crate**: correct by construction but a large dependency tree for one BPE;
  the fixtures show the small Rust version matches.
- **Q4_K_M or F16 backbone**: see the table above.
