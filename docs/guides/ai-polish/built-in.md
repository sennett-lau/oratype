# Built-in AI polish

Built-in runs a small language model on your computer with llama.cpp's `llama-server`, which ships
inside OraType. No Ollama, no Terminal, no account: your text never leaves your computer.

## Set it up

Pick **Built-in** in the AI setup step, or in **Settings → AI**, choose a model and press
**Set up** (or **Download** in Settings). OraType downloads the model, checks its SHA-256,
starts the server and sends one test request.

## The models

| Model | File | Size | Offered on |
|---|---|---|---|
| Best quality | Qwen3 4B Instruct 2507, Q4_K_M | 2.5 GB | Apple Silicon with 16 GB of memory or more |
| Faster | Qwen3 1.7B, Q4_K_M | 1.1 GB | Apple Silicon with 8 GB of memory or more |
| Qwen 3.5 | Qwen 3.5 4B, Q4_0 | 2.6 GB | Apple Silicon with 16 GB of memory or more |

- The models come from Unsloth's Hugging Face repositories (Qwen does not publish quantised GGUF
  files of these) and are released by Qwen under the Apache-2.0 licence. They are stored in
  `~/Library/Application Support/dev.oratype.mac/models/`, next to the speech models.
- Qwen 3.5 4B is the AI half of the setup suggested for Cantonese (see **Your languages** in
  onboarding or Settings → General). In the polish evaluations it does better than Qwen3 4B on Cantonese
  and worse on English and Mandarin, so it is suggested only for combinations with Cantonese.
- Qwen 3.5 4B is the Q4_0 file because it answers about a third faster than Q4_K_M on a Mac at
  the same quality ([report](../../benchmarks/reports/2026-10-20-cantonese-model-sizes/README.md)).
  If you set it up with OraType 1.2.0 or earlier you have the Q4_K_M file: it keeps working, and
  Settings shows **Update** to download the faster one (the old file is deleted once the new one
  passes its test).
- Intel Macs, and Macs with less memory than the table says, get no Built-in option; use an
  [OpenAI-compatible](openai-compatible.md) server or service instead.
- **Delete** in Settings removes the model file.

## How it runs

- The server listens only on your computer (127.0.0.1, a random port and a random key that
  changes on every start), uses the GPU, and has thinking turned off (`--reasoning off`).
- It starts when Built-in is in use and stops when OraType quits or you pick another option.
  While it runs it keeps the model in memory (about 3 GB for Best quality).
- The very first start after installing or updating OraType takes a minute or two while macOS
  prepares the GPU code; later starts take a few seconds.
- Right after it starts, OraType has it read the polish instructions once for each of your
  languages, so your first dictation does not wait for that (5–11 s with the 4B models).
- **Fast language switching** (Settings → AI → Built-in AI, on by default) keeps one
  copy of the instructions ready per language, up to three. Switching language then takes under
  a second instead of up to about 10 s, and each extra language uses about 1–2 GB more memory.
  Turn it off on a Mac that is short of memory. See [Benchmarks](../benchmarks/README.md) for
  measured numbers.

## Building OraType yourself

`npm run build:app` builds `llama-server` first (`scripts/build-llama-server.sh`, needs `cmake`).
Without it the app still builds and runs, and Built-in AI says the server is missing. See
[CONTRIBUTING.md](../../../CONTRIBUTING.md#setup).
