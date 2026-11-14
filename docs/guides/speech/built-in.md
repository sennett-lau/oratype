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
| Qwen3-ASR-1.7B-HK-Cantonese | Qwen3-ASR 1.7B fine-tuned for Hong Kong Cantonese, version 2, Q6_K, with its Q8_0 audio projector (`Qwen3-ASR-1.7B-Cantonese-HK.Q6_K.gguf`, `Qwen3-ASR-1.7B-Cantonese-HK.mmproj-Q8_0.gguf`) | 1.8 GB | Apple Silicon with 8 GB of memory or more |
| Qwen3-ASR-0.6B-HK-Cantonese | Qwen3-ASR 0.6B fine-tuned for Hong Kong Cantonese, Q6_K, with its Q8_0 audio projector (`Qwen3-ASR-0.6B-Cantonese-HK.Q6_K.gguf`, `Qwen3-ASR-0.6B-Cantonese-HK.mmproj-Q8_0.gguf`) | 709 MB | Apple Silicon with 8 GB of memory or more |

- The whisper models come from the official whisper.cpp repository on Hugging Face,
  Qwen3-ASR-1.7B-HK-Cantonese from
  [joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF](https://huggingface.co/joe2643/Qwen3-ASR-1.7B-Cantonese-HK-GGUF)
  and Qwen3-ASR-0.6B-HK-Cantonese from
  [joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF](https://huggingface.co/joe2643/Qwen3-ASR-0.6B-Cantonese-HK-GGUF)
  (both Apache-2.0), fine-tunes of Qwen3-ASR 1.7B and 0.6B by the OraType team. They are stored in
  `~/Library/Application Support/dev.oratype.mac/models/`.
- The fine-tune writes Hong Kong Cantonese in Hong Kong Traditional characters and keeps English
  words in English. It is the Q6_K file: its Q4_K_M wrote some English clips in Chinese, and its
  Q8_0 is slower with no better results
  ([report](../../benchmarks/reports/2026-11-10-hk-cantonese-asr/README.md)). If you set up
  Qwen3-ASR with OraType 1.2.1 or earlier you have the stock Qwen3-ASR files: they keep working,
  and Settings shows **Update** to download the new model (the old files are deleted once the new
  ones pass their test).
- Qwen3-ASR-0.6B-HK-Cantonese is the smaller fine-tune for Macs with less memory. On OraType's
  speech clips it passed as many as the 1.7B, one Cantonese clip fewer, in about half the time
  and with about 1 GB less memory
  ([report](../../benchmarks/reports/2026-11-14-hk-cantonese-0.6b/README.md)). It is the Q6_K
  file, for the same reasons as the 1.7B.
- When you tell OraType the languages you speak (onboarding or Settings → General → **Your
  languages**) and it has a suggested setup for them, that model is chosen and tagged **Suggested
  for your languages**. Today: for any combination with Cantonese, Qwen3-ASR-1.7B-HK-Cantonese,
  or Qwen3-ASR-0.6B-HK-Cantonese on a Mac with less than 16 GB of memory. You can still pick the
  other one, and Home does not ask you to switch.
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
