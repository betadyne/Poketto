use std::collections::HashMap;

use poketto_core::models::{Game, GameType, WineSettings, WineType};

use crate::GameFormData;

pub fn validate_game_form(title: &str, vndb_id: &str) -> Option<String> {
    if title.trim().is_empty() {
        return Some("Title is required.".to_string());
    }
    let id = vndb_id.trim();
    if !id.is_empty() && !is_vndb_id(id) {
        return Some("VNDB ID must look like v17.".to_string());
    }
    None
}

pub fn is_vndb_id(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some('v') | Some('V') => {}
        _ => return false,
    }
    let mut digits = 0;
    for c in chars {
        if !c.is_ascii_digit() {
            return false;
        }
        digits += 1;
    }
    digits > 0
}

pub fn apply_form(
    existing: Option<&Game>,
    form: &GameFormData,
    use_steam_runtime: bool,
    new_id: &str,
) -> Game {
    let text = |value: &str| {
        let trimmed = value.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    };
    let (game_type, wine_type) = match form.platform {
        1 => (GameType::WindowsExe, Some(WineType::Wine)),
        2 => (GameType::WindowsExe, Some(WineType::Proton)),
        _ => (GameType::LinuxNative, None),
    };
    let wine_prefix = text(form.wine_prefix.as_str());
    let wine_settings = wine_type.map(|wine_type| WineSettings {
        use_global_prefix: wine_prefix.is_none(),
        wine_prefix,
        wine_version: text(form.wine_binary.as_str()),
        wine_type: Some(wine_type),
        use_steam_runtime,
        env_vars: HashMap::new(),
    });
    Game {
        id: existing.map(|game| game.id.clone()).unwrap_or_else(|| new_id.to_string()),
        title: form.title.trim().to_string(),
        path: form.exec_path.trim().to_string(),
        work_dir: text(form.work_dir.as_str()),
        vndb_id: text(form.vndb_id.as_str()),
        cover_url: text(form.cover_url.as_str()),
        play_time_minutes: existing.map(|game| game.play_time_minutes).unwrap_or(0),
        is_finished: existing.map(|game| game.is_finished).unwrap_or(false),
        last_played: existing.and_then(|game| game.last_played.clone()),
        is_hidden: existing.map(|game| game.is_hidden).unwrap_or(false),
        show_spoilers: existing.map(|game| game.show_spoilers).unwrap_or(false),
        user_status: existing.map(|game| game.user_status).unwrap_or(0),
        user_vote: existing.map(|game| game.user_vote).unwrap_or(0),
        game_type: Some(game_type),
        wine_settings,
        rating: existing.and_then(|game| game.rating),
    }
}

pub fn platform_index(game: &Game) -> i32 {
    match game.game_type {
        Some(GameType::WindowsExe) => match game.wine_settings.as_ref().and_then(|w| w.wine_type.clone()) {
            Some(
                WineType::Proton | WineType::ProtonGE | WineType::ProtonCachyOS | WineType::ProtonTKG,
            ) => 2,
            _ => 1,
        },
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form() -> GameFormData {
        GameFormData {
            id: "".into(),
            title: "Test Game".into(),
            vndb_id: "v17".into(),
            exec_path: "/games/test/game.exe".into(),
            work_dir: "".into(),
            cover_url: "".into(),
            platform: 2,
            wine_prefix: "".into(),
            wine_binary: "".into(),
        }
    }

    #[test]
    fn title_is_required() {
        assert_eq!(
            validate_game_form("  ", ""),
            Some("Title is required.".to_string())
        );
        assert_eq!(validate_game_form("Game", ""), None);
    }

    #[test]
    fn vndb_id_shape_is_checked() {
        assert_eq!(validate_game_form("Game", "v17"), None);
        assert_eq!(validate_game_form("Game", "V104"), None);
        assert!(validate_game_form("Game", "17").is_some());
        assert!(validate_game_form("Game", "vn17").is_some());
        assert!(validate_game_form("Game", "v").is_some());
    }

    #[test]
    fn add_builds_proton_game_with_defaults() {
        let game = apply_form(None, &form(), false, "new-id");
        assert_eq!(game.id, "new-id");
        assert_eq!(game.game_type, Some(GameType::WindowsExe));
        let wine = game.wine_settings.expect("wine settings");
        assert_eq!(wine.wine_type, Some(WineType::Proton));
        assert_eq!(wine.use_global_prefix, true);
        assert_eq!(game.play_time_minutes, 0);
    }

    #[test]
    fn edit_preserves_progress_and_uses_native() {
        let mut existing = apply_form(None, &form(), false, "g1");
        existing.play_time_minutes = 90;
        existing.is_finished = true;
        let mut edited = form();
        edited.platform = 0;
        edited.title = "Renamed".to_string().into();
        let game = apply_form(Some(&existing), &edited, false, "ignored");
        assert_eq!(game.id, "g1");
        assert_eq!(game.title, "Renamed");
        assert_eq!(game.play_time_minutes, 90);
        assert_eq!(game.is_finished, true);
        assert_eq!(game.game_type, Some(GameType::LinuxNative));
        assert_eq!(game.wine_settings, None);
    }

    #[test]
    fn platform_index_round_trips() {
        let proton = apply_form(None, &form(), false, "g1");
        assert_eq!(platform_index(&proton), 2);
        let mut native_form = form();
        native_form.platform = 0;
        let native = apply_form(None, &native_form, false, "g2");
        assert_eq!(platform_index(&native), 0);
    }
}
