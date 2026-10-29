//! Plans `language-model-suggestions` and `languages-in-general`: Your languages and the Home
//! card of a suggested setup. Both are saved at once by their own controls (onboarding,
//! Settings → General, Home), not by the Settings Save bar, and every window gets the change as
//! `config:patch`.

use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::json;
use tauri::Emitter;

use crate::commands::language_presets::{
    give_bundled_presets, library_store, LIBRARY_CHANGED_EVENT,
};
use crate::storage;

fn emit_patch(app: &tauri::AppHandle, config: &storage::AppConfig) {
    let _ = app.emit(
        "config:patch",
        json!({
            "your_languages": config.your_languages,
            "dismissed_language_setup": config.dismissed_language_setup,
        }),
    );
}

/// What `set_your_languages` stored.
#[derive(Debug, Clone, Serialize)]
pub struct YourLanguagesSaved {
    /// Your languages as stored (normalized).
    pub your_languages: Vec<String>,
    /// Every language's settings, with the built-in presets just given to added languages.
    pub languages: BTreeMap<String, storage::TranslationLanguageSettings>,
}

/// Saves Your languages (written-language codes such as `en`, `zh-Hant-HK`). Plan
/// `languages-in-general`: an added language without settings gets its built-in preset at
/// once, so the row shows it and dictation recognises the language.
#[tauri::command]
pub async fn set_your_languages(
    app: tauri::AppHandle,
    state: tauri::State<'_, storage::ConfigManager>,
    languages: Vec<String>,
) -> Result<YourLanguagesSaved, String> {
    let mut config = state.load().await.map_err(|error| error.to_string())?;
    config.your_languages = Some(languages);
    config.normalize_values();
    let store = library_store(&app).ok();
    let given = store
        .as_ref()
        .map(|store| give_bundled_presets(&mut config, store))
        .unwrap_or_default();
    state
        .save(&config)
        .await
        .map_err(|error| error.to_string())?;
    if let Some(store) = &store {
        if !given.is_empty() {
            let _ = store.record_bundled_given(&given);
        }
    }
    let config = state.load().await.map_err(|error| error.to_string())?;
    emit_patch(&app, &config);
    if !given.is_empty() {
        // Other windows take only the language settings, as after an automatic update.
        let _ = app.emit(
            LIBRARY_CHANGED_EVENT,
            json!({ "languages": config.translation.languages }),
        );
    }
    tracing::info!(
        "Your languages saved: {}",
        config.your_languages.clone().unwrap_or_default().join(", ")
    );
    Ok(YourLanguagesSaved {
        your_languages: config.your_languages.unwrap_or_default(),
        languages: config.translation.languages,
    })
}

/// "Don't show again" on the Home card of the suggested setup `setup_id`.
#[tauri::command]
pub async fn dismiss_language_setup(
    app: tauri::AppHandle,
    state: tauri::State<'_, storage::ConfigManager>,
    setup_id: String,
) -> Result<(), String> {
    let mut config = state.load().await.map_err(|error| error.to_string())?;
    config.dismissed_language_setup = Some(setup_id);
    state
        .save(&config)
        .await
        .map_err(|error| error.to_string())?;
    let config = state.load().await.map_err(|error| error.to_string())?;
    emit_patch(&app, &config);
    Ok(())
}
