use std::fs;

use tauri::Manager;

use crate::error::{AppError, AppResult};

#[tauri::command]
#[specta::specta]
pub fn read_log_file(app: tauri::AppHandle, limit: Option<usize>) -> AppResult<Vec<String>> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    let log_path = log_dir.join("Poketto.log");

    if !log_path.exists() {
        return Ok(Vec::new());
    }

    let content = fs::read_to_string(&log_path)?;
    let lines: Vec<String> = content.lines().map(String::from).collect();

    match limit {
        Some(n) if n < lines.len() => Ok(lines[lines.len() - n..].to_vec()),
        _ => Ok(lines),
    }
}

#[tauri::command]
#[specta::specta]
pub fn get_log_path(app: tauri::AppHandle) -> AppResult<String> {
    let log_dir = app
        .path()
        .app_log_dir()
        .map_err(|e| AppError::Io(e.to_string()))?;
    let log_path = log_dir.join("Poketto.log");
    Ok(log_path.to_string_lossy().to_string())
}
