//! Plan `language-router`: the second `llama-server`, which runs Laya's mmBERT backbone as an
//! embedding model, and the client that asks it for per-token hidden states.
//!
//! - It is the same program as Built-in AI (plan `ai-polish-setup`) and uses the same
//!   [`ServerManager`], so starting, health checks, the random API key, the pid file and log
//!   filtering are shared. Only the arguments differ ([`embeddings_launch_spec`]); the Built-in
//!   AI server is a separate process and is not changed.
//! - `--embeddings --pooling none` makes `POST /embedding` return one vector per input token
//!   (the backbone's last hidden states, after its final norm), which is what Laya's head reads.
//! - The GGUF's own text tokenizer is wrong for this model (the converter stored a default BPE
//!   pre-tokenizer), so the client always sends token ids from [`super::tokenizer`].
//! - The backbone attends in both directions, so one input must fit one micro-batch:
//!   context, batch and micro-batch are all [`EMBEDDING_CONTEXT_TOKENS`], Laya's own limit.

use std::path::Path;
use std::time::Duration;

use crate::llm::builtin::{Endpoint, LaunchSpec, ProcessLauncher, ServerManager};

use super::RouterError;

/// Laya's `max_len`: the longest sequence it is trained on and the server's context size.
pub const EMBEDDING_CONTEXT_TOKENS: usize = 1024;
const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// The arguments of one embedding-server start. Same safety settings as Built-in AI
/// (127.0.0.1, key in the environment, no web UI, offline); no chat settings.
pub fn embeddings_launch_spec(
    binary: &Path,
    model_path: &Path,
    model_alias: &str,
    port: u16,
    api_key: &str,
) -> LaunchSpec {
    let tokens = EMBEDDING_CONTEXT_TOKENS.to_string();
    let args = [
        "--model",
        &model_path.to_string_lossy(),
        "--alias",
        model_alias,
        "--host",
        "127.0.0.1",
        "--port",
        &port.to_string(),
        "--n-gpu-layers",
        "999",
        "--embeddings",
        "--pooling",
        "none",
        "--ctx-size",
        &tokens,
        "--batch-size",
        &tokens,
        "--ubatch-size",
        &tokens,
        "--parallel",
        "1",
        "--no-webui",
        "--offline",
        "--cors-origins",
        "localhost",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    LaunchSpec {
        binary: binary.to_path_buf(),
        args,
        envs: vec![("LLAMA_API_KEY".to_string(), api_key.to_string())],
    }
}

/// The language router's own server manager (not started by anything yet; plan
/// `language-router` wires it to the app's paths and exit handler later).
pub fn laya_server() -> &'static ServerManager {
    static SERVER: std::sync::OnceLock<ServerManager> = std::sync::OnceLock::new();
    SERVER.get_or_init(|| {
        ServerManager::with_spec(
            Box::new(ProcessLauncher::new()),
            "Language router server",
            embeddings_launch_spec,
        )
    })
}

/// Per-token hidden states: `rows` vectors of `dim` numbers, row after row.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenStates {
    pub rows: usize,
    pub dim: usize,
    pub data: Vec<f32>,
}

impl TokenStates {
    pub fn row(&self, index: usize) -> &[f32] {
        &self.data[index * self.dim..(index + 1) * self.dim]
    }
}

/// Asks a running embedding server for hidden states.
pub struct EmbeddingsClient {
    http: reqwest::Client,
    endpoint: Endpoint,
}

impl EmbeddingsClient {
    pub fn new(endpoint: Endpoint) -> Self {
        let http = reqwest::Client::builder()
            .no_proxy()
            .timeout(REQUEST_TIMEOUT)
            .build()
            .unwrap_or_default();
        Self { http, endpoint }
    }

    /// `POST {origin}/embedding` with `{"content": [ids]}`. The server adds no tokens of its
    /// own to a token-id input, so the answer has one row per id.
    pub async fn token_states(&self, ids: &[u32]) -> Result<TokenStates, RouterError> {
        if ids.is_empty() || ids.len() > EMBEDDING_CONTEXT_TOKENS {
            return Err(RouterError::Question(format!(
                "{} tokens (1 to {EMBEDDING_CONTEXT_TOKENS} allowed)",
                ids.len()
            )));
        }
        let url = format!("{}/embedding", origin(&self.endpoint.base_url));
        let body = serde_json::json!({ "content": [ids] });
        let response =
            crate::llm::protocol::apply_auth_headers(self.http.post(url), &self.endpoint.api_key)
                .json(&body)
                .send()
                .await
                .map_err(|error| RouterError::Server(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            let text: String = text.chars().take(200).collect();
            return Err(RouterError::BadResponse(format!(
                "HTTP {}: {text}",
                status.as_u16()
            )));
        }
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|error| RouterError::BadResponse(error.to_string()))?;
        parse_token_states(&value, ids.len())
    }
}

/// `http://127.0.0.1:1234/v1` -> `http://127.0.0.1:1234` (`/embedding` is not under `/v1`).
fn origin(base_url: &str) -> &str {
    let trimmed = base_url.trim_end_matches('/');
    trimmed.strip_suffix("/v1").unwrap_or(trimmed)
}

/// Reads `[{"index": 0, "embedding": [[...], [...]]}]` and checks it has `rows` rows of one
/// width.
pub fn parse_token_states(
    value: &serde_json::Value,
    rows: usize,
) -> Result<TokenStates, RouterError> {
    let bad = |reason: &str| RouterError::BadResponse(reason.to_string());
    let embedding = value
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("embedding"))
        .and_then(|embedding| embedding.as_array())
        .ok_or_else(|| bad("no embedding in the answer"))?;
    if embedding.len() != rows {
        return Err(RouterError::BadResponse(format!(
            "{} rows for {rows} tokens (is the server running with --pooling none?)",
            embedding.len()
        )));
    }
    let mut dim = 0;
    let mut data = Vec::new();
    for row in embedding {
        let row = row.as_array().ok_or_else(|| bad("a row is not a list"))?;
        if dim == 0 {
            dim = row.len();
            data.reserve(dim * rows);
        } else if row.len() != dim {
            return Err(bad("rows of different widths"));
        }
        for number in row {
            data.push(number.as_f64().ok_or_else(|| bad("not a number"))? as f32);
        }
    }
    if dim == 0 {
        return Err(bad("empty rows"));
    }
    Ok(TokenStates { rows, dim, data })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_embedding_server_returns_token_states_and_keeps_the_built_in_safety_settings() {
        let spec = embeddings_launch_spec(
            Path::new("/app/llama-server"),
            Path::new("/models/laya.gguf"),
            "laya",
            5556,
            "secret",
        );
        let args = spec.args.join(" ");
        for part in [
            "--model /models/laya.gguf",
            "--host 127.0.0.1",
            "--port 5556",
            "--embeddings",
            "--pooling none",
            "--ctx-size 1024",
            "--batch-size 1024",
            "--ubatch-size 1024",
            "--no-webui",
            "--offline",
        ] {
            assert!(args.contains(part), "missing {part} in {args}");
        }
        assert!(!args.contains("--reasoning"));
        assert!(!args.contains("secret"));
        assert_eq!(
            spec.envs,
            vec![("LLAMA_API_KEY".to_string(), "secret".to_string())]
        );
    }

    #[test]
    fn the_origin_drops_the_openai_path() {
        assert_eq!(origin("http://127.0.0.1:9/v1"), "http://127.0.0.1:9");
        assert_eq!(origin("http://127.0.0.1:9/v1/"), "http://127.0.0.1:9");
        assert_eq!(origin("http://127.0.0.1:9"), "http://127.0.0.1:9");
    }

    #[test]
    fn token_states_are_read_row_by_row_and_checked() {
        let value = serde_json::json!([{"index": 0, "embedding": [[1.0, 2.0], [3.0, 4.5]]}]);
        let states = parse_token_states(&value, 2).unwrap();
        assert_eq!((states.rows, states.dim), (2, 2));
        assert_eq!(states.row(1), &[3.0, 4.5]);

        // A pooled answer (one vector) is refused.
        let pooled = serde_json::json!([{"index": 0, "embedding": [[1.0, 2.0]]}]);
        assert!(matches!(
            parse_token_states(&pooled, 2),
            Err(RouterError::BadResponse(_))
        ));
        let ragged = serde_json::json!([{"index": 0, "embedding": [[1.0, 2.0], [3.0]]}]);
        assert!(parse_token_states(&ragged, 2).is_err());
        assert!(parse_token_states(&serde_json::json!({"error": "x"}), 1).is_err());
    }

    #[tokio::test]
    async fn empty_and_too_long_inputs_are_refused_before_any_request() {
        let client = EmbeddingsClient::new(Endpoint {
            base_url: "http://127.0.0.1:9/v1".into(),
            api_key: "k".into(),
        });
        assert!(matches!(
            client.token_states(&[]).await,
            Err(RouterError::Question(_))
        ));
        let long = vec![5u32; EMBEDDING_CONTEXT_TOKENS + 1];
        assert!(matches!(
            client.token_states(&long).await,
            Err(RouterError::Question(_))
        ));
    }
}
