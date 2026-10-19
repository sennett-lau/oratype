# Built-in speech recognition

Built-in runs [whisper.cpp](https://github.com/ggml-org/whisper.cpp) inside OraType, or Qwen3-ASR
in llama.cpp's `llama-server`, which ships inside OraType. No server to set up, no account, no
key: your audio never leaves your computer.

## Set it up

Pick **Built-in** in the speech setup step, or in **Settings → Speech**, choose a model and press
**Set up** (or **Download** in Settings). OraType downloads the model, checks its SHA-256 and
loads it. A download that breaks off resumes where it stopped.

## Models

| Model | File | Size | Offered on |
|---|---|---|---|
| Best accuracy | whisper large-v3-turbo, q5_0 (`ggml-large-v3-turbo-q5_0.bin`) | 574 MB | Apple Silicon with 8 GB of memory or more |
| Faster | whisper small, q5_1 (`ggml-small-q5_1.bin`) | 190 MB | Every Mac |
| Qwen3-ASR | Qwen3-ASR 1.7B, Q8_0, with its audio projector (`Qwen3-ASR-1.7B-Q8_0.gguf`, `mmproj-Qwen3-ASR-1.7B-Q8_0.gguf`) | 2.5 GB | Apple Silicon with 8 GB of memory or more |

- The whisper models come from the official whisper.cpp repository on Hugging Face, Qwen3-ASR
  from ggml-org's (Apache-2.0). They are stored in
  `~/Library/Application Support/dev.oratype.mac/models/`.
- When you tell OraType the languages you speak (onboarding or Settings → Speech → **Your
  languages**) and it has a suggested setup for them, that model is chosen and tagged **Suggested
  for your languages**. Today: Qwen3-ASR for any combination with Cantonese.
- OraType checks the chip, the memory and the free disk space (1.1 × the model size) and offers
  only the models that suit your computer. With both offered, Best accuracy is selected.
- **Delete** in Settings removes a model file.

## Hardware notes

- On Apple Silicon, whisper.cpp uses the GPU through Metal. On an M1 Pro a short clip takes
  about 2 seconds with large-v3-turbo (see [Benchmarks](../benchmarks/README.md)); newer chips are
  faster.
- Intel Macs have no GPU that whisper.cpp uses well, so they get only the small model. A GPU
  computer on your network or a cloud service is much faster there; see
  [OpenAI-compatible](openai-compatible.md).
- The model loads when a recording starts (while you speak), stays in memory, and is freed
  after 10 minutes without use.
- Qwen3-ASR runs in its own `llama-server`, next to the Built-in AI one, on 127.0.0.1 with a
  random port and key. It starts when Qwen3-ASR is in use, uses about 3 GB of memory, and stops
  when OraType quits or you pick another model. On an M1 Pro a short clip takes about 0.4 s.

## Languages

Leave **Settings → Speech → Spoken language** on auto-detect to have whisper detect it; it handles mixed English, Mandarin and
Cantonese (Cantonese comes out as standard written Chinese). A fixed language saves the detection
pass and can help a short clip. Qwen3-ASR detects the language itself and writes Cantonese as it
was said; see [Cantonese](../languages/cantonese.md). See [Languages](../languages/README.md).

## No speech, no text

Before recognition, OraType checks that the recording holds at least 200 ms of voice, so a key
click or a bump of the microphone is not sent. After recognition it drops segments whisper itself
marks as probably not speech, and a lone "Thank you." from a very short clip. The pill then shows
"Didn't catch that" and nothing is pasted.
