//! Plan `language-prompt-library`: commands behind the language sheet's Browse and Preview
//! screens, and the library status that Settings shows (versions, automatic updates). Plan
//! `languages-in-general`: chosen languages without settings get their built-in preset.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use serde::Serialize;
use tauri::{Emitter, Manager};

use crate::llm::language_library::{
    self as library,
    fetch::{self, FetchError},
    store::{AutoUpdateRecord, IndexMeta, LibraryStore},
    Index, IndexEntry, Preset,
};
use crate::storage::{AppConfig, ConfigManager, LibraryPresetRef, TranslationLanguageSettings};

/// Emitted when the library cache or an automatic update changed; the payload holds the saved
/// `translation.languages` when an update changed them.
pub const LIBRARY_CHANGED_EVENT: &str = "language-library:changed";
/// The daily check runs at most this often.
const AUTO_UPDATE_INTERVAL_SECS: i64 = 24 * 60 * 60;
/// How often the app looks whether the daily check is due, and the wait after startup.
const AUTO_UPDATE_POLL: std::time::Duration = std::time::Duration::from_secs(60 * 60);
const AUTO_UPDATE_STARTUP_DELAY: std::time::Duration = std::time::Duration::from_secs(90);

fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(fetch::client)
}

pub(crate) fn library_store(app: &tauri::AppHandle) -> Result<LibraryStore, String> {
    app.path()
        .app_data_dir()
        .map(|dir| LibraryStore::in_app_data(&dir))
        .map_err(|error| error.to_string())
}

/// One preset in the Browse list.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PresetListing {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub summary: String,
    pub languages: Vec<String>,
    /// The regional note used for this language, if any ("notes for en-GB").
    pub variant: Option<String>,
    pub version: i64,
    pub authors: Vec<String>,
    pub model_hint: Option<String>,
    /// This version is on disk.
    pub downloaded: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PresetList {
    /// Matching presets, best first (the built-in default is added by the sheet).
    pub presets: Vec<PresetListing>,
    /// Names of presets for the same language that did not match.
    pub related: Vec<String>,
    /// The library could not be reached; only downloaded presets are listed.
    pub offline: bool,
}

/// A preset rendered for one language: what Preview shows and what "Use this preset" stores.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PresetDetail {
    pub id: String,
    pub name: String,
    pub tier: String,
    pub summary: String,
    pub version: i64,
    pub sha256: String,
    pub authors: Vec<String>,
    pub model_hint: Option<String>,
    /// The full text the AI gets for this language.
    pub text: String,
    pub variant: Option<String>,
    pub detect_codes: Vec<String>,
    pub hints: Vec<String>,
    pub require_hint: bool,
    pub applies_to: Vec<String>,
}

pub(crate) fn detail(preset: &Preset, code: &str) -> PresetDetail {
    PresetDetail {
        id: preset.id.clone(),
        name: preset.name.clone(),
        tier: preset.tier.clone(),
        summary: preset.summary.clone(),
        version: preset.version,
        sha256: preset.sha256.clone(),
        authors: preset.authors.clone(),
        model_hint: preset.model_hint.clone(),
        text: library::render_for_code(preset, code),
        variant: library::variant_for_code(preset.variants.keys(), code).map(str::to_string),
        detect_codes: preset.detect_codes.clone(),
        hints: preset.hints.clone(),
        require_hint: preset.require_hint,
        applies_to: preset.applies_to.clone(),
    }
}

/// The Browse list for `code` from an index: matches best first, related names, and whether
/// each is downloaded. `offline` keeps only downloaded presets.
pub(crate) fn preset_list(
    index: &library::Index,
    store: &LibraryStore,
    code: &str,
    offline: bool,
) -> PresetList {
    let (matches, related) = library::find_presets_for(&index.presets, code);
    let listing = |entry: &IndexEntry| PresetListing {
        id: entry.id.clone(),
        name: entry.name.clone(),
        tier: entry.tier.clone(),
        summary: entry.summary.clone(),
        languages: entry.languages.clone(),
        variant: library::variant_for_code(&entry.variants, code).map(str::to_string),
        version: entry.version,
        authors: entry.authors.clone(),
        model_hint: entry.model_hint.clone(),
        downloaded: store.has_preset(&entry.id, &entry.sha256),
    };
    let presets = matches
        .iter()
        .filter_map(|m| index.entry(&m.id))
        .map(listing)
        .filter(|item| !offline || item.downloaded)
        .collect();
    let related = related
        .iter()
        .filter_map(|id| index.entry(id))
        .map(|entry| entry.name.clone())
        .collect();
    PresetList {
        presets,
        related,
        offline,
    }
}

/// Browse: refreshes the index (a conditional GET) and lists the presets for `code`. Offline,
/// lists the downloaded ones from the cached index.
#[tauri::command]
pub async fn list_language_presets(
    app: tauri::AppHandle,
    code: String,
) -> Result<PresetList, String> {
    let store = library_store(&app)?;
    let (index, offline) =
        match fetch::refresh_index(http_client(), library::LIBRARY_BASE_URL, &store).await {
            Ok(index) => {
                // Settings may now know about newer versions ("Update" tags).
                let _ = app.emit(
                    LIBRARY_CHANGED_EVENT,
                    serde_json::json!({ "languages": null }),
                );
                (index, false)
            }
            Err(error) => {
                if let FetchError::Invalid(reason) = &error {
                    tracing::warn!("Language presets: {reason}");
                }
                (store.cached_index().unwrap_or_default(), true)
            }
        };
    Ok(preset_list(&index, &store, &code, offline))
}

/// Preview: downloads preset `id` if needed (size, SHA-256 and validation checked) and renders
/// it for `code`. Nothing is used until the sheet saves it.
#[tauri::command]
pub async fn download_language_preset(
    app: tauri::AppHandle,
    config_manager: tauri::State<'_, crate::storage::ConfigManager>,
    id: String,
    code: String,
) -> Result<PresetDetail, String> {
    let store = library_store(&app)?;
    let (_, preset) =
        fetch::ensure_preset(http_client(), library::LIBRARY_BASE_URL, &store, &id).await?;
    if let Ok(config) = config_manager.load().await {
        store.prune(&id, &versions_in_use(&config, &id, &preset.sha256));
    }
    tracing::info!(
        "Language presets: {} v{} ready for {code}",
        preset.id,
        preset.version
    );
    Ok(detail(&preset, &code))
}

/// The stored versions of `id` to keep: the ones languages use, plus `newest`.
pub(crate) fn versions_in_use(
    config: &crate::storage::AppConfig,
    id: &str,
    newest: &str,
) -> Vec<String> {
    let mut keep: Vec<String> = config
        .translation
        .languages
        .values()
        .filter_map(|settings| settings.library_preset.as_ref())
        .filter(|preset| preset.id == id)
        .map(|preset| preset.sha256.clone())
        .collect();
    if !newest.is_empty() {
        keep.push(newest.to_string());
    }
    keep
}

/// A stored version of a preset rendered for `code`, or `None` when it is missing or damaged
/// (the sheet then offers to download it again). Never goes online.
#[tauri::command]
pub fn load_language_preset(
    app: tauri::AppHandle,
    id: String,
    sha256: String,
    code: String,
) -> Result<Option<PresetDetail>, String> {
    let store = library_store(&app)?;
    Ok(store
        .load_preset(&id, &sha256)
        .ok()
        .map(|preset| detail(&preset, &code)))
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LatestVersion {
    pub version: i64,
    pub sha256: String,
}

/// What Settings needs to show "Update" tags and banners, from the cache only.
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LibraryStatus {
    /// Newest version of every preset in the cached index.
    pub latest: BTreeMap<String, LatestVersion>,
    /// Automatic updates by language code.
    pub updates: BTreeMap<String, AutoUpdateRecord>,
}

#[tauri::command]
pub fn get_language_library_status(app: tauri::AppHandle) -> Result<LibraryStatus, String> {
    let store = library_store(&app)?;
    let latest = store
        .cached_index()
        .unwrap_or_default()
        .presets
        .into_iter()
        .map(|entry| {
            (
                entry.id,
                LatestVersion {
                    version: entry.version,
                    sha256: entry.sha256,
                },
            )
        })
        .collect();
    Ok(LibraryStatus {
        latest,
        updates: store.updates(),
    })
}

// ─── Automatic updates ───

/// The chosen languages (Your languages, then the Translate ones; plan `languages-in-general`)
/// that want automatic updates: on, auto-update on, using a preset.
pub(crate) fn auto_update_languages(config: &AppConfig) -> Vec<(String, LibraryPresetRef)> {
    config
        .chosen_language_codes()
        .into_iter()
        .filter_map(|code| {
            let settings = config.translation.language(&code)?;
            let preset = settings.library_preset.clone()?;
            (settings.enabled && settings.auto_update).then_some((code, preset))
        })
        .collect()
}

/// Whether the daily check is due.
pub(crate) fn auto_check_due(meta: &IndexMeta, now: i64) -> bool {
    meta.auto_checked_at
        .is_none_or(|checked| now - checked >= AUTO_UPDATE_INTERVAL_SECS)
}

/// The languages to move to a newer version: auto-update on and not edited. Edited languages
/// keep their text; the sheet offers the new version instead.
pub(crate) fn plan_auto_updates(config: &AppConfig, index: &Index) -> Vec<(String, IndexEntry)> {
    auto_update_languages(config)
        .into_iter()
        .filter(|(code, _)| {
            config
                .translation
                .language(code)
                .is_some_and(|settings| settings.instructions.is_none())
        })
        .filter_map(|(code, preset)| {
            let entry = index.entry(&preset.id)?;
            (entry.version > preset.version).then(|| (code, entry.clone()))
        })
        .collect()
}

/// Applies downloaded versions to the languages in `plan`. Returns the records of what changed.
pub(crate) fn apply_auto_updates(
    config: &mut AppConfig,
    updates: &[(String, Preset)],
    now: i64,
) -> Vec<(String, AutoUpdateRecord)> {
    let mut records = Vec::new();
    for (code, preset) in updates {
        let Some(settings) = config.translation.languages.get_mut(code) else {
            continue;
        };
        let Some(current) = settings.library_preset.as_mut() else {
            continue;
        };
        if current.id != preset.id || preset.version <= current.version {
            continue;
        }
        records.push((
            code.clone(),
            AutoUpdateRecord {
                id: preset.id.clone(),
                from: current.version,
                to: preset.version,
                at: now,
            },
        ));
        *current = LibraryPresetRef {
            id: preset.id.clone(),
            version: preset.version,
            sha256: preset.sha256.clone(),
        };
    }
    records
}

/// Plan `language-prompt-library`: the daily check. Runs only while a chosen language with
/// auto-update on uses a preset, at most once a day; fetches the index (conditional GET) and
/// moves languages that are not edited to the newest version. Logs ids, versions and codes.
pub async fn run_auto_update(app: &tauri::AppHandle) {
    let config_manager = app.state::<crate::storage::ConfigManager>();
    let Ok(config) = config_manager.load().await else {
        return;
    };
    if auto_update_languages(&config).is_empty() {
        return;
    }
    let Ok(store) = library_store(app) else {
        return;
    };
    let now = fetch::now_unix();
    if !auto_check_due(&store.index_meta(), now) {
        return;
    }
    let _ = store.mark_auto_checked(now);
    let index = match fetch::refresh_index(http_client(), library::LIBRARY_BASE_URL, &store).await {
        Ok(index) => index,
        Err(error) => {
            tracing::info!("Language presets: daily check skipped: {error}");
            return;
        }
    };
    let mut downloaded = Vec::new();
    for (code, entry) in plan_auto_updates(&config, &index) {
        match fetch::ensure_preset(http_client(), library::LIBRARY_BASE_URL, &store, &entry.id)
            .await
        {
            Ok((_, preset)) => downloaded.push((code, preset)),
            Err(error) => tracing::warn!(
                "Language presets: could not update {} for {code}: {error}",
                entry.id
            ),
        }
    }
    // Load again: the user may have saved while the downloads ran.
    let Ok(mut config) = config_manager.load().await else {
        return;
    };
    let records = apply_auto_updates(&mut config, &downloaded, now);
    if !records.is_empty() {
        if let Err(error) = config_manager.save(&config).await {
            tracing::warn!("Language presets: could not save the updates: {error}");
            return;
        }
        for (code, record) in &records {
            tracing::info!(
                "Language presets: {code} updated automatically: {} v{} -> v{}",
                record.id,
                record.from,
                record.to
            );
            let _ = store.record_update(code, record.clone());
            store.prune(&record.id, &versions_in_use(&config, &record.id, ""));
        }
    }
    let _ = app.emit(
        LIBRARY_CHANGED_EVENT,
        serde_json::json!({
            "languages": (!records.is_empty()).then(|| config.translation.languages.clone()),
        }),
    );
}

// ─── Built-in presets (plan `languages-in-general`) ───

/// Gives every chosen language (Your languages, then Translate's) that has no settings yet its
/// official built-in preset, stored like a download. Automatic updates stay off: OraType does not
/// go online for a preset the user did not pick, and newer built-in versions come with app
/// updates ([`bundled_preset_updates`]). A language with settings (another preset, its own text, switched off, its own
/// hints) is never touched, and one given its preset before is not given it again, so a language
/// moved back to the built-in instructions keeps them. Returns the codes given a preset; the
/// caller saves the config, then records them (`LibraryStore::record_bundled_given`). No network.
pub(crate) fn give_bundled_presets(config: &mut AppConfig, store: &LibraryStore) -> Vec<String> {
    let given_before = store.bundled_given();
    let mut given = Vec::new();
    for code in config.chosen_language_codes() {
        if config.translation.languages.contains_key(&code) || given_before.contains(&code) {
            continue;
        }
        let Some(bundled) = library::bundled::bundled_preset_for(&code) else {
            continue;
        };
        let entry = &bundled.entry;
        if !store.has_preset(&entry.id, &entry.sha256) {
            if let Err(error) = store.store_preset(&entry.id, bundled.bytes) {
                tracing::warn!(
                    "Language presets: could not store the built-in {} for {code}: {error}",
                    entry.id
                );
                continue;
            }
        }
        config.translation.languages.insert(
            code.clone(),
            TranslationLanguageSettings {
                library_preset: Some(LibraryPresetRef {
                    id: entry.id.clone(),
                    version: entry.version,
                    sha256: entry.sha256.clone(),
                }),
                ..TranslationLanguageSettings::default()
            },
        );
        tracing::info!(
            "Language presets: {code} uses the built-in {} v{}",
            entry.id,
            entry.version
        );
        given.push(code);
    }
    given
}

/// The chosen languages whose preset is an official one this app has a newer built-in version
/// of (it came with an app update), with that version, stored like a download. Applied with
/// [`apply_auto_updates`], like the daily check, so edited text is still never replaced. No
/// network.
pub(crate) fn bundled_preset_updates(
    config: &AppConfig,
    store: &LibraryStore,
) -> Vec<(String, library::Preset)> {
    let mut updates = Vec::new();
    for code in config.chosen_language_codes() {
        let Some(current) = config
            .translation
            .language(&code)
            .and_then(|settings| settings.library_preset.as_ref())
        else {
            continue;
        };
        let Some(bundled) = library::bundled::bundled_presets().iter().find(|bundled| {
            bundled.entry.id == current.id && bundled.entry.version > current.version
        }) else {
            continue;
        };
        if !store.has_preset(&bundled.entry.id, &bundled.entry.sha256) {
            if let Err(error) = store.store_preset(&bundled.entry.id, bundled.bytes) {
                tracing::warn!(
                    "Language presets: could not store the built-in {} for {code}: {error}",
                    bundled.entry.id
                );
                continue;
            }
        }
        updates.push((code, bundled.preset.clone()));
    }
    updates
}

/// At startup: gives chosen languages without settings their built-in preset (users who added
/// languages before presets were built in, or whose list was migrated), moves official presets
/// to the newer built-in version an app update brought, and saves the config when that changed
/// it. Runs before any window loads the config.
pub(crate) async fn give_bundled_presets_at_startup(
    app: &tauri::AppHandle,
    config_manager: &ConfigManager,
) {
    let Ok(store) = library_store(app) else {
        return;
    };
    let Ok(mut config) = config_manager.load().await else {
        return;
    };
    let given = give_bundled_presets(&mut config, &store);
    let updates = bundled_preset_updates(&config, &store);
    let records = apply_auto_updates(&mut config, &updates, fetch::now_unix());
    if given.is_empty() && records.is_empty() {
        return;
    }
    if let Err(error) = config_manager.save(&config).await {
        tracing::warn!("Language presets: could not save the built-in presets: {error}");
        return;
    }
    let _ = store.record_bundled_given(&given);
    for (code, record) in &records {
        tracing::info!(
            "Language presets: {code} moved to the built-in {} v{} (was v{})",
            record.id,
            record.to,
            record.from
        );
        let _ = store.record_update(code, record.clone());
        store.prune(&record.id, &versions_in_use(&config, &record.id, ""));
    }
}

/// Starts the daily check loop (after a short delay at startup, then every hour it looks
/// whether a check is due).
pub fn start_auto_updates(app: tauri::AppHandle) {
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(AUTO_UPDATE_STARTUP_DELAY).await;
        loop {
            run_auto_update(&app).await;
            tokio::time::sleep(AUTO_UPDATE_POLL).await;
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::language_library::store::tests::{english_bytes, temp_store};

    fn repository_index() -> library::Index {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../presets/languages/index.json");
        library::parse_index(&std::fs::read(path).unwrap()).unwrap()
    }

    #[test]
    fn browse_lists_matches_with_their_variant_and_download_state() {
        let store = temp_store("browse");
        let index = repository_index();
        let list = preset_list(&index, &store, "zh-Hant-HK", false);
        assert_eq!(list.presets[0].id, "cantonese-hong-kong");
        assert_eq!(list.presets[0].tier, "official");
        assert!(!list.presets[0].downloaded);
        assert!(list
            .related
            .iter()
            .any(|name| name.starts_with("Mandarin (Taiwan)")));

        store.store_preset("english", &english_bytes()).unwrap();
        let english = preset_list(&index, &store, "en", false);
        assert_eq!(english.presets.len(), 1);
        assert!(english.presets[0].downloaded);
        assert_eq!(english.presets[0].variant, None);
    }

    #[test]
    fn offline_browse_lists_only_downloaded_presets() {
        let store = temp_store("browse-offline");
        let index = repository_index();
        assert!(preset_list(&index, &store, "zh-Hant-HK", true)
            .presets
            .is_empty());
        store.store_preset("english", &english_bytes()).unwrap();
        let list = preset_list(&index, &store, "en", true);
        assert!(list.offline);
        assert_eq!(list.presets.len(), 1);
    }

    /// Plan `languages-in-general`: Your languages without settings, as the dev build had them.
    fn unset_languages(codes: &[&str]) -> AppConfig {
        let mut config = AppConfig::default();
        config.your_languages = Some(codes.iter().map(|code| code.to_string()).collect());
        config.translation.targets = vec!["ja".to_string()];
        config
    }

    #[test]
    fn added_languages_get_their_built_in_preset() {
        let store = temp_store("bundled-give");
        let mut config = unset_languages(&["en", "zh-Hant-HK", "zh-Hans"]);
        let given = give_bundled_presets(&mut config, &store);
        assert_eq!(given, ["en", "zh-Hant-HK"]);

        for (code, id) in [("en", "english"), ("zh-Hant-HK", "cantonese-hong-kong")] {
            let settings = config.translation.language(code).unwrap();
            let preset = settings.library_preset.as_ref().unwrap();
            assert_eq!(preset.id, id);
            // No daily check for a preset the user did not pick.
            assert!(!settings.auto_update);
            assert!(settings.enabled);
            assert_eq!(settings.instructions, None);
            // Stored like a download: the hash checks out and it validates.
            let stored = store.load_preset(id, &preset.sha256).unwrap();
            assert_eq!(stored.version, preset.version);
        }
        // No official preset for Simplified Mandarin or Japanese (Translate only).
        assert!(config.translation.language("zh-Hans").is_none());
        assert!(config.translation.language("ja").is_none());

        // The router now hears both, with the presets' rules.
        let profiles = crate::llm::language_router::language_profiles(&config, Some(&store));
        assert_eq!(profiles[0].detect_codes, ["en"]);
        assert_eq!(profiles[1].detect_codes, ["yue", "zh"]);
        assert!(profiles[1].require_hint);
        assert!(profiles[1].instructions.starts_with("Written Cantonese"));
    }

    #[test]
    fn an_app_update_moves_official_presets_to_the_newer_built_in_version() {
        let store = temp_store("bundled-update");
        let english = library::bundled::bundled_presets()
            .iter()
            .find(|bundled| bundled.entry.id == "english")
            .unwrap();
        let mut config = unset_languages(&["en", "zh-Hant-HK"]);
        config.translation.languages = config_with(serde_json::json!({
            "en": {"library_preset": reference("english", 1), "instructions": "Mine."},
            "zh-Hant-HK": {"library_preset": reference("someone-else", 1)}
        }))
        .translation
        .languages;

        let updates = bundled_preset_updates(&config, &store);
        assert_eq!(
            updates
                .iter()
                .map(|(code, preset)| (code.as_str(), preset.version))
                .collect::<Vec<_>>(),
            [("en", english.entry.version)]
        );
        assert!(store.has_preset("english", &english.entry.sha256));
        let records = apply_auto_updates(&mut config, &updates, 10);
        assert_eq!(records.len(), 1);
        let settings = config.translation.language("en").unwrap();
        assert_eq!(
            settings.library_preset.as_ref().unwrap().version,
            english.entry.version
        );
        // The user's own text still wins over any preset version.
        assert_eq!(settings.instructions.as_deref(), Some("Mine."));
        // Once current, nothing more to do.
        assert!(bundled_preset_updates(&config, &store).is_empty());
    }

    #[test]
    fn languages_with_settings_are_left_alone() {
        let store = temp_store("bundled-keep");
        let mut config = unset_languages(&["en", "zh-Hant-HK", "zh-Hant-TW"]);
        let mine = config_with(serde_json::json!({
            "en": {"instructions": "My English."},
            "zh-Hant-HK": {"enabled": false},
            "zh-Hant-TW": {"user_hints": ["醬"]}
        }))
        .translation
        .languages;
        config.translation.languages = mine.clone();
        assert!(give_bundled_presets(&mut config, &store).is_empty());
        assert_eq!(config.translation.languages, mine);

        // Another preset stays too.
        let mut config = unset_languages(&["zh-Hant-HK"]);
        config.translation.languages = config_with(serde_json::json!({
            "zh-Hant-HK": {"library_preset": reference("someone-else", 2)}
        }))
        .translation
        .languages;
        assert!(give_bundled_presets(&mut config, &store).is_empty());
        assert_eq!(
            config
                .translation
                .language("zh-Hant-HK")
                .unwrap()
                .library_preset
                .as_ref()
                .unwrap()
                .id,
            "someone-else"
        );
    }

    /// Running again changes nothing, and a language moved back to the built-in instructions
    /// (its settings removed) is not given the preset again.
    #[test]
    fn giving_built_in_presets_is_idempotent() {
        let store = temp_store("bundled-once");
        let mut config = unset_languages(&["en", "zh-Hant-HK"]);
        let given = give_bundled_presets(&mut config, &store);
        store.record_bundled_given(&given).unwrap();
        let after_first = config.translation.languages.clone();
        assert!(give_bundled_presets(&mut config, &store).is_empty());
        assert_eq!(config.translation.languages, after_first);

        config.translation.languages.remove("zh-Hant-HK");
        assert!(give_bundled_presets(&mut config, &store).is_empty());
        assert!(config.translation.language("zh-Hant-HK").is_none());

        // A Translate language without settings counts as chosen too.
        config.translation.targets = vec!["zh-Hant-TW".to_string()];
        assert_eq!(give_bundled_presets(&mut config, &store), ["zh-Hant-TW"]);
    }

    fn config_with(languages: serde_json::Value) -> AppConfig {
        AppConfig::from_stored_value(serde_json::json!({
            "translation": {
                "targets": ["zh-Hant-HK", "en", "zh-Hant-TW"],
                "active_target": "en",
                "languages": languages
            }
        }))
        .unwrap()
    }

    fn reference(id: &str, version: i64) -> serde_json::Value {
        serde_json::json!({"id": id, "version": version, "sha256": "a".repeat(64)})
    }

    /// Plan `language-prompt-library`: only languages that are on, use a preset and have
    /// auto-update on take part; edited ones are offered the update instead of getting it.
    #[test]
    fn auto_updates_apply_only_to_languages_that_are_not_edited() {
        let config = config_with(serde_json::json!({
            "zh-Hant-HK": {"library_preset": reference("cantonese-hong-kong", 1), "auto_update": true},
            "en": {"library_preset": reference("english", 1), "auto_update": true, "instructions": "Mine."},
            "zh-Hant-TW": {"library_preset": reference("mandarin-taiwan", 1)}
        }));
        let codes: Vec<String> = auto_update_languages(&config)
            .into_iter()
            .map(|(code, _)| code)
            .collect();
        assert_eq!(codes, ["zh-Hant-HK", "en"]);

        let index = repository_index();
        let plan: Vec<String> = plan_auto_updates(&config, &index)
            .into_iter()
            .map(|(code, entry)| format!("{code}:{}:v{}", entry.id, entry.version))
            .collect();
        assert_eq!(plan, ["zh-Hant-HK:cantonese-hong-kong:v4"]);

        // Plan `languages-in-general`: a language only in Your languages takes part too.
        let mut spoken_only = config_with(serde_json::json!({
            "ko": {"library_preset": reference("korean", 1), "auto_update": true}
        }));
        assert!(auto_update_languages(&spoken_only).is_empty());
        spoken_only.your_languages = Some(vec!["ko".to_string()]);
        assert_eq!(auto_update_languages(&spoken_only)[0].0, "ko");

        // Off: not checked at all.
        let off = config_with(serde_json::json!({
            "zh-Hant-HK": {"library_preset": reference("cantonese-hong-kong", 1), "auto_update": true, "enabled": false}
        }));
        assert!(auto_update_languages(&off).is_empty());
        // Already at the newest version: nothing to do.
        let current = config_with(serde_json::json!({
            "zh-Hant-HK": {"library_preset": reference("cantonese-hong-kong", 4), "auto_update": true}
        }));
        assert!(plan_auto_updates(&current, &index).is_empty());
    }

    #[test]
    fn applying_an_update_moves_the_language_and_records_it() {
        let mut config = config_with(serde_json::json!({
            "zh-Hant-HK": {"library_preset": reference("cantonese-hong-kong", 1), "auto_update": true}
        }));
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../presets/languages/cantonese-hong-kong/preset.md"),
        )
        .unwrap();
        let preset = library::parse_preset("cantonese-hong-kong", &bytes).unwrap();
        let records = apply_auto_updates(&mut config, &[("zh-Hant-HK".into(), preset.clone())], 50);
        assert_eq!(records.len(), 1);
        assert_eq!(records[0].1.from, 1);
        assert_eq!(records[0].1.to, preset.version);
        let stored = config.translation.languages["zh-Hant-HK"]
            .library_preset
            .clone()
            .unwrap();
        assert_eq!(stored.version, preset.version);
        assert_eq!(stored.sha256, preset.sha256);
        // Applying the same version again changes nothing.
        assert!(apply_auto_updates(&mut config, &[("zh-Hant-HK".into(), preset)], 60).is_empty());
    }

    #[test]
    fn the_daily_check_runs_at_most_once_a_day() {
        let never = IndexMeta::default();
        assert!(auto_check_due(&never, 1_000));
        let recent = IndexMeta {
            auto_checked_at: Some(1_000),
            ..IndexMeta::default()
        };
        assert!(!auto_check_due(
            &recent,
            1_000 + AUTO_UPDATE_INTERVAL_SECS - 1
        ));
        assert!(auto_check_due(&recent, 1_000 + AUTO_UPDATE_INTERVAL_SECS));
    }

    #[test]
    fn the_detail_renders_for_the_language() {
        let preset = library::parse_preset("english", &english_bytes()).unwrap();
        let detail = detail(&preset, "en");
        assert_eq!(detail.detect_codes, ["en"]);
        assert!(detail.text.contains("Examples:"));
        assert_eq!(detail.variant, None);
    }
}
