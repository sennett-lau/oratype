//! Plan `languages-in-general`: the official language presets built into the app, so a language
//! added to Your languages gets its preset without a download. They are the repository's
//! `presets/languages/` files at build time, checked against the built-in `index.json` with the
//! same size, SHA-256 and validation checks as a download (`fetch::verify_file`). A newer version
//! arrives with an app update and is applied at startup; the daily online check runs only for
//! languages whose automatic updates the user turned on.

use std::sync::OnceLock;

use super::{fetch, find_presets_for, parse_index, IndexEntry, Preset};

const INDEX_JSON: &str = include_str!("../../../../presets/languages/index.json");

/// Every official preset, by id. A test checks this list against the official entries of
/// `index.json`, so a new official preset must be added here.
const FILES: &[(&str, &str)] = &[
    (
        "cantonese-hong-kong",
        include_str!("../../../../presets/languages/cantonese-hong-kong/preset.md"),
    ),
    (
        "english",
        include_str!("../../../../presets/languages/english/preset.md"),
    ),
    (
        "mandarin-taiwan",
        include_str!("../../../../presets/languages/mandarin-taiwan/preset.md"),
    ),
];

/// A built-in preset that passed the checks.
#[derive(Debug)]
pub struct BundledPreset {
    pub entry: IndexEntry,
    pub bytes: &'static [u8],
    pub preset: Preset,
}

/// The built-in presets. One that fails its checks is left out (and logged).
pub fn bundled_presets() -> &'static [BundledPreset] {
    static PRESETS: OnceLock<Vec<BundledPreset>> = OnceLock::new();
    PRESETS.get_or_init(load)
}

fn load() -> Vec<BundledPreset> {
    let index = match parse_index(INDEX_JSON.as_bytes()) {
        Ok(index) => index,
        Err(error) => {
            tracing::warn!("Language presets: the built-in index is not usable: {error}");
            return Vec::new();
        }
    };
    FILES
        .iter()
        .filter_map(|(id, text)| {
            let Some(entry) = index.entry(id).filter(|entry| entry.tier == "official") else {
                tracing::warn!("Language presets: built-in {id} is not an official preset");
                return None;
            };
            match fetch::verify_file(entry, text.as_bytes()) {
                Ok(preset) => Some(BundledPreset {
                    entry: entry.clone(),
                    bytes: text.as_bytes(),
                    preset,
                }),
                Err(error) => {
                    tracing::warn!("Language presets: built-in {id}: {error}");
                    None
                }
            }
        })
        .collect()
}

/// The built-in preset for a language code: the best direct match, as Browse lists it first
/// (`zh-Hant-HK` → Cantonese (Hong Kong), `en` → English). None for a language that has no
/// official preset (`zh-Hans`, `ja` today).
pub fn bundled_preset_for(code: &str) -> Option<&'static BundledPreset> {
    let presets = bundled_presets();
    let entries: Vec<IndexEntry> = presets
        .iter()
        .map(|bundled| bundled.entry.clone())
        .collect();
    let (matches, _) = find_presets_for(&entries, code);
    let best = matches.into_iter().find(|found| found.direct)?;
    presets.iter().find(|bundled| bundled.entry.id == best.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    /// Every official preset of the repository is built in, and each one passes the download
    /// checks: its size and SHA-256 are the index's.
    #[test]
    fn every_official_preset_is_built_in_and_verified() {
        let index = parse_index(INDEX_JSON.as_bytes()).unwrap();
        let mut official: Vec<&str> = index
            .presets
            .iter()
            .filter(|entry| entry.tier == "official")
            .map(|entry| entry.id.as_str())
            .collect();
        official.sort();
        let mut built_in: Vec<&str> = bundled_presets()
            .iter()
            .map(|bundled| bundled.entry.id.as_str())
            .collect();
        built_in.sort();
        assert_eq!(built_in, official);
        assert_eq!(FILES.len(), official.len());

        for bundled in bundled_presets() {
            assert_eq!(
                format!("{:x}", Sha256::digest(bundled.bytes)),
                bundled.entry.sha256
            );
            assert_eq!(bundled.preset.sha256, bundled.entry.sha256);
            assert_eq!(bundled.preset.version, bundled.entry.version);
            // The built-in index is the repository's.
            let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../presets/languages")
                .join(&bundled.entry.path);
            assert_eq!(std::fs::read(file).unwrap(), bundled.bytes);
        }
    }

    #[test]
    fn a_changed_built_in_file_fails_the_check() {
        let bundled = bundled_preset_for("en").unwrap();
        let mut bytes = bundled.bytes.to_vec();
        let last = bytes.len() - 2;
        bytes[last] = b'!';
        assert_eq!(
            fetch::verify_file(&bundled.entry, &bytes).unwrap_err(),
            fetch::VERIFY_FAILED
        );
    }

    #[test]
    fn languages_find_their_official_preset() {
        let id = |code: &str| bundled_preset_for(code).map(|bundled| bundled.entry.id.as_str());
        assert_eq!(id("zh-Hant-HK"), Some("cantonese-hong-kong"));
        assert_eq!(id("en"), Some("english"));
        assert_eq!(id("zh-Hant-TW"), Some("mandarin-taiwan"));
        // No official preset: Simplified Mandarin, Japanese.
        assert_eq!(id("zh-Hans"), None);
        assert_eq!(id("ja"), None);
    }
}
