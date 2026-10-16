//! Plan `language-model-suggestions`: the languages the user speaks and the Home card of a
//! suggested setup. Both are saved at once by their own controls (onboarding, Settings, Home),
//! not by the Settings Save bar, and every window gets the change as `config:patch`.

use serde_json::json;
use tauri::Emitter;

use crate::storage;

fn emit_patch(app: &tauri::AppHandle, config: &storage::AppConfig) {
    let _ = app.emit(
        "config:patch",
        json!({
            "spoken_languages": config.spoken_languages,
            "dismissed_language_setup": config.dismissed_language_setup,
        }),
    );
}

/// Saves the languages the user speaks (codes such as `en`, `yue`, `zh`) and returns them as
/// stored (normalized).
#[tauri::command]
pub async fn set_spoken_languages(
    app: tauri::AppHandle,
    state: tauri::State<'_, storage::ConfigManager>,
    languages: Vec<String>,
) -> Result<Vec<String>, String> {
    let mut config = state.load().await.map_err(|error| error.to_string())?;
    config.spoken_languages = Some(languages);
    state
        .save(&config)
        .await
        .map_err(|error| error.to_string())?;
    let config = state.load().await.map_err(|error| error.to_string())?;
    emit_patch(&app, &config);
    tracing::info!(
        "Spoken languages saved: {}",
        config
            .spoken_languages
            .clone()
            .unwrap_or_default()
            .join(", ")
    );
    Ok(config.spoken_languages.unwrap_or_default())
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
