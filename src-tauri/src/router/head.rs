//! Plan `language-router`: Laya's decision head in plain Rust, and the token sequence it reads.
//!
//! One question is one sequence (Laya's `build_sequence`):
//!
//! ```text
//! <bos> "{type} question: {instructions}" <eos> <mask> " option 0" <mask> " option 1" ... <eos> text <eos>
//! ```
//!
//! The whole sequence, question and options included, goes through the mmBERT backbone (in
//! `llama-server`, see `embeddings`). The head then works on the backbone's last hidden states
//! `h` (one 768-wide row per token):
//!
//! 1. `h += type_emb[question type]` (0 choice, 1 score, 2 yes/no) on every row;
//! 2. two PyTorch `TransformerEncoderLayer`s, pre-norm, 12 heads, ReLU feed-forward of 3072:
//!    `x += SelfAttention(LayerNorm1(x))`, then `x += Linear2(ReLU(Linear1(LayerNorm2(x))))`;
//! 3. the rows at the `<mask>` markers go through the scorer
//!    `LayerNorm -> Linear(768, 768) -> GELU (erf) -> Linear(768, 1)`, one logit per option;
//! 4. `softmax(logits / temperature)`; the multilingual checkpoint ships temperature 1.0 (it is
//!    not calibrated), and like the `laya` package a temperature is kept within 0.5 to 5.0.
//!
//! Only the marker rows of the last layer are needed, so that layer computes keys and values
//! for every row but queries, attention output and feed-forward for the markers only. Laya's
//! `act_head` (an "escalate" signal) is not used.

use std::collections::HashMap;
use std::path::Path;

use safetensors::{Dtype, SafeTensors};
use serde::Deserialize;

use super::embeddings::TokenStates;
use super::tokenizer::Tokenizer;
use super::RouterError;

/// Laya's question types, in the order of its `type_emb` rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuestionKind {
    /// Pick one of several options.
    Choice = 0,
    /// Pick a level on an ordered scale.
    Score = 1,
    /// Yes or no (Laya calls it "noul").
    YesNo = 2,
}

impl QuestionKind {
    fn name(self) -> &'static str {
        match self {
            QuestionKind::Choice => "choice",
            QuestionKind::Score => "score",
            QuestionKind::YesNo => "noul",
        }
    }
}

/// One option of a question: the key returned with its probability, and an optional
/// description the model reads as `key: description`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionOption {
    pub key: String,
    pub description: Option<String>,
}

/// A typed question, as the `laya` package takes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Question {
    pub kind: QuestionKind,
    pub instructions: String,
    pub options: Vec<QuestionOption>,
}

impl Question {
    /// A choice between `(key, description)` options (an empty description is left out).
    pub fn choice(instructions: &str, options: &[(&str, &str)]) -> Self {
        Self {
            kind: QuestionKind::Choice,
            instructions: instructions.to_string(),
            options: options
                .iter()
                .map(|(key, description)| QuestionOption {
                    key: key.to_string(),
                    description: (!description.is_empty()).then(|| description.to_string()),
                })
                .collect(),
        }
    }

    /// An ordered scale; the keys are "0", "1", ...
    pub fn score(instructions: &str, levels: &[&str]) -> Self {
        Self {
            kind: QuestionKind::Score,
            instructions: instructions.to_string(),
            options: levels
                .iter()
                .enumerate()
                .map(|(index, level)| QuestionOption {
                    key: index.to_string(),
                    description: Some(level.to_string()),
                })
                .collect(),
        }
    }

    /// A yes/no statement; the keys are "false" and "true", with Laya's default descriptions.
    pub fn yes_no(instructions: &str) -> Self {
        Self {
            kind: QuestionKind::YesNo,
            instructions: instructions.to_string(),
            options: vec![
                QuestionOption {
                    key: "false".into(),
                    description: None,
                },
                QuestionOption {
                    key: "true".into(),
                    description: None,
                },
            ],
        }
    }

    /// The option texts the model reads (Laya's `render_options`).
    pub fn rendered_options(&self) -> Vec<String> {
        let described = |option: &QuestionOption| match option.description.as_deref() {
            Some(description) if !description.is_empty() => Some(description.to_string()),
            _ => None,
        };
        self.options
            .iter()
            .enumerate()
            .map(|(index, option)| match self.kind {
                QuestionKind::Choice => match described(option) {
                    Some(description) => format!("{}: {description}", option.key),
                    None => option.key.clone(),
                },
                QuestionKind::Score => {
                    format!("level {index}: {}", described(option).unwrap_or_default())
                }
                QuestionKind::YesNo => {
                    let default = if index == 0 {
                        "no, the statement does not hold"
                    } else {
                        "yes, the statement holds"
                    };
                    format!(
                        "{}: {}",
                        option.key,
                        described(option).unwrap_or_else(|| default.to_string())
                    )
                }
            })
            .collect()
    }
}

/// Laya's settings, stored as JSON in the head file's `laya.config` metadata.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct HeadConfig {
    #[serde(default = "default_max_len")]
    pub max_len: usize,
    #[serde(default = "default_head_max_len")]
    pub head_max_len: usize,
    #[serde(default = "default_temperature")]
    pub temperature: [f32; 3],
    #[serde(default)]
    pub temperature_by_options: HashMap<String, f32>,
}

fn default_max_len() -> usize {
    512
}
fn default_head_max_len() -> usize {
    192
}
fn default_temperature() -> [f32; 3] {
    [1.0; 3]
}

impl Default for HeadConfig {
    fn default() -> Self {
        Self {
            max_len: default_max_len(),
            head_max_len: default_head_max_len(),
            temperature: default_temperature(),
            temperature_by_options: HashMap::new(),
        }
    }
}

impl HeadConfig {
    /// The temperature for `kind` with `options` options (Laya's `temp_bucket`), kept within
    /// 0.5 to 5.0 like the `laya` package does.
    pub fn temperature_for(&self, kind: QuestionKind, options: usize) -> f32 {
        let size = match options {
            0..=2 => "2",
            3..=5 => "3-5",
            6..=10 => "6-10",
            _ => "11+",
        };
        let key = format!("{}:{size}", kind.name());
        let t = self
            .temperature_by_options
            .get(&key)
            .copied()
            .unwrap_or(self.temperature[kind as usize]);
        if t.is_finite() {
            t.clamp(0.5, 5.0)
        } else {
            1.0
        }
    }
}

/// The special tokens Laya's sequence uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SpecialTokens {
    /// `<bos>` (Laya's `cls_token`).
    pub cls: u32,
    /// `<eos>` (Laya's `sep_token`).
    pub sep: u32,
    pub mask: u32,
}

impl SpecialTokens {
    pub fn from_tokenizer(tokenizer: &Tokenizer) -> Result<Self, RouterError> {
        let id = |token: &str| {
            tokenizer
                .token_id(token)
                .ok_or_else(|| RouterError::Model(format!("tokenizer has no {token}")))
        };
        Ok(Self {
            cls: id("<bos>")?,
            sep: id("<eos>")?,
            mask: id("<mask>")?,
        })
    }
}

const MASK_TEXT: &str = "<mask>";
/// Laya cuts every option to this many tokens.
const OPTION_TOKENS: usize = 48;

/// One question as model input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sequence {
    pub ids: Vec<u32>,
    /// Where each option's `<mask>` marker is, in option order.
    pub markers: Vec<usize>,
    pub kind: QuestionKind,
}

/// Laya's `build_sequence` for a plain-text state (right-truncated to fit `max_len`).
pub fn build_sequence(
    tokenizer: &Tokenizer,
    special: SpecialTokens,
    text: &str,
    question: &Question,
    max_len: usize,
    head_max_len: usize,
) -> Result<Sequence, RouterError> {
    let options = question.rendered_options();
    if options.is_empty() {
        return Err(RouterError::Question("no options".into()));
    }
    let instructions = question.instructions.replace(MASK_TEXT, " ");
    let mut head_ids = tokenizer.encode(&format!(
        "{} question: {instructions}",
        question.kind.name()
    ));
    let mut option_ids: Vec<Vec<u32>> = options
        .iter()
        .map(|option| {
            let mut ids = vec![special.mask];
            let tokens = tokenizer.encode(&format!(" {}", option.replace(MASK_TEXT, " ")));
            ids.extend(tokens.into_iter().take(OPTION_TOKENS));
            ids
        })
        .collect();
    let used: usize = option_ids.iter().map(Vec::len).sum();
    let mut budget = head_max_len as i64 - used as i64;
    if budget < 16 {
        // Too many or too long options: cut every option to the same length.
        let per = ((head_max_len as i64 - 16).div_euclid(option_ids.len() as i64)).max(4) as usize;
        for ids in &mut option_ids {
            ids.truncate(per);
        }
        let used: usize = option_ids.iter().map(Vec::len).sum();
        budget = head_max_len as i64 - used as i64;
    }
    head_ids.truncate(budget.max(8) as usize);

    let mut ids = Vec::with_capacity(max_len);
    ids.push(special.cls);
    ids.extend(head_ids);
    ids.push(special.sep);
    let mut markers = Vec::with_capacity(option_ids.len());
    for option in option_ids {
        markers.push(ids.len());
        ids.extend(option);
    }
    ids.push(special.sep);
    let room = max_len.saturating_sub(ids.len() + 1);
    let state = tokenizer.encode(&text.replace(MASK_TEXT, " "));
    ids.extend(state.into_iter().take(room));
    ids.push(special.sep);
    ids.truncate(max_len);
    markers.retain(|&marker| marker < max_len);
    if markers.len() != options.len() {
        return Err(RouterError::Question(format!(
            "the options do not fit in {head_max_len} tokens"
        )));
    }
    Ok(Sequence {
        ids,
        markers,
        kind: question.kind,
    })
}

struct EncoderLayer {
    in_proj_weight: Vec<f32>,
    in_proj_bias: Vec<f32>,
    out_proj_weight: Vec<f32>,
    out_proj_bias: Vec<f32>,
    norm1_weight: Vec<f32>,
    norm1_bias: Vec<f32>,
    norm2_weight: Vec<f32>,
    norm2_bias: Vec<f32>,
    linear1_weight: Vec<f32>,
    linear1_bias: Vec<f32>,
    linear2_weight: Vec<f32>,
    linear2_bias: Vec<f32>,
}

/// The loaded head (about 60 MB of f32 weights).
pub struct LayaHead {
    dim: usize,
    heads: usize,
    ffn: usize,
    layers: Vec<EncoderLayer>,
    type_emb: Vec<f32>,
    scorer_norm_weight: Vec<f32>,
    scorer_norm_bias: Vec<f32>,
    scorer_hidden_weight: Vec<f32>,
    scorer_hidden_bias: Vec<f32>,
    scorer_out_weight: Vec<f32>,
    scorer_out_bias: f32,
    pub config: HeadConfig,
}

const LAYER_NORM_EPS: f32 = 1e-5;

impl LayaHead {
    pub fn from_file(path: &Path) -> Result<Self, RouterError> {
        let bytes = std::fs::read(path)
            .map_err(|error| RouterError::Model(format!("{}: {error}", path.display())))?;
        Self::from_safetensors(&bytes, None)
    }

    /// Reads the head from a safetensors file with the original PyTorch names
    /// (`head.layers.N.*`, `type_emb.weight`, `scorer.*`). `heads` is the attention head count;
    /// `None` means Laya's rule, width / 64.
    pub fn from_safetensors(bytes: &[u8], heads: Option<usize>) -> Result<Self, RouterError> {
        let model_error =
            |error: safetensors::SafeTensorError| RouterError::Model(error.to_string());
        let (_, metadata) = SafeTensors::read_metadata(bytes).map_err(model_error)?;
        let config = match metadata
            .metadata()
            .as_ref()
            .and_then(|map| map.get("laya.config"))
        {
            Some(json) => serde_json::from_str(json)
                .map_err(|error| RouterError::Model(format!("laya.config: {error}")))?,
            None => HeadConfig::default(),
        };
        let tensors = SafeTensors::deserialize(bytes).map_err(model_error)?;
        let get = |name: &str, shape: &[usize]| -> Result<Vec<f32>, RouterError> {
            let view = tensors
                .tensor(name)
                .map_err(|_| RouterError::Model(format!("head has no {name}")))?;
            if view.dtype() != Dtype::F32 || view.shape() != shape {
                return Err(RouterError::Model(format!(
                    "{name}: {:?} {:?}, expected F32 {shape:?}",
                    view.dtype(),
                    view.shape()
                )));
            }
            Ok(view
                .data()
                .chunks_exact(4)
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect())
        };

        let type_shape = tensors
            .tensor("type_emb.weight")
            .map_err(|_| RouterError::Model("head has no type_emb.weight".into()))?
            .shape()
            .to_vec();
        let [3, dim] = type_shape[..] else {
            return Err(RouterError::Model(format!("type_emb is {type_shape:?}")));
        };
        let heads = heads.unwrap_or((dim / 64).max(1));
        if dim % heads != 0 {
            return Err(RouterError::Model(format!(
                "{dim} is not divisible by {heads} heads"
            )));
        }
        let ffn = tensors
            .tensor("head.layers.0.linear1.weight")
            .map(|view| view.shape().first().copied().unwrap_or(0))
            .unwrap_or(4 * dim);
        let mut layers = Vec::new();
        while tensors
            .tensor(&format!("head.layers.{}.norm1.weight", layers.len()))
            .is_ok()
        {
            let p = format!("head.layers.{}.", layers.len());
            layers.push(EncoderLayer {
                in_proj_weight: get(&format!("{p}self_attn.in_proj_weight"), &[3 * dim, dim])?,
                in_proj_bias: get(&format!("{p}self_attn.in_proj_bias"), &[3 * dim])?,
                out_proj_weight: get(&format!("{p}self_attn.out_proj.weight"), &[dim, dim])?,
                out_proj_bias: get(&format!("{p}self_attn.out_proj.bias"), &[dim])?,
                norm1_weight: get(&format!("{p}norm1.weight"), &[dim])?,
                norm1_bias: get(&format!("{p}norm1.bias"), &[dim])?,
                norm2_weight: get(&format!("{p}norm2.weight"), &[dim])?,
                norm2_bias: get(&format!("{p}norm2.bias"), &[dim])?,
                linear1_weight: get(&format!("{p}linear1.weight"), &[ffn, dim])?,
                linear1_bias: get(&format!("{p}linear1.bias"), &[ffn])?,
                linear2_weight: get(&format!("{p}linear2.weight"), &[dim, ffn])?,
                linear2_bias: get(&format!("{p}linear2.bias"), &[dim])?,
            });
        }
        if layers.is_empty() {
            return Err(RouterError::Model("head has no layers".into()));
        }
        Ok(Self {
            dim,
            heads,
            ffn,
            layers,
            type_emb: get("type_emb.weight", &[3, dim])?,
            scorer_norm_weight: get("scorer.0.weight", &[dim])?,
            scorer_norm_bias: get("scorer.0.bias", &[dim])?,
            scorer_hidden_weight: get("scorer.1.weight", &[dim, dim])?,
            scorer_hidden_bias: get("scorer.1.bias", &[dim])?,
            scorer_out_weight: get("scorer.3.weight", &[1, dim])?,
            scorer_out_bias: get("scorer.3.bias", &[1])?[0],
            config,
        })
    }

    pub fn dim(&self) -> usize {
        self.dim
    }

    /// One logit per option marker, from the backbone's hidden states for the whole sequence.
    pub fn logits(
        &self,
        states: &TokenStates,
        markers: &[usize],
        kind: QuestionKind,
    ) -> Result<Vec<f32>, RouterError> {
        let d = self.dim;
        if states.dim != d || states.rows == 0 || states.data.len() != states.rows * d {
            return Err(RouterError::BadResponse(format!(
                "hidden states are {} x {}, the head needs width {d}",
                states.rows, states.dim
            )));
        }
        if markers.is_empty() || markers.iter().any(|&m| m >= states.rows) {
            return Err(RouterError::Question(
                "option marker outside the sequence".into(),
            ));
        }
        let type_row = &self.type_emb[kind as usize * d..(kind as usize + 1) * d];
        let mut x = states.data.clone();
        for row in x.chunks_exact_mut(d) {
            add_in_place(row, type_row);
        }
        let mut rows = states.rows;
        let last = self.layers.len() - 1;
        for (index, layer) in self.layers.iter().enumerate() {
            let queries: Vec<usize> = if index == last {
                markers.to_vec()
            } else {
                (0..rows).collect()
            };
            x = self.encoder_layer(layer, &x, rows, &queries);
            rows = queries.len();
        }
        // After the last layer, `x` holds the marker rows in marker order.
        let normed = layer_norm(&x, d, &self.scorer_norm_weight, &self.scorer_norm_bias);
        let mut hidden = linear(
            &normed,
            rows,
            d,
            &self.scorer_hidden_weight,
            d,
            &self.scorer_hidden_bias,
        );
        for value in &mut hidden {
            *value = gelu(*value);
        }
        Ok(hidden
            .chunks_exact(d)
            .map(|row| dot(row, &self.scorer_out_weight) + self.scorer_out_bias)
            .collect())
    }

    /// One pre-norm encoder layer. Returns the new rows for `queries` only (all rows except
    /// in the last layer).
    fn encoder_layer(
        &self,
        layer: &EncoderLayer,
        x: &[f32],
        rows: usize,
        queries: &[usize],
    ) -> Vec<f32> {
        let d = self.dim;
        let heads = self.heads;
        let head_dim = d / heads;
        let normed = layer_norm(x, d, &layer.norm1_weight, &layer.norm1_bias);
        // Keys and values for every row; queries only where needed.
        let kv = linear(
            &normed,
            rows,
            d,
            &layer.in_proj_weight[d * d..],
            2 * d,
            &layer.in_proj_bias[d..],
        );
        let query_input: Vec<f32> = queries
            .iter()
            .flat_map(|&row| normed[row * d..(row + 1) * d].iter().copied())
            .collect();
        let q = linear(
            &query_input,
            queries.len(),
            d,
            &layer.in_proj_weight[..d * d],
            d,
            &layer.in_proj_bias[..d],
        );
        let scale = 1.0 / (head_dim as f32).sqrt();
        let mut attended = vec![0.0f32; queries.len() * d];
        let mut scores = vec![0.0f32; rows];
        for qi in 0..queries.len() {
            for h in 0..heads {
                let range = h * head_dim..(h + 1) * head_dim;
                let query = &q[qi * d..(qi + 1) * d][range.clone()];
                let mut max = f32::NEG_INFINITY;
                for (j, score) in scores.iter_mut().enumerate() {
                    let key = &kv[j * 2 * d..j * 2 * d + d][range.clone()];
                    *score = dot(query, key) * scale;
                    max = max.max(*score);
                }
                let mut sum = 0.0;
                for score in scores.iter_mut() {
                    *score = (*score - max).exp();
                    sum += *score;
                }
                let out = &mut attended[qi * d..(qi + 1) * d][range.clone()];
                for (j, score) in scores.iter().enumerate() {
                    let value = &kv[j * 2 * d + d..(j + 1) * 2 * d][range.clone()];
                    let weight = score / sum;
                    for (o, v) in out.iter_mut().zip(value) {
                        *o += weight * v;
                    }
                }
            }
        }
        let projected = linear(
            &attended,
            queries.len(),
            d,
            &layer.out_proj_weight,
            d,
            &layer.out_proj_bias,
        );
        let mut y: Vec<f32> = queries
            .iter()
            .flat_map(|&row| x[row * d..(row + 1) * d].iter().copied())
            .collect();
        add_in_place(&mut y, &projected);
        let normed = layer_norm(&y, d, &layer.norm2_weight, &layer.norm2_bias);
        let mut hidden = linear(
            &normed,
            queries.len(),
            d,
            &layer.linear1_weight,
            self.ffn,
            &layer.linear1_bias,
        );
        for value in &mut hidden {
            *value = value.max(0.0);
        }
        let ff = linear(
            &hidden,
            queries.len(),
            self.ffn,
            &layer.linear2_weight,
            d,
            &layer.linear2_bias,
        );
        add_in_place(&mut y, &ff);
        y
    }
}

/// `softmax(logits / temperature)`.
pub fn probabilities(logits: &[f32], temperature: f32) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps: Vec<f32> = logits
        .iter()
        .map(|&z| ((z - max) / temperature).exp())
        .collect();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|e| e / sum).collect()
}

fn add_in_place(target: &mut [f32], other: &[f32]) {
    for (t, o) in target.iter_mut().zip(other) {
        *t += o;
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    // Eight independent sums let the compiler use SIMD.
    let mut sums = [0.0f32; 8];
    let mut a_chunks = a.chunks_exact(8);
    let mut b_chunks = b.chunks_exact(8);
    for (x, y) in (&mut a_chunks).zip(&mut b_chunks) {
        for i in 0..8 {
            sums[i] += x[i] * y[i];
        }
    }
    let tail: f32 = a_chunks
        .remainder()
        .iter()
        .zip(b_chunks.remainder())
        .map(|(x, y)| x * y)
        .sum();
    sums.iter().sum::<f32>() + tail
}

fn layer_norm(x: &[f32], d: usize, weight: &[f32], bias: &[f32]) -> Vec<f32> {
    let mut out = Vec::with_capacity(x.len());
    for row in x.chunks_exact(d) {
        let mean = row.iter().sum::<f32>() / d as f32;
        let var = row.iter().map(|v| (v - mean) * (v - mean)).sum::<f32>() / d as f32;
        let inv = 1.0 / (var + LAYER_NORM_EPS).sqrt();
        out.extend(
            row.iter()
                .zip(weight.iter().zip(bias))
                .map(|(v, (w, b))| (v - mean) * inv * w + b),
        );
    }
    out
}

/// `x (rows x in) @ weight (out x in)^T + bias`, the layout of PyTorch's `nn.Linear`.
///
/// On macOS this is one `cblas_sgemm` call into Apple's Accelerate framework (part of the
/// system, no extra library), about ten times faster than the portable loops, which other
/// systems use and the tests compare against.
fn linear(
    x: &[f32],
    rows: usize,
    input: usize,
    weight: &[f32],
    output: usize,
    bias: &[f32],
) -> Vec<f32> {
    #[cfg(target_os = "macos")]
    {
        accelerate::linear(x, rows, input, weight, output, bias)
    }
    #[cfg(not(target_os = "macos"))]
    {
        linear_portable(x, rows, input, weight, output, bias)
    }
}

#[cfg(target_os = "macos")]
mod accelerate {
    const ROW_MAJOR: i32 = 101;
    const NO_TRANS: i32 = 111;
    const TRANS: i32 = 112;

    #[link(name = "Accelerate", kind = "framework")]
    extern "C" {
        /// `C = alpha * op(A) * op(B) + beta * C` (the standard CBLAS interface).
        fn cblas_sgemm(
            order: i32,
            trans_a: i32,
            trans_b: i32,
            m: i32,
            n: i32,
            k: i32,
            alpha: f32,
            a: *const f32,
            lda: i32,
            b: *const f32,
            ldb: i32,
            beta: f32,
            c: *mut f32,
            ldc: i32,
        );
    }

    pub fn linear(
        x: &[f32],
        rows: usize,
        input: usize,
        weight: &[f32],
        output: usize,
        bias: &[f32],
    ) -> Vec<f32> {
        assert!(x.len() >= rows * input && weight.len() >= output * input);
        assert!(bias.len() >= output);
        // Start from the bias in every row and add x @ weight^T onto it (beta = 1).
        let mut out: Vec<f32> = (0..rows)
            .flat_map(|_| bias[..output].iter().copied())
            .collect();
        if rows == 0 || output == 0 {
            return out;
        }
        // SAFETY: the slices are at least as long as the sizes passed (checked above), the
        // matrices are row-major with the given leading dimensions, and `out` is not aliased.
        unsafe {
            cblas_sgemm(
                ROW_MAJOR,
                NO_TRANS,
                TRANS,
                rows as i32,
                output as i32,
                input as i32,
                1.0,
                x.as_ptr(),
                input as i32,
                weight.as_ptr(),
                input as i32,
                1.0,
                out.as_mut_ptr(),
                output as i32,
            );
        }
        out
    }
}

/// The portable version of [`linear`]: plain loops, large products split over threads by
/// output column.
#[cfg_attr(target_os = "macos", allow(dead_code))] // Used off macOS; on macOS by the tests.
fn linear_portable(
    x: &[f32],
    rows: usize,
    input: usize,
    weight: &[f32],
    output: usize,
    bias: &[f32],
) -> Vec<f32> {
    let compute = |columns: std::ops::Range<usize>| -> Vec<f32> {
        let width = columns.len();
        let mut part = vec![0.0f32; rows * width];
        for (c, column) in columns.enumerate() {
            let w = &weight[column * input..(column + 1) * input];
            for r in 0..rows {
                part[r * width + c] = dot(&x[r * input..(r + 1) * input], w) + bias[column];
            }
        }
        part
    };
    let threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
        .min(8);
    if threads < 2 || rows * input * output < 2_000_000 {
        return compute(0..output);
    }
    let chunk = output.div_ceil(threads);
    let parts: Vec<(usize, Vec<f32>)> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..output)
            .step_by(chunk)
            .map(|start| {
                let end = (start + chunk).min(output);
                (start, scope.spawn(move || compute(start..end)))
            })
            .collect();
        handles
            .into_iter()
            .map(|(start, handle)| (start, handle.join().unwrap_or_default()))
            .collect()
    });
    let mut out = vec![0.0f32; rows * output];
    for (start, part) in parts {
        let width = part.len() / rows.max(1);
        for r in 0..rows {
            out[r * output + start..r * output + start + width]
                .copy_from_slice(&part[r * width..(r + 1) * width]);
        }
    }
    out
}

/// GELU with the exact error function, as PyTorch's `nn.GELU()`.
fn gelu(x: f32) -> f32 {
    0.5 * x * (1.0 + erf(x / std::f32::consts::SQRT_2))
}

/// Error function (Abramowitz and Stegun 7.1.26, absolute error below 1.5e-7), in f64.
fn erf(x: f32) -> f32 {
    let x = x as f64;
    let sign = if x < 0.0 { -1.0 } else { 1.0 };
    let x = x.abs();
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let poly = t
        * (0.254_829_592
            + t * (-0.284_496_736
                + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    (sign * (1.0 - poly * (-x * x).exp())) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Deserialize)]
    struct TinyTensor {
        shape: Vec<usize>,
        data: Vec<f32>,
    }

    #[derive(Deserialize)]
    struct TinyHead {
        d: usize,
        heads: usize,
        rows: usize,
        qtype: usize,
        markers: Vec<usize>,
        hidden: Vec<f32>,
        logits: Vec<f32>,
        weights: HashMap<String, TinyTensor>,
    }

    /// The tiny head's weights written as a safetensors file, so loading is tested too.
    fn tiny_safetensors(fixture: &TinyHead) -> Vec<u8> {
        let bytes: Vec<(String, Vec<usize>, Vec<u8>)> = fixture
            .weights
            .iter()
            .map(|(name, tensor)| {
                let data = tensor.data.iter().flat_map(|v| v.to_le_bytes()).collect();
                (name.clone(), tensor.shape.clone(), data)
            })
            .collect();
        let views: Vec<(String, safetensors::tensor::TensorView)> = bytes
            .iter()
            .map(|(name, shape, data)| {
                (
                    name.clone(),
                    safetensors::tensor::TensorView::new(Dtype::F32, shape.clone(), data).unwrap(),
                )
            })
            .collect();
        let metadata = HashMap::from([(
            "laya.config".to_string(),
            r#"{"max_len": 64, "head_max_len": 32, "temperature": [1.0, 2.0, 0.1]}"#.to_string(),
        )]);
        safetensors::serialize(views, Some(metadata)).unwrap()
    }

    fn tiny() -> TinyHead {
        serde_json::from_str(include_str!("../../tests/fixtures/laya/tiny_head.json")).unwrap()
    }

    /// Weights, input and logits come from PyTorch's own modules (see
    /// `tests/fixtures/laya/generate_fixtures.py`).
    #[test]
    fn the_head_matches_pytorch_on_a_tiny_model() {
        let fixture = tiny();
        let head =
            LayaHead::from_safetensors(&tiny_safetensors(&fixture), Some(fixture.heads)).unwrap();
        assert_eq!(head.dim(), fixture.d);
        assert_eq!(head.layers.len(), 2);
        let states = TokenStates {
            rows: fixture.rows,
            dim: fixture.d,
            data: fixture.hidden.clone(),
        };
        let kind = [
            QuestionKind::Choice,
            QuestionKind::Score,
            QuestionKind::YesNo,
        ][fixture.qtype];
        let logits = head.logits(&states, &fixture.markers, kind).unwrap();
        assert_eq!(logits.len(), fixture.logits.len());
        for (got, want) in logits.iter().zip(&fixture.logits) {
            assert!(
                (got - want).abs() < 1e-4,
                "{logits:?} vs {:?}",
                fixture.logits
            );
        }
        // The marker-only last layer gives the same rows as a full one: reorder the markers.
        let reversed: Vec<usize> = fixture.markers.iter().rev().copied().collect();
        let back = head.logits(&states, &reversed, kind).unwrap();
        assert!((back[0] - logits[logits.len() - 1]).abs() < 1e-6);
    }

    #[test]
    fn the_config_and_temperatures_come_from_the_file_metadata() {
        let fixture = tiny();
        let head =
            LayaHead::from_safetensors(&tiny_safetensors(&fixture), Some(fixture.heads)).unwrap();
        assert_eq!(head.config.max_len, 64);
        assert_eq!(head.config.head_max_len, 32);
        assert_eq!(head.config.temperature_for(QuestionKind::Score, 3), 2.0);
        // Sharpening below 0.5 is refused, like the `laya` package does.
        assert_eq!(head.config.temperature_for(QuestionKind::YesNo, 2), 0.5);
        let mut config = HeadConfig::default();
        config
            .temperature_by_options
            .insert("choice:3-5".into(), 1.5);
        assert_eq!(config.temperature_for(QuestionKind::Choice, 4), 1.5);
        assert_eq!(config.temperature_for(QuestionKind::Choice, 2), 1.0);
    }

    #[test]
    fn bad_inputs_are_errors_not_panics() {
        let fixture = tiny();
        let head =
            LayaHead::from_safetensors(&tiny_safetensors(&fixture), Some(fixture.heads)).unwrap();
        let narrow = TokenStates {
            rows: 2,
            dim: 3,
            data: vec![0.0; 6],
        };
        assert!(head.logits(&narrow, &[0], QuestionKind::Choice).is_err());
        let states = TokenStates {
            rows: fixture.rows,
            dim: fixture.d,
            data: fixture.hidden.clone(),
        };
        assert!(head
            .logits(&states, &[fixture.rows], QuestionKind::Choice)
            .is_err());
        assert!(head.logits(&states, &[], QuestionKind::Choice).is_err());
        assert!(LayaHead::from_safetensors(b"not a file", None).is_err());
    }

    #[test]
    fn the_fast_and_portable_matrix_products_agree() {
        let (rows, input, output) = (5, 70, 300);
        let x: Vec<f32> = (0..rows * input)
            .map(|i| ((i * 37 % 101) as f32 - 50.0) / 25.0)
            .collect();
        let w: Vec<f32> = (0..output * input)
            .map(|i| ((i * 53 % 97) as f32 - 48.0) / 40.0)
            .collect();
        let b: Vec<f32> = (0..output).map(|i| i as f32 / 100.0).collect();
        let fast = linear(&x, rows, input, &w, output, &b);
        let portable = linear_portable(&x, rows, input, &w, output, &b);
        // Also over the thread threshold.
        let big_rows = 100;
        let big_x: Vec<f32> = (0..big_rows * input)
            .map(|i| ((i % 13) as f32 - 6.0) / 7.0)
            .collect();
        let big_fast = linear(&big_x, big_rows, input, &w, output, &b);
        let big_portable = linear_portable(&big_x, big_rows, input, &w, output, &b);
        for (got, want) in fast
            .iter()
            .zip(&portable)
            .chain(big_fast.iter().zip(&big_portable))
        {
            assert!(
                (got - want).abs() < 1e-3 * want.abs().max(1.0),
                "{got} vs {want}"
            );
        }
        // One hand-checked value: row 0, column 0.
        let manual: f32 = (0..input).map(|k| x[k] * w[k]).sum::<f32>() + b[0];
        assert!((portable[0] - manual).abs() < 1e-4);
    }

    #[test]
    fn probabilities_are_a_tempered_softmax() {
        let p = probabilities(&[1.0, 2.0, 3.0], 1.0);
        assert!((p.iter().sum::<f32>() - 1.0).abs() < 1e-6);
        assert!((p[2] - 0.665_240_9).abs() < 1e-5);
        let flat = probabilities(&[1.0, 2.0, 3.0], 5.0);
        assert!(flat[2] < p[2]);
        let big = probabilities(&[1000.0, 0.0], 1.0);
        assert!((big[0] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn gelu_uses_the_exact_error_function() {
        // Values from torch.nn.functional.gelu.
        for (x, want) in [
            (-3.0, -0.004_049_689),
            (-0.5, -0.154_268_77),
            (0.0, 0.0),
            (1.0, 0.841_344_7),
            (2.5, 2.484_475_9),
        ] {
            assert!((gelu(x) - want).abs() < 1e-6, "{x}: {}", gelu(x));
        }
    }

    #[test]
    fn options_are_rendered_like_laya() {
        let choice = Question::choice("Pick", &[("billing", "invoices"), ("sales", "")]);
        assert_eq!(choice.rendered_options(), ["billing: invoices", "sales"]);
        let score = Question::score("How urgent?", &["low", "high"]);
        assert_eq!(score.rendered_options(), ["level 0: low", "level 1: high"]);
        let yes_no = Question::yes_no("Is it a question?");
        assert_eq!(
            yes_no.rendered_options(),
            [
                "false: no, the statement does not hold",
                "true: yes, the statement holds"
            ]
        );
    }

    fn tiny_tokenizer() -> (Tokenizer, SpecialTokens) {
        let tokenizer = Tokenizer::from_json(include_bytes!(
            "../../tests/fixtures/laya/tiny_tokenizer.json"
        ))
        .unwrap();
        let special = SpecialTokens::from_tokenizer(&tokenizer).unwrap();
        (tokenizer, special)
    }

    #[test]
    fn the_sequence_has_the_question_then_marked_options_then_the_text() {
        let (tokenizer, special) = tiny_tokenizer();
        let question = Question::choice("which one?", &[("yes", ""), ("no", "")]);
        let seq = build_sequence(&tokenizer, special, "hello world", &question, 1024, 256).unwrap();
        let head = tokenizer.encode("choice question: which one?");
        assert_eq!(seq.ids[0], special.cls);
        assert_eq!(&seq.ids[1..1 + head.len()], &head[..]);
        assert_eq!(seq.ids[1 + head.len()], special.sep);
        assert_eq!(seq.markers[0], head.len() + 2);
        for &marker in &seq.markers {
            assert_eq!(seq.ids[marker], special.mask);
        }
        let yes = tokenizer.encode(" yes");
        assert_eq!(&seq.ids[seq.markers[0] + 1..seq.markers[1]], &yes[..]);
        let text = tokenizer.encode("hello world");
        let end = seq.ids.len();
        assert_eq!(&seq.ids[end - 1 - text.len()..end - 1], &text[..]);
        assert_eq!(seq.ids[end - 1], special.sep);
        assert_eq!(seq.ids[end - 2 - text.len()], special.sep);
        assert_eq!(seq.kind, QuestionKind::Choice);
    }

    #[test]
    fn a_long_text_is_cut_to_the_limit_and_mask_text_is_blanked() {
        let (tokenizer, special) = tiny_tokenizer();
        let question = Question::yes_no("hello <mask> world");
        let long = "hello world ".repeat(200);
        let seq = build_sequence(&tokenizer, special, &long, &question, 64, 32).unwrap();
        assert_eq!(seq.ids.len(), 64);
        assert_eq!(*seq.ids.last().unwrap(), special.sep);
        // Only the two option markers are masks.
        assert_eq!(seq.ids.iter().filter(|&&id| id == special.mask).count(), 2);

        let many: Vec<(String, String)> = (0..40)
            .map(|i| (format!("option {i}"), "a long description".into()))
            .collect();
        let refs: Vec<(&str, &str)> = many.iter().map(|(k, d)| (k.as_str(), d.as_str())).collect();
        let crowded = Question::choice("pick", &refs);
        assert!(matches!(
            build_sequence(&tokenizer, special, "hi", &crowded, 64, 32),
            Err(RouterError::Question(_))
        ));
    }
}
