//! Plans `language-model-suggestions` and `languages-in-general`: Your languages and the Home
//! card of a suggested setup. Both are saved at once by their own controls (onboarding,
//! Settings → General, Home), not by the Settings Save bar, and every window gets the change as
//! `config:patch`.

use serde_json::json;
use tauri::Emitter;

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

/// Saves Your languages (written-language codes such as `en`, `zh-Hant-HK`) and returns them as
/// stored (normalized).
#[tauri::command]
pub async fn set_your_languages(
    app: tauri::AppHandle,
    state: tauri::State<'_, storage::ConfigManager>,
    languages: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut config = state.load().await.map_err(|error| error.to_string())?;
    config.your_languages = Some(languages);
    state
        .save(&config)
        .await
        .map_err(|error| error.to_string())?;
    let config = state.load().await.map_err(|error| error.to_string())?;
    emit_patch(&app, &config);
    tracing::info!(
        "Your languages saved: {}",
        config.your_languages.clone().unwrap_or_default().join(", ")
    );
    Ok(config.your_languages.unwrap_or_default())
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
