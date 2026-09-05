use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

use crate::database::AppDatabase;
use crate::error::AppResult;
use crate::models::GameMetadata;

#[tauri::command]
#[specta::specta]
pub fn get_all_games(db: State<AppDatabase>) -> Vec<GameMetadata> {
    match db.get_all_games() {
        Ok(games) => games,
        Err(e) => {
            log::error!("Failed to load games from database: {e}");
            Vec::new()
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn add_local_game(path: String, db: State<AppDatabase>) -> AppResult<GameMetadata> {
    let path_buf = PathBuf::from(&path);
    let title = path_buf
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown Game")
        .to_string();

    let game = GameMetadata {
        id: Uuid::new_v4().to_string(),
        title,
        path,
        vndb_id: None,
        steam_app_id: None,
        cover_url: None,
        play_time: 0,
        is_finished: false,
        last_played: None,
        is_hidden: false,
        show_spoilers: false,
        game_type: None,
        wine_settings: None,
    };

    db.insert_game(&game)?;

    Ok(game)
}

#[tauri::command]
#[specta::specta]
pub fn remove_game(id: String, db: State<AppDatabase>) -> AppResult<()> {
    db.delete_game(&id)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn update_game(game: GameMetadata, db: State<AppDatabase>) -> AppResult<()> {
    db.update_game(&game)?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn set_game_hidden(id: String, hidden: bool, db: State<AppDatabase>) -> AppResult<()> {
    if let Some(mut game) = db.get_game_by_id(&id)? {
        game.is_hidden = hidden;
        db.update_game(&game)?;
    }
    Ok(())
}
