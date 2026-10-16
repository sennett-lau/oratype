//! Plan `language-model-suggestions`: end-to-end check of built-in Qwen3-ASR. Typelite's own speech
//! server manager starts `llama-server` with the model and its audio projector, and the provider
//! the pipeline picks for a built-in Qwen3-ASR preset transcribes a Cantonese clip.
//!
//! Ignored by default so `cargo test` stays offline. Run it with:
//!
//! ```sh
//! TYPELITE_E2E_QWEN3_ASR_DIR=<folder with Qwen3-ASR-1.7B-Q8_0.gguf and
//! mmproj-Qwen3-ASR-1.7B-Q8_0.gguf> cargo test --test e2e_qwen3_asr -- --ignored --nocapture
//! ```
//!
//! It needs `src-tauri/binaries/llama-server-<triple>` from `scripts/build-llama-server.sh` (or
//! `TYPELITE_LLAMA_SERVER`), and macOS `say` with the Sinji voice for the audio.

use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;

use typelite_lib::llm::builtin;
use typelite_lib::storage::SpeechPreset;
use typelite_lib::stt::{self, SttConfig};

const SAMPLE_RATE: u32 = 16_000;
/// Same chunk size the recorder sends (100 ms of 16-bit mono audio).
const CHUNK_BYTES: usize = (SAMPLE_RATE as usize / 10) * 2;

/// 16 kHz mono PCM of `text` spoken by the macOS voice `voice`.
fn synthesise(text: &str, voice: &str) -> Vec<u8> {
    let path = std::env::temp_dir().join(format!("typelite-e2e-asr-{}.wav", std::process::id()));
    let status = Command::new("/usr/bin/say")
        .args(["-v", voice, "-o"])
        .arg(&path)
        .args(["--data-format=LEI16@16000", text])
        .status()
        .expect("`say` must be available (macOS)");
    assert!(status.success(), "`say` failed");
    let wav = std::fs::read(&path).expect("read synthesised wav");
    let _ = std::fs::remove_file(&path);
    let mut i = 12;
    while i + 8 <= wav.len() {
        let size = u32::from_le_bytes(wav[i + 4..i + 8].try_into().unwrap()) as usize;
        if &wav[i..i + 4] == b"data" {
            return wav[i + 8..(i + 8 + size).min(wav.len())].to_vec();
        }
        i += 8 + size + (size & 1);
    }
    panic!("no data chunk in wav");
}

/// The folder with the Qwen3-ASR model and projector, if set and complete.
fn model_dir() -> Option<PathBuf> {
    let dir = std::env::var("TYPELITE_E2E_QWEN3_ASR_DIR")
        .ok()
        .filter(|v| !v.trim().is_empty())
        .map(PathBuf::from)?;
    let model = stt::models::asr_model(stt::models::QWEN3_ASR_MODEL_ID)?;
    model.installed(&dir).then_some(dir)
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "needs llama-server (scripts/build-llama-server.sh) and TYPELITE_E2E_QWEN3_ASR_DIR"]
async fn builtin_qwen3_asr_keeps_spoken_cantonese() {
    let Some(dir) = model_dir() else {
        println!("built-in Qwen3-ASR: TYPELITE_E2E_QWEN3_ASR_DIR is not set to the model folder, skipping");
        return;
    };
    let pid_file =
        std::env::temp_dir().join(format!("typelite-e2e-asr-{}.pid", std::process::id()));
    builtin::speech_server().set_paths(dir, pid_file);
    assert!(
        builtin::speech_server().binary_available(),
        "build llama-server first: bash scripts/build-llama-server.sh"
    );
    let preset =
        SpeechPreset::builtin_whisper(stt::models::QWEN3_ASR_MODEL_ID, "Qwen3-ASR-1.7B-Q8_0.gguf");

    // The automatic test setup runs; it starts the server.
    let started = Instant::now();
    let config = stt::llama_asr::config_for(&preset).expect("a Qwen3-ASR preset");
    let setup_test_ms = stt::llama_asr::self_test(&config)
        .await
        .expect("the setup test should pass");
    let startup = started.elapsed();

    // A recording through the provider the pipeline picks for this preset.
    let pcm = synthesise("佢頭先話份報告聽日先可以俾我哋。", "Sinji");
    let mut provider = stt::provider_for_preset(&preset, None).expect("Qwen3-ASR provider");
    let stt_config = SttConfig {
        api_key: String::new(),
        language: None,
        sample_rate: SAMPLE_RATE,
    };
    provider.connect(&stt_config).await.expect("connect");
    for chunk in pcm.chunks(CHUNK_BYTES) {
        provider.send_audio(chunk).await.expect("send audio");
    }
    let request_started = Instant::now();
    let text = provider
        .disconnect()
        .await
        .expect("transcription")
        .unwrap_or_default();
    let elapsed = request_started.elapsed();
    let language = provider.detected_language();
    let tagged = provider.answered_as_qwen3_asr();
    builtin::speech_server().stop();
    println!(
        "built-in Qwen3-ASR: start + setup test {startup:?} ({setup_test_ms} ms), transcription {elapsed:?} -> {text:?} ({language:?})"
    );
    // Spoken Cantonese survives (Whisper writes 他剛才說 instead); the tag is removed and reported.
    assert!(text.contains('佢'), "not spoken Cantonese: {text:?}");
    assert!(
        !text.contains("asr_text"),
        "the language tag was not removed: {text:?}"
    );
    assert!(tagged, "the answer should count as Qwen3-ASR");
    assert_eq!(language.as_deref(), Some("yue"));
}
