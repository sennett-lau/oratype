//! Plan `language-router`: decides which language a transcript is in (English, Mandarin,
//! Cantonese or another one), so AI polish can later pick a language-specific model or prompt.
//!
//! This is groundwork. Nothing here is wired into the dictation pipeline or Settings yet:
//!
//! - [`chars::CharRouter`] decides from the characters alone (Chinese share, Cantonese-only
//!   characters, Traditional vs Simplified). It needs no model and answers in microseconds.
//! - [`laya::LayaRouter`] asks Laya, a "System 1" decision model (Apache-2.0), through a
//!   second `llama-server` that runs Laya's mmBERT backbone as an embedding model
//!   ([`embeddings`]). Laya's decision head runs here in plain Rust ([`head`]) on the
//!   backbone's per-token hidden states, with Laya's own tokenizer ([`tokenizer`]).
//! - [`models`] lists the three Laya files (backbone GGUF, head, tokenizer) for the shared
//!   download and checksum code.
//!
//! See `docs/plans/2026-09-25-language-router/laya-head.md` for the head's maths and the
//! measured accuracy: zero-shot Laya is not good at telling these languages apart, so the
//! character router is the one to use first.

pub mod chars;
pub mod embeddings;
pub mod head;
pub mod models;
pub mod tokenizer;

use async_trait::async_trait;

/// The languages the router tells apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    English,
    /// Standard written Chinese (Simplified or Traditional characters).
    Mandarin,
    /// Written colloquial Cantonese (嘅, 咗, 喺, ...).
    Cantonese,
    Other,
}

impl Language {
    pub const ALL: [Language; 4] = [
        Language::English,
        Language::Mandarin,
        Language::Cantonese,
        Language::Other,
    ];

    /// A stable lower-case id ("english", "mandarin", "cantonese", "other").
    pub fn id(self) -> &'static str {
        match self {
            Language::English => "english",
            Language::Mandarin => "mandarin",
            Language::Cantonese => "cantonese",
            Language::Other => "other",
        }
    }
}

/// Which characters a Chinese text is written in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChineseScript {
    Simplified,
    Traditional,
    /// No character that exists only in one of the two forms.
    Unknown,
}

/// One routing decision.
#[derive(Debug, Clone, PartialEq)]
pub struct LanguageGuess {
    pub language: Language,
    /// 0.0 to 1.0. For [`chars::CharRouter`] a rule-based score, for [`laya::LayaRouter`] the
    /// probability Laya gives the answer.
    pub confidence: f32,
    /// Set when the text has Chinese characters.
    pub script: Option<ChineseScript>,
}

/// Why a router could not decide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RouterError {
    /// The Laya server could not be started or reached.
    Server(String),
    /// The server answered with something unexpected.
    BadResponse(String),
    /// A Laya file is missing or does not load.
    Model(String),
    /// The question does not fit Laya's option budget.
    Question(String),
}

impl std::fmt::Display for RouterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RouterError::Server(reason) => write!(f, "Language router server: {reason}"),
            RouterError::BadResponse(reason) => {
                write!(f, "Language router server answered badly: {reason}")
            }
            RouterError::Model(reason) => write!(f, "Language router model: {reason}"),
            RouterError::Question(reason) => write!(f, "Language router question: {reason}"),
        }
    }
}

impl std::error::Error for RouterError {}

/// Decides the language of one transcript.
#[async_trait]
pub trait LanguageRouter: Send + Sync {
    async fn route(&self, text: &str) -> Result<LanguageGuess, RouterError>;
}
