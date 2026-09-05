use tauri::State;

use crate::database::{save_settings, AppDatabase};
use crate::error::{AppError, AppResult};
use crate::models::{WineSettings, WineVersion};
use crate::state::AppState;
use crate::wine;

#[tauri::command]
#[specta::specta]
pub fn get_platform() -> String {
    #[cfg(target_os = "linux")]
    {
        "linux".to_string()
    }
    #[cfg(target_os = "windows")]
    {
        "windows".to_string()
    }
    #[cfg(target_os = "macos")]
    {
        "macos".to_string()
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        "unknown".to_string()
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_available_wine_versions(state: State<AppState>) -> Vec<WineVersion> {
    #[cfg(target_os = "linux")]
    {
        state.wine_versions.lock().clone()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = state;
        Vec::new()
    }
}

#[tauri::command]
#[specta::specta]
pub async fn refresh_wine_versions(state: State<'_, AppState>) -> AppResult<Vec<WineVersion>> {
    #[cfg(target_os = "linux")]
    {
        let versions = tokio::task::spawn_blocking(wine::get_all_wine_versions)
            .await
            .unwrap_or_default();
        let mut cache = state.wine_versions.lock();
        *cache = versions.clone();
        Ok(versions)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = state;
        Ok(Vec::new())
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_default_wine_settings(state: State<AppState>) -> WineSettings {
    let settings = state.settings.lock();

    let mut wine_settings = WineSettings {
        use_global_prefix: true,
        wine_prefix: settings.default_wine_prefix.clone(),
        wine_version: settings.default_wine_binary.clone(),
        wine_type: None,
        use_steam_runtime: settings.use_steam_runtime,
        env_vars: std::collections::HashMap::new(),
    };

    #[cfg(target_os = "linux")]
    if wine_settings.wine_version.is_none() {
        if let Some(default_wine) = wine::get_default_wine() {
            wine_settings.wine_version = Some(default_wine.binary_path);
            wine_settings.wine_type = Some(default_wine.wine_type);
        }
    }

    wine_settings
}

#[tauri::command]
#[specta::specta]
pub fn get_default_prefix_path(game_id: String, db: State<AppDatabase>) -> String {
    #[cfg(target_os = "linux")]
    {
        let vndb_id: Option<String> = db
            .get_game_by_id(&game_id)
            .ok()
            .flatten()
            .and_then(|game| game.vndb_id);
        wine::get_default_prefix_path(&game_id, vndb_id.as_deref())
            .to_str()
            .unwrap_or("")
            .to_string()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = game_id;
        let _ = db;
        String::new()
    }
}

#[tauri::command]
#[specta::specta]
pub fn save_game_wine_settings(
    game_id: String,
    wine_settings: WineSettings,
    db: State<AppDatabase>,
) -> AppResult<()> {
    let mut game = db
        .get_game_by_id(&game_id)?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

    game.wine_settings = Some(wine_settings);

    db.update_game(&game)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn save_global_wine_defaults(
    prefix: Option<String>,
    binary: Option<String>,
    use_steam_runtime: bool,
    state: State<AppState>,
) -> AppResult<()> {
    let mut settings = state.settings.lock();

    settings.default_wine_prefix = prefix;
    settings.default_wine_binary = binary;
    settings.use_steam_runtime = use_steam_runtime;

    save_settings(&settings)?;

    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn is_steam_runtime_available() -> bool {
    #[cfg(target_os = "linux")]
    {
        wine::is_steam_runtime_available()
    }
    #[cfg(not(target_os = "linux"))]
    {
        false
    }
}

#[tauri::command]
#[specta::specta]
pub fn validate_wine_binary(binary_path: String) -> AppResult<String> {
    #[cfg(target_os = "linux")]
    {
        wine::validate_wine_binary(&binary_path).map_err(AppError::ProcessLaunch)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = binary_path;
        Err(AppError::ProcessLaunch(
            "Wine is only supported on Linux".into(),
        ))
    }
}
