//! Plan `language-router`: Laya as a whole. The tokenizer and head run here; the backbone runs
//! in the language router's `llama-server` (see `embeddings`).
//!
//! ```text
//! text + question ──tokenizer──> ids ──llama-server /embedding──> hidden states
//!                                               ──head──> one probability per option
//! ```

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;

use crate::llm::builtin::Endpoint;

use super::chars;
use super::embeddings::{laya_server, EmbeddingsClient, TokenStates};
use super::head::{self, LayaHead, Question, Sequence, SpecialTokens};
use super::models;
use super::tokenizer::Tokenizer;
use super::{Language, LanguageGuess, LanguageRouter, RouterError};

/// The model name the language router's server is started with.
pub const LAYA_ALIAS: &str = "laya-multilingual";

/// The tokenizer and head (the backbone is in the server).
pub struct LayaModel {
    tokenizer: Tokenizer,
    special: SpecialTokens,
    head: LayaHead,
}

impl LayaModel {
    /// Loads the head and tokenizer from `dir` under their model-table names. Takes about a
    /// second (the tokenizer file is 34 MB of JSON); call it off the async runtime.
    pub fn load(dir: &Path) -> Result<Self, RouterError> {
        Self::from_files(
            &dir.join(models::HEAD.file.file_name),
            &dir.join(models::TOKENIZER.file.file_name),
        )
    }

    pub fn from_files(head: &Path, tokenizer: &Path) -> Result<Self, RouterError> {
        let tokenizer = Tokenizer::from_file(tokenizer)?;
        let special = SpecialTokens::from_tokenizer(&tokenizer)?;
        Ok(Self {
            tokenizer,
            special,
            head: LayaHead::from_file(head)?,
        })
    }

    /// The model input for one question about `text`.
    pub fn sequence(&self, text: &str, question: &Question) -> Result<Sequence, RouterError> {
        head::build_sequence(
            &self.tokenizer,
            self.special,
            text,
            question,
            self.head
                .config
                .max_len
                .min(super::embeddings::EMBEDDING_CONTEXT_TOKENS),
            self.head.config.head_max_len,
        )
    }

    /// `(option key, probability)` in option order, from the backbone's hidden states.
    pub fn decide(
        &self,
        sequence: &Sequence,
        states: &TokenStates,
        question: &Question,
    ) -> Result<Vec<(String, f32)>, RouterError> {
        let logits = self.head.logits(states, &sequence.markers, sequence.kind)?;
        let temperature = self
            .head
            .config
            .temperature_for(sequence.kind, logits.len());
        let probabilities = head::probabilities(&logits, temperature);
        Ok(question
            .options
            .iter()
            .map(|option| option.key.clone())
            .zip(probabilities)
            .collect())
    }
}

/// Laya with a running backbone server.
pub struct LayaClassifier {
    model: Arc<LayaModel>,
    client: EmbeddingsClient,
}

impl LayaClassifier {
    pub fn new(model: Arc<LayaModel>, endpoint: Endpoint) -> Self {
        Self {
            model,
            client: EmbeddingsClient::new(endpoint),
        }
    }

    /// Starts (or reuses) the language router's server for the backbone in `models_dir` and
    /// loads the head and tokenizer from there. The server manager's paths must be set first
    /// (`laya_server().set_paths`).
    pub async fn start(models_dir: &Path) -> Result<Self, RouterError> {
        let endpoint = laya_server()
            .ensure(models::BACKBONE.file.file_name, LAYA_ALIAS)
            .await
            .map_err(|error| RouterError::Server(error.to_string()))?;
        let dir = models_dir.to_path_buf();
        let model = tokio::task::spawn_blocking(move || LayaModel::load(&dir))
            .await
            .map_err(|error| RouterError::Model(error.to_string()))??;
        Ok(Self::new(Arc::new(model), endpoint))
    }

    /// `(option key, probability)` for one question about `text`, in option order.
    pub async fn classify(
        &self,
        text: &str,
        question: &Question,
    ) -> Result<Vec<(String, f32)>, RouterError> {
        let started = Instant::now();
        let sequence = self.model.sequence(text, question)?;
        let states = self.client.token_states(&sequence.ids).await?;
        let backbone_ms = started.elapsed().as_millis();
        let model = self.model.clone();
        let question = question.clone();
        let head_started = Instant::now();
        let answer =
            tokio::task::spawn_blocking(move || model.decide(&sequence, &states, &question))
                .await
                .map_err(|error| RouterError::Model(error.to_string()))??;
        tracing::debug!(
            "Laya: backbone {backbone_ms} ms, head {} ms",
            head_started.elapsed().as_millis()
        );
        Ok(answer)
    }
}

/// The question [`LayaRouter`] asks. Of the wordings tried (see the plan's `laya-head.md`)
/// this one scored best, and it is still poor: zero-shot Laya does not tell Cantonese from
/// Mandarin.
pub fn language_question() -> Question {
    Question::choice(
        "Classify the language of the dictated text.",
        &[
            ("english", "English words"),
            ("mandarin", "Chinese characters, standard Mandarin grammar"),
            (
                "cantonese",
                "Chinese characters with Cantonese words such as 嘅 咗 喺 唔 佢 哋",
            ),
            ("other", "any other language"),
        ],
    )
}

/// Routes with Laya's answer to [`language_question`].
pub struct LayaRouter {
    classifier: LayaClassifier,
    question: Question,
}

impl LayaRouter {
    pub fn new(classifier: LayaClassifier) -> Self {
        Self {
            classifier,
            question: language_question(),
        }
    }
}

#[async_trait]
impl LanguageRouter for LayaRouter {
    async fn route(&self, text: &str) -> Result<LanguageGuess, RouterError> {
        let answer = self.classifier.classify(text, &self.question).await?;
        let (key, probability) = answer
            .into_iter()
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .ok_or_else(|| RouterError::BadResponse("no answer".into()))?;
        let language = Language::ALL
            .into_iter()
            .find(|language| language.id() == key)
            .unwrap_or(Language::Other);
        let has_chinese = matches!(language, Language::Mandarin | Language::Cantonese);
        Ok(LanguageGuess {
            language,
            confidence: probability,
            script: has_chinese.then(|| chars::chinese_script(text)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[test]
    fn the_language_question_has_one_option_per_language() {
        let question = language_question();
        let keys: Vec<&str> = question.options.iter().map(|o| o.key.as_str()).collect();
        let ids: Vec<&str> = Language::ALL.iter().map(|l| l.id()).collect();
        assert_eq!(keys, ids);
    }

    #[derive(Deserialize)]
    struct RealSequence {
        name: String,
        state: String,
        #[serde(rename = "type")]
        kind: String,
        instructions: String,
        options: Vec<(String, String)>,
        ids: Vec<u32>,
        markers: Vec<usize>,
        logits: Vec<f32>,
        probs: Vec<f32>,
    }

    /// The fixture's question in this module's form.
    fn question_from(fixture: &RealSequence) -> Question {
        let options: Vec<(&str, &str)> = fixture
            .options
            .iter()
            .map(|(key, description)| (key.as_str(), description.as_str()))
            .collect();
        match fixture.kind.as_str() {
            "choice" => Question::choice(&fixture.instructions, &options),
            "score" => {
                let levels: Vec<&str> = options.iter().map(|(_, level)| *level).collect();
                Question::score(&fixture.instructions, &levels)
            }
            _ => Question::yes_no(&fixture.instructions),
        }
    }

    fn real_sequences() -> Vec<RealSequence> {
        serde_json::from_str(include_str!(
            "../../tests/fixtures/laya/real_sequences.json"
        ))
        .unwrap()
    }

    /// Needs the real head and tokenizer (`TYPELITE_E2E_LAYA_MODEL`, files next to it):
    /// sequences equal `laya.common.build_sequence`, and the head on PyTorch's hidden states
    /// for the "billing" sequence gives PyTorch's probabilities.
    #[test]
    #[ignore = "needs the Laya head and tokenizer files; set TYPELITE_E2E_LAYA_MODEL"]
    fn sequences_and_head_match_the_laya_package_on_real_weights() {
        let dir = models::test_files_dir().expect("TYPELITE_E2E_LAYA_MODEL");
        let model = LayaModel::load(&dir).unwrap();
        for fixture in real_sequences() {
            let question = question_from(&fixture);
            let sequence = model.sequence(&fixture.state, &question).unwrap();
            assert_eq!(sequence.ids, fixture.ids, "{}", fixture.name);
            assert_eq!(sequence.markers, fixture.markers, "{}", fixture.name);
        }
        let billing = real_sequences()
            .into_iter()
            .find(|fixture| fixture.name == "billing")
            .unwrap();
        let bytes = include_bytes!("../../tests/fixtures/laya/billing_hidden.f32");
        let data: Vec<f32> = bytes
            .chunks_exact(4)
            .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
            .collect();
        let states = TokenStates {
            rows: billing.ids.len(),
            dim: data.len() / billing.ids.len(),
            data,
        };
        let question = question_from(&billing);
        let sequence = model.sequence(&billing.state, &question).unwrap();
        let started = Instant::now();
        let logits = model
            .head
            .logits(&states, &sequence.markers, sequence.kind)
            .unwrap();
        eprintln!(
            "head: {} ms for {} rows",
            started.elapsed().as_millis(),
            states.rows
        );
        for (got, want) in logits.iter().zip(&billing.logits) {
            assert!(
                (got - want).abs() < 1e-3 * want.abs().max(1.0),
                "{logits:?} vs {:?}",
                billing.logits
            );
        }
        let answer = model.decide(&sequence, &states, &question).unwrap();
        for ((key, got), want) in answer.iter().zip(&billing.probs) {
            assert!((got - want).abs() < 1e-4, "{key}: {got} vs {want}");
        }
        assert_eq!(answer[0].0, "billing");
    }
}
