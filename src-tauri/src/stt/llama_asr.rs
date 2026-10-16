//! Plan `language-model-suggestions`: built-in speech that runs in `llama-server` (Qwen3-ASR).
//!
//! Typelite starts a second `llama-server` with the model and its audio projector
//! (`llm::builtin::speech_server`, separate from the AI server so both stay loaded) and uploads
//! each recording to its OpenAI-compatible `/v1/audio/transcriptions` endpoint with the same
//! client as any speech server. That client removes Qwen3-ASR's `language X<asr_text>` tag and
//! reports the language, so routing and the Chinese script conversion work as for a Qwen3-ASR
//! server (plan `qwen3-asr-support`).
//!
//! The server is started when this engine becomes the speech engine (and at app start), so a
//! dictation normally finds it ready; a server that died is started again by the next one.

use async_trait::async_trait;

use crate::error::AppError;
use crate::llm::builtin::{self, Endpoint, ServerError};
use crate::storage::{AppConfig, SpeechPreset};

use super::models::{self, AsrModel};
use super::silence::{log_decision, NoSpeechGuard, VoiceActivity};
use super::whisper_compat::{WhisperCompatConfig, WhisperCompatProvider};
use super::{SttConfig, SttProvider, TranscriptEvent};

/// What a built-in preset with a `llama-server` speech model needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LlamaAsrConfig {
    /// Shown in logs; the speech preset name.
    pub provider_name: String,
    /// The model id, sent as the server's model alias.
    pub model_id: String,
    pub model_file: String,
    pub projector_file: String,
}

/// The settings for `preset`, or why it cannot run: no model downloaded, or not a
/// `llama-server` model.
pub fn config_for(preset: &SpeechPreset) -> Result<LlamaAsrConfig, String> {
    let model = models::asr_model(&preset.model)
        .ok_or_else(|| format!("{} is not a built-in Qwen3-ASR model", preset.model))?;
    if preset.model_file.trim().is_empty() {
        return Err(super::builtin::MODEL_MISSING_MESSAGE.to_string());
    }
    Ok(LlamaAsrConfig {
        provider_name: preset.name.clone(),
        model_id: model.file.id.to_string(),
        model_file: preset.model_file.trim().to_string(),
        projector_file: model.projector.file_name.to_string(),
    })
}

async fn ensure_server(config: &LlamaAsrConfig) -> Result<Endpoint, ServerError> {
    builtin::speech_server()
        .ensure_with(
            &config.model_file,
            Some(&config.projector_file),
            &config.model_id,
        )
        .await
}

fn start_error(error: ServerError) -> String {
    format!("Built-in speech (Qwen3-ASR) could not start: {error}")
}

/// A client that never goes through a proxy: the server is on 127.0.0.1.
fn local_client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap_or_default()
}

/// Upload settings for the running server.
fn upload_config(config: &LlamaAsrConfig, endpoint: &Endpoint) -> WhisperCompatConfig {
    WhisperCompatConfig {
        provider_name: config.provider_name.clone(),
        endpoint: format!("{}/audio/transcriptions", endpoint.base_url),
        model: config.model_id.clone(),
        verbose_json: false,
    }
}

/// Max audio buffer, as for an upload server: about 12 minutes at 16 kHz.
const MAX_AUDIO_BYTES: usize = 24 * 1024 * 1024;

/// Speech provider for a built-in Qwen3-ASR preset.
pub struct LlamaAsrProvider {
    config: LlamaAsrConfig,
    stt_config: Option<SttConfig>,
    audio_buffer: Vec<u8>,
    upload_probe: Option<crate::timing::UploadProbe>,
    detected_language: Option<String>,
    answered_as_qwen3_asr: bool,
}

impl LlamaAsrProvider {
    pub fn new(config: LlamaAsrConfig) -> Self {
        Self {
            config,
            stt_config: None,
            audio_buffer: Vec::new(),
            upload_probe: None,
            detected_language: None,
            answered_as_qwen3_asr: false,
        }
    }
}

#[async_trait]
impl SttProvider for LlamaAsrProvider {
    async fn connect(&mut self, config: &SttConfig) -> Result<(), AppError> {
        self.stt_config = Some(config.clone());
        self.audio_buffer.clear();
        // Start the server while the user speaks, in case it is not running yet.
        let server_config = self.config.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(error) = ensure_server(&server_config).await {
                tracing::warn!("{}", start_error(error));
            }
        });
        tracing::info!(
            "{} provider ready (buffering mode)",
            self.config.provider_name
        );
        Ok(())
    }

    async fn send_audio(&mut self, chunk: &[u8]) -> Result<(), AppError> {
        if self.audio_buffer.len() + chunk.len() > MAX_AUDIO_BYTES {
            return Err(AppError::Config(format!(
                "{}: audio exceeds maximum length (~12 min)",
                self.config.provider_name
            )));
        }
        self.audio_buffer.extend_from_slice(chunk);
        Ok(())
    }

    async fn recv_transcript(&mut self) -> Result<Option<TranscriptEvent>, AppError> {
        // Transcribed in disconnect(), like an upload server.
        std::future::pending().await
    }

    async fn disconnect(&mut self) -> Result<Option<String>, AppError> {
        self.detected_language = None;
        self.answered_as_qwen3_asr = false;
        let Some(config) = self.stt_config.clone() else {
            return Ok(None);
        };
        let audio = std::mem::take(&mut self.audio_buffer);
        if audio.is_empty() {
            return Ok(None);
        }
        // The voice check before the server is involved, so a click never starts it.
        let activity = VoiceActivity::measure(&audio, config.sample_rate);
        if !activity.has_speech() {
            if let Some(probe) = &self.upload_probe {
                probe.note_audio(audio.len(), config.sample_rate);
            }
            log_decision(
                &self.config.provider_name,
                &activity,
                Some(NoSpeechGuard::VoiceCheck),
            );
            return Ok(None);
        }

        let endpoint = ensure_server(&self.config)
            .await
            .map_err(|error| AppError::Config(start_error(error)))?;
        let mut upload = WhisperCompatProvider::with_client(
            upload_config(&self.config, &endpoint),
            local_client(),
        );
        if let Some(probe) = &self.upload_probe {
            upload.set_upload_probe(probe.clone());
        }
        upload
            .connect(&SttConfig {
                api_key: endpoint.api_key.clone(),
                ..config
            })
            .await?;
        upload.send_audio(&audio).await?;
        let text = upload.disconnect().await?;
        self.detected_language = upload.detected_language();
        self.answered_as_qwen3_asr = upload.answered_as_qwen3_asr();
        Ok(text)
    }

    fn name(&self) -> &str {
        &self.config.provider_name
    }

    fn set_upload_probe(&mut self, probe: crate::timing::UploadProbe) {
        self.upload_probe = Some(probe);
    }

    fn detected_language(&self) -> Option<String> {
        self.detected_language.clone()
    }

    fn answered_as_qwen3_asr(&self) -> bool {
        self.answered_as_qwen3_asr
    }
}

/// The automatic test after setup and Settings' check: start the server and transcribe one second
/// of silence. Returns the time of the transcription request in milliseconds.
pub async fn self_test(config: &LlamaAsrConfig) -> Result<u32, String> {
    let endpoint = ensure_server(config).await.map_err(start_error)?;
    let upload = upload_config(config, &endpoint);
    let silence = vec![0u8; 32_000];
    let wav = WhisperCompatProvider::build_wav(&silence, 16_000);
    let part = reqwest::multipart::Part::bytes(wav)
        .file_name("test.wav")
        .mime_str("audio/wav")
        .map_err(|error| error.to_string())?;
    let form = reqwest::multipart::Form::new()
        .text("model", upload.model.clone())
        .part("file", part);
    let started = std::time::Instant::now();
    let response = local_client()
        .post(&upload.endpoint)
        .header("Authorization", format!("Bearer {}", endpoint.api_key))
        .multipart(form)
        .timeout(std::time::Duration::from_secs(60))
        .send()
        .await
        .map_err(|error| format!("Built-in speech (Qwen3-ASR) did not answer: {error}"))?;
    let elapsed = started.elapsed().as_millis() as u32;
    if !response.status().is_success() {
        return Err(format!(
            "Built-in speech (Qwen3-ASR) answered HTTP {}",
            response.status().as_u16()
        ));
    }
    Ok(elapsed)
}

/// The `llama-server` model the speech server should run for `config`: the active speech preset's
/// when it is a built-in Qwen3-ASR preset that passed its test.
pub fn wanted_model(config: &AppConfig) -> Option<LlamaAsrConfig> {
    let preset = config.active_speech_preset();
    if !preset.is_builtin_whisper() || preset.verified_at.is_none() {
        return None;
    }
    config_for(preset).ok()
}

/// Starts or stops the speech server to match `config` (at app start and after the speech engine
/// changed). Starting runs in the background.
pub fn sync_with_config(config: &AppConfig) {
    match wanted_model(config) {
        Some(wanted) => {
            tauri::async_runtime::spawn(async move {
                if let Err(error) = ensure_server(&wanted).await {
                    tracing::warn!("{}", start_error(error));
                }
            });
        }
        None => builtin::speech_server().stop(),
    }
}

/// The model of a built-in speech preset that runs in `llama-server`, if it is one.
pub fn asr_model_of(preset: &SpeechPreset) -> Option<&'static AsrModel> {
    if !preset.is_builtin_whisper() {
        return None;
    }
    models::asr_model(&preset.model)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qwen_preset(model_file: &str) -> SpeechPreset {
        SpeechPreset::builtin_whisper(models::QWEN3_ASR_MODEL_ID, model_file)
    }

    #[test]
    fn a_qwen3_asr_preset_names_both_files() {
        let config = config_for(&qwen_preset("Qwen3-ASR-1.7B-Q8_0.gguf")).unwrap();
        assert_eq!(config.model_id, "qwen3-asr-1.7b");
        assert_eq!(config.model_file, "Qwen3-ASR-1.7B-Q8_0.gguf");
        assert_eq!(config.projector_file, "mmproj-Qwen3-ASR-1.7B-Q8_0.gguf");
        assert_eq!(
            config_for(&qwen_preset("")).unwrap_err(),
            super::super::builtin::MODEL_MISSING_MESSAGE
        );
        assert!(config_for(&SpeechPreset::builtin_whisper(
            "large-v3-turbo",
            "ggml-large-v3-turbo-q5_0.bin"
        ))
        .is_err());
    }

    #[test]
    fn uploads_go_to_the_running_server_without_verbose_json() {
        let config = config_for(&qwen_preset("Qwen3-ASR-1.7B-Q8_0.gguf")).unwrap();
        let upload = upload_config(
            &config,
            &Endpoint {
                base_url: "http://127.0.0.1:4321/v1".into(),
                api_key: "k".into(),
            },
        );
        assert_eq!(
            upload.endpoint,
            "http://127.0.0.1:4321/v1/audio/transcriptions"
        );
        assert_eq!(upload.model, "qwen3-asr-1.7b");
        assert!(!upload.verbose_json);
    }

    #[test]
    fn the_server_runs_only_for_a_tested_qwen3_asr_preset_in_use() {
        let mut config = AppConfig::default();
        let preset =
            config.install_builtin_whisper(models::QWEN3_ASR_MODEL_ID, "Qwen3-ASR-1.7B-Q8_0.gguf");
        assert_eq!(wanted_model(&config), None);
        config.mark_speech_verified(&preset, 1);
        assert_eq!(
            wanted_model(&config).map(|wanted| wanted.model_id),
            Some("qwen3-asr-1.7b".to_string())
        );
        assert!(asr_model_of(&config.active_speech_preset()).is_some());

        let whisper =
            config.install_builtin_whisper("large-v3-turbo", "ggml-large-v3-turbo-q5_0.bin");
        config.mark_speech_verified(&whisper, 2);
        assert_eq!(wanted_model(&config), None);
        assert!(asr_model_of(&config.active_speech_preset()).is_none());
    }
}
