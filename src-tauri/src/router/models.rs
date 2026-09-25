//! Plan `language-router`: the three files Laya needs, for the shared download, resume and
//! SHA-256 check (`stt::models::download_model`). They go in the same `models` folder as the
//! speech and Built-in AI models.
//!
//! - **Backbone**: Laya-multilingual's mmBERT encoder as GGUF, from a community conversion
//!   (Convai Innovations publishes no GGUF). Q8_0, 341 MB: next to the F16 original its
//!   hidden states keep a median cosine of 0.9998 and move Laya's probabilities by at most
//!   0.03 on our fixtures; Q4_K_M (249 MB) moved them by up to 0.17. F16 (629 MB) is exact
//!   but twice the size for no routing gain.
//! - **Head**: the decision-head tensors and Laya's config, split out of the original
//!   checkpoint by the same conversion (checked tensor by tensor against the original).
//! - **Tokenizer**: the original `tokenizer.json` from Convai Innovations' repository.
//!
//! Every URL is pinned to one commit. Sizes and SHA-256 values are the `x-linked-size` and
//! `x-linked-etag` headers of the Hugging Face files.

use std::path::Path;

use crate::stt::models::{self, KnownModel};

/// One Laya file and where it comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayaFile {
    pub file: KnownModel,
    pub url: &'static str,
}

pub const BACKBONE: LayaFile = LayaFile {
    file: KnownModel {
        id: "laya-multilingual-q8_0",
        file_name: "laya-multilingual-Q8_0.gguf",
        size_bytes: 341_095_424,
        sha256: "c7c1e2e8be46c98e8b1b819fb73e1de23b895fca45e0fe554872bb84a08bf2ff",
    },
    url: "https://huggingface.co/fr0stbit3/laya-multilingual-gguf/resolve/65c0eefc17dc3166b39078e8abfc48cbc869d552/laya-multilingual-Q8_0.gguf",
};

pub const HEAD: LayaFile = LayaFile {
    file: KnownModel {
        id: "laya-multilingual-head",
        file_name: "laya-multilingual-head.safetensors",
        size_bytes: 59_881_360,
        sha256: "f7db19f131423a16253ff85b0664050788951510f561d3a4dbd7ffbf6784fb40",
    },
    url: "https://huggingface.co/fr0stbit3/laya-multilingual-gguf/resolve/65c0eefc17dc3166b39078e8abfc48cbc869d552/laya-multilingual-head.safetensors",
};

/// Saved under its own name so it cannot clash with another model's `tokenizer.json`.
pub const TOKENIZER: LayaFile = LayaFile {
    file: KnownModel {
        id: "laya-multilingual-tokenizer",
        file_name: "laya-multilingual-tokenizer.json",
        size_bytes: 34_363_188,
        sha256: "609d8f4c067cd3950f88594c5a802616cea245823836ef5848ee4fc40aab5b6f",
    },
    url: "https://huggingface.co/convaiinnovations/laya-multilingual/resolve/e4e9ddf21a7b1903b7acffd8814ad4307bf63a67/tokenizer/tokenizer.json",
};

/// All of them; Laya works only when every one is in place.
pub const FILES: [LayaFile; 3] = [BACKBONE, HEAD, TOKENIZER];

/// The file entries, for the shared model-file helpers.
pub fn files() -> Vec<KnownModel> {
    FILES.iter().map(|file| file.file).collect()
}

/// Bytes to download for the whole set.
pub fn total_bytes() -> u64 {
    FILES.iter().map(|file| file.file.size_bytes).sum()
}

/// True when all three files are fully in `dir`.
pub fn installed(dir: &Path) -> bool {
    models::installed_from(dir, &files()).len() == FILES.len()
}

/// The folder of the Laya files for tests: the folder of `TYPELITE_E2E_LAYA_MODEL` (the
/// backbone GGUF), where the head and tokenizer must sit under the names above.
#[cfg(test)]
pub(crate) fn test_files_dir() -> Option<std::path::PathBuf> {
    let model = std::path::PathBuf::from(std::env::var("TYPELITE_E2E_LAYA_MODEL").ok()?);
    model.parent().map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_table_is_well_formed() {
        for laya in FILES {
            let file = laya.file;
            assert_eq!(file.sha256.len(), 64, "{}", file.id);
            assert!(file
                .sha256
                .chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
            assert!(laya.url.starts_with("https://huggingface.co/"));
            // Pinned to a commit, not a branch.
            let revision = laya
                .url
                .split("/resolve/")
                .nth(1)
                .unwrap()
                .split('/')
                .next()
                .unwrap();
            assert_eq!(revision.len(), 40, "{}", laya.url);
            assert!(file.file_name.starts_with("laya-multilingual"));
            // Never the name of a speech or Built-in AI model file.
            assert!(models::known_model_by_file(file.file_name).is_none());
            assert!(!crate::llm::models::files()
                .iter()
                .any(|other| other.file_name == file.file_name || other.id == file.id));
        }
        assert!(BACKBONE.url.ends_with(BACKBONE.file.file_name));
        assert!(HEAD.url.ends_with(HEAD.file.file_name));
        assert!(TOKENIZER.url.ends_with("/tokenizer.json"));
        assert_eq!(total_bytes(), 435_339_972);
    }

    #[test]
    fn installed_needs_all_three_full_files() {
        let dir = std::env::temp_dir().join(format!(
            "typelite-laya-models-{}-{}",
            std::process::id(),
            crate::storage::now_unix_ms()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        for laya in [BACKBONE, HEAD] {
            std::fs::File::create(dir.join(laya.file.file_name))
                .unwrap()
                .set_len(laya.file.size_bytes)
                .unwrap();
        }
        assert!(!installed(&dir));
        std::fs::write(dir.join(TOKENIZER.file.file_name), b"short").unwrap();
        assert!(!installed(&dir));
        std::fs::File::create(dir.join(TOKENIZER.file.file_name))
            .unwrap()
            .set_len(TOKENIZER.file.size_bytes)
            .unwrap();
        assert!(installed(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
