use chrono::NaiveDate;
use poketto_core::models::{AppSettings, Game};

use crate::{DetailCharacter, DetailTag};

pub fn format_playtime(minutes: u64) -> String {
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else {
        format!("{minutes}m")
    }
}

pub fn vn_length_label(length: Option<i32>, minutes: Option<i32>) -> Option<String> {
    let band = if let Some(minutes) = minutes {
        match minutes.max(0) {
            0..=119 => 0,
            120..=599 => 1,
            600..=1799 => 2,
            1800..=3000 => 3,
            _ => 4,
        }
    } else {
        match length {
            Some(1) => 0,
            Some(2) => 1,
            Some(3) => 2,
            Some(4) => 3,
            Some(5) => 4,
            _ => return None,
        }
    };
    Some(
        match band {
            0 => "Very Short (~2h)",
            1 => "Short (~10h)",
            2 => "Medium (~30h)",
            3 => "Long (~50h)",
            _ => "Very Long (50h+)",
        }
        .to_string(),
    )
}

pub fn character_role_label(role: &str) -> &str {
    match role {
        "main" => "Protagonist",
        "primary" => "Main Characters",
        "side" => "Side Characters",
        "appears" => "Makes an Appearance",
        _ => role,
    }
}

pub fn release_status_label(devstatus: Option<i32>) -> Option<&'static str> {
    match devstatus {
        Some(0) => Some("Finished"),
        Some(1) => Some("Ongoing"),
        Some(2) => Some("Cancelled"),
        _ => None,
    }
}

pub fn relative_last_played(stamp: Option<&str>, today: NaiveDate) -> String {
    let date = stamp
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Local).date_naive());
    let Some(date) = date else {
        return "Never played".to_string();
    };
    let diff = (today - date).num_days();
    if diff <= 0 {
        "Today".to_string()
    } else if diff == 1 {
        "Yesterday".to_string()
    } else if diff < 7 {
        format!("{diff} days ago")
    } else if diff < 30 {
        format!("{} weeks ago", diff / 7)
    } else if diff < 365 {
        format!("{} months ago", diff / 30)
    } else {
        date.format("%b %-d, %Y").to_string()
    }
}

pub fn presence_buttons(game: &Game, settings: &AppSettings) -> Vec<(String, String)> {
    let mut buttons = Vec::new();
    if settings.discord_btn_vndb_game {
        if let Some(id) = &game.vndb_id {
            buttons.push(("View on VNDB".to_string(), format!("https://vndb.org/{id}")));
        }
    }
    if settings.discord_btn_vndb_profile && buttons.len() < 2 {
        if let Some(id) = &settings.vndb_user_id {
            buttons.push(("My VNDB Profile".to_string(), format!("https://vndb.org/{id}")));
        }
    }
    if settings.discord_btn_github && buttons.len() < 2 {
        buttons.push((
            "GitHub".to_string(),
            "https://github.com/betadyne/Poketto".to_string(),
        ));
    }
    buttons
}

#[derive(Clone, Default)]
pub struct SpoilerStore {
    pub tags: Vec<(String, i32)>,
    pub characters: Vec<(String, String, String, i32)>,
    pub avatars: Vec<(String, String)>,
}

pub fn visible_tags(tags: &[(String, i32)], show_spoilers: bool) -> Vec<DetailTag> {
    tags.iter()
        .filter(|(_, spoiler)| show_spoilers || *spoiler == 0)
        .map(|(name, spoiler)| DetailTag {
            name: name.clone().into(),
            spoiler: *spoiler,
            is_spoiler: *spoiler != 0,
        })
        .collect()
}

pub fn visible_characters(
    characters: &[(String, String, String, i32)],
    show_spoilers: bool,
) -> Vec<DetailCharacter> {
    characters
        .iter()
        .filter(|(_, _, _, spoiler)| show_spoilers || *spoiler < 2)
        .map(|(id, name, role, spoiler)| DetailCharacter {
            id: id.clone().into(),
            name: name.clone().into(),
            role: role.clone().into(),
            spoiler: *spoiler,
            avatar: slint::Image::default(),
        })
        .collect()
}

pub struct DetailPayload {
    pub id: String,
    pub title: String,
    pub meta: String,
    pub playtime: String,
    pub synopsis: String,
    pub finished: bool,
    pub show_spoilers: bool,
    pub user_status: i32,
    pub user_vote: i32,
    pub tags: Vec<(String, i32)>,
    pub characters: Vec<(String, String, String, i32)>,
    pub character_avatars: Vec<(String, String)>,
    pub cover_url: Option<String>,
    pub nsfw: bool,
}

pub fn assemble_detail(
    game: &Game,
    detail: Option<&poketto_core::models::VndbVnDetail>,
    characters: &[poketto_core::models::VndbCharacter],
) -> DetailPayload {
    let mut meta_parts = Vec::new();
    let mut synopsis = "(No synopsis available.)".to_string();
    let mut tags = Vec::new();
    let mut cover_url = game.cover_url.clone();
    if let Some(detail) = detail {
        if let Some(released) = detail.released.as_deref() {
            meta_parts.push(released.to_string());
        }
        if let Some(status) = release_status_label(detail.devstatus) {
            meta_parts.push(status.to_string());
        }
        if let Some(rating) = detail.rating {
            meta_parts.push(format!("{rating:.2}"));
        }
        if let Some(label) = vn_length_label(detail.length, detail.length_minutes) {
            meta_parts.push(label);
        }
        if let Some(description) = detail.description.as_deref() {
            let plain = poketto_core::vndb::clean_bbcode(description);
            if !plain.is_empty() {
                synopsis = plain;
            }
        }
        if let Some(detail_tags) = detail.tags.as_deref() {
            tags = detail_tags
                .iter()
                .take(8)
                .map(|tag| (tag.name.clone(), tag.spoiler))
                .collect();
        }
        if cover_url.is_none() {
            cover_url = detail.image.as_ref().map(|image| image.url.clone());
        }
    }
    let character_avatars: Vec<(String, String)> = characters
        .iter()
        .take(12)
        .filter_map(|character| {
            character
                .image
                .as_ref()
                .map(|image| (character.id.clone(), image.url.clone()))
        })
        .collect();
    let characters: Vec<(String, String, String, i32)> = characters
        .iter()
        .take(12)
        .map(|character| {
            let role = character
                .vns
                .as_deref()
                .unwrap_or_default()
                .first()
                .map(|vn| character_role_label(&vn.role).to_string())
                .unwrap_or_default();
            let spoiler = character
                .traits
                .as_deref()
                .unwrap_or_default()
                .iter()
                .map(|t| t.spoiler)
                .chain(
                    character
                        .vns
                        .as_deref()
                        .unwrap_or_default()
                        .iter()
                        .map(|vn| vn.spoiler),
                )
                .max()
                .unwrap_or(0);
            (character.id.clone(), character.name.clone(), role, spoiler)
        })
        .collect();
    let mut playtime = format_playtime(game.play_time_minutes);
    if game.is_finished {
        playtime.push_str(" · Finished");
    }
    let relative = relative_last_played(
        game.last_played.as_deref(),
        chrono::Local::now().date_naive(),
    );
    if relative == "Never played" {
        playtime.push_str(" · Never played");
    } else {
        playtime.push_str(&format!(" · Last played {relative}"));
    }
    DetailPayload {
        id: game.id.clone(),
        title: game.title.clone(),
        meta: meta_parts.join(" · "),
        playtime: format!("Played {playtime}"),
        synopsis,
        finished: game.is_finished,
        show_spoilers: game.show_spoilers,
        user_status: game.user_status,
        user_vote: game.user_vote,
        tags,
        characters,
        character_avatars,
        cover_url,
        nsfw: detail
            .as_ref()
            .and_then(|detail| detail.image.as_ref())
            .is_some_and(|image| super::games::cover_nsfw(image.sexual, image.violence)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bbcode_markers_strip_but_text_stays() {
        assert_eq!(
            poketto_core::vndb::clean_bbcode("A [b]bold[/b] tale with [spoiler]secret[/spoiler]."),
            "A bold tale with secret."
        );
    }

    #[test]
    fn whitespace_collapses() {
        assert_eq!(
            poketto_core::vndb::clean_bbcode("line one\nline   two"),
            "line one line two"
        );
        assert_eq!(poketto_core::vndb::clean_bbcode(""), "");
    }

    #[test]
    fn playtime_formats_like_legacy() {
        assert_eq!(format_playtime(0), "0m");
        assert_eq!(format_playtime(45), "45m");
        assert_eq!(format_playtime(60), "1h 0m");
        assert_eq!(format_playtime(125), "2h 5m");
    }

    #[test]
    fn buttons_respect_settings_and_cap() {
        let game = Game {
            id: "g1".to_string(),
            title: "Test".to_string(),
            path: "/games/g1".to_string(),
            work_dir: None,
            vndb_id: Some("v17".to_string()),
            cover_url: None,
            play_time_minutes: 0,
            is_finished: false,
            last_played: None,
            is_hidden: false,
            show_spoilers: false,
            user_status: 0,
            user_vote: 0,
            game_type: None,
            wine_settings: None,
            rating: None,
        };
        let settings = AppSettings {
            vndb_user_id: Some("u1".to_string()),
            discord_btn_vndb_game: true,
            discord_btn_vndb_profile: true,
            discord_btn_github: true,
            ..AppSettings::default()
        };
        let buttons = presence_buttons(&game, &settings);
        assert_eq!(buttons.len(), 2);
        assert_eq!(buttons[0].0, "View on VNDB");
        assert_eq!(buttons[1].0, "My VNDB Profile");
    }

    #[test]
    fn buttons_empty_without_ids() {
        let mut game = Game {
            id: "g1".to_string(),
            title: "Test".to_string(),
            path: "/games/g1".to_string(),
            work_dir: None,
            vndb_id: None,
            cover_url: None,
            play_time_minutes: 0,
            is_finished: false,
            last_played: None,
            is_hidden: false,
            show_spoilers: false,
            user_status: 0,
            user_vote: 0,
            game_type: None,
            wine_settings: None,
            rating: None,
        };
        game.vndb_id = None;
        let settings = AppSettings::default();
        assert_eq!(presence_buttons(&game, &settings).len(), 0);
    }

    fn detail_game() -> Game {
        Game {
            id: "g1".to_string(),
            title: "Muv-Luv".to_string(),
            path: "/games/g1".to_string(),
            work_dir: None,
            vndb_id: Some("v17".to_string()),
            cover_url: None,
            play_time_minutes: 125,
            is_finished: true,
            last_played: None,
            is_hidden: false,
            show_spoilers: false,
            user_status: 0,
            user_vote: 0,
            game_type: None,
            wine_settings: None,
            rating: None,
        }
    }

    #[test]
    fn local_only_payload_has_fallbacks() {
        let payload = assemble_detail(&detail_game(), None, &[]);
        assert_eq!(payload.meta, "");
        assert_eq!(payload.playtime, "Played 2h 5m · Finished · Never played");
        assert_eq!(payload.synopsis, "(No synopsis available.)");
        assert_eq!(payload.tags.len(), 0);
        assert_eq!(payload.cover_url, None);
    }

    #[test]
    fn detail_payload_merges_vndb_data() {
        let json = r#"{"id": "v17", "title": "Muv-Luv", "description": "A [b]story[/b].", "released": "2003-02-28", "rating": 8.55, "length_minutes": 3000, "tags": [{"id": "g1", "name": "Drama", "rating": 8.0}], "developers": [{"id": "p1", "name": "Age"}], "image": {"url": "https://img.jpg"}}"#;
        let detail: poketto_core::models::VndbVnDetail =
            serde_json::from_str(json).expect("fixture");
        let payload = assemble_detail(&detail_game(), Some(&detail), &[]);
        assert_eq!(payload.meta, "2003-02-28 · 8.55 · Long (~50h)");
        assert_eq!(payload.synopsis, "A story.");
        assert_eq!(payload.tags, vec![("Drama".to_string(), 0)]);
        assert_eq!(payload.cover_url.as_deref(), Some("https://img.jpg"));
    }

    #[test]
    fn spoiler_levels_carry_max_trait_and_vn_flags() {
        let json = r#"{"id": "c1", "name": "Meiya", "vns": [{"id": "v17", "role": "main", "spoiler": 1}], "traits": [{"id": "t1", "name": "Twintails", "spoiler": 0}, {"id": "t2", "name": "Secret", "spoiler": 2}]}"#;
        let character: poketto_core::models::VndbCharacter =
            serde_json::from_str(json).expect("fixture");
        let payload = assemble_detail(&detail_game(), None, &[character]);
        assert_eq!(
            payload.characters,
            vec![("c1".to_string(), "Meiya".to_string(), "Protagonist".to_string(), 2)]
        );
    }

    #[test]
    fn nsfw_flag_follows_cover_scores() {
        let safe_json = r#"{"id": "v17", "title": "Safe", "image": {"url": "https://img.jpg", "sexual": 0.0, "violence": 0.0}}"#;
        let safe: poketto_core::models::VndbVnDetail =
            serde_json::from_str(safe_json).expect("fixture");
        assert_eq!(assemble_detail(&detail_game(), Some(&safe), &[]).nsfw, false);
        let spicy_json = r#"{"id": "v17", "title": "Spicy", "image": {"url": "https://img.jpg", "sexual": 0.0, "violence": 1.1}}"#;
        let spicy: poketto_core::models::VndbVnDetail =
            serde_json::from_str(spicy_json).expect("fixture");
        assert_eq!(assemble_detail(&detail_game(), Some(&spicy), &[]).nsfw, true);
    }

    #[test]
    fn vn_length_bands_follow_vndb_scale() {
        assert_eq!(
            vn_length_label(Some(1), Some(119)).as_deref(),
            Some("Very Short (~2h)")
        );
        assert_eq!(
            vn_length_label(None, Some(120)).as_deref(),
            Some("Short (~10h)")
        );
        assert_eq!(
            vn_length_label(None, Some(599)).as_deref(),
            Some("Short (~10h)")
        );
        assert_eq!(
            vn_length_label(None, Some(600)).as_deref(),
            Some("Medium (~30h)")
        );
        assert_eq!(
            vn_length_label(None, Some(1799)).as_deref(),
            Some("Medium (~30h)")
        );
        assert_eq!(
            vn_length_label(None, Some(1800)).as_deref(),
            Some("Long (~50h)")
        );
        assert_eq!(
            vn_length_label(None, Some(2999)).as_deref(),
            Some("Long (~50h)")
        );
        assert_eq!(
            vn_length_label(None, Some(3000)).as_deref(),
            Some("Long (~50h)")
        );
        assert_eq!(
            vn_length_label(None, Some(9000)).as_deref(),
            Some("Very Long (50h+)")
        );
    }

    #[test]
    fn vn_length_falls_back_to_enum() {
        assert_eq!(
            vn_length_label(Some(2), None).as_deref(),
            Some("Short (~10h)")
        );
        assert_eq!(
            vn_length_label(Some(5), None).as_deref(),
            Some("Very Long (50h+)")
        );
        assert_eq!(vn_length_label(None, None), None);
        assert_eq!(vn_length_label(Some(9), None), None);
    }

    #[test]
    fn character_roles_use_legacy_names() {
        assert_eq!(character_role_label("main"), "Protagonist");
        assert_eq!(character_role_label("primary"), "Main Characters");
        assert_eq!(character_role_label("side"), "Side Characters");
        assert_eq!(character_role_label("appears"), "Makes an Appearance");
        assert_eq!(character_role_label("cameo"), "cameo");
        assert_eq!(character_role_label(""), "");
    }

    #[test]
    fn release_status_labels_devstatus() {
        assert_eq!(release_status_label(Some(0)), Some("Finished"));
        assert_eq!(release_status_label(Some(1)), Some("Ongoing"));
        assert_eq!(release_status_label(Some(2)), Some("Cancelled"));
        assert_eq!(release_status_label(None), None);
        assert_eq!(release_status_label(Some(99)), None);
    }

    #[test]
    fn relative_last_played_follows_ladder() {
        let today = NaiveDate::from_ymd_opt(2024, 6, 15).expect("date");
        assert_eq!(
            relative_last_played(Some("2024-06-15T12:00:00Z"), today),
            "Today"
        );
        assert_eq!(
            relative_last_played(Some("2024-06-14T12:00:00Z"), today),
            "Yesterday"
        );
        assert_eq!(
            relative_last_played(Some("2024-06-13T12:00:00Z"), today),
            "2 days ago"
        );
        assert_eq!(
            relative_last_played(Some("2024-06-10T12:00:00Z"), today),
            "5 days ago"
        );
        assert_eq!(
            relative_last_played(Some("2024-06-06T12:00:00Z"), today),
            "1 weeks ago"
        );
        assert_eq!(
            relative_last_played(Some("2024-05-06T12:00:00Z"), today),
            "1 months ago"
        );
        assert_eq!(
            relative_last_played(Some("2023-05-11T12:00:00Z"), today),
            "May 11, 2023"
        );
        assert_eq!(relative_last_played(None, today), "Never played");
        assert_eq!(relative_last_played(Some("garbage"), today), "Never played");
        assert_eq!(
            relative_last_played(Some("2024-06-20T12:00:00Z"), today),
            "Today"
        );
    }

    #[test]
    fn character_avatar_urls_collected() {
        let json = r#"{"id": "c1", "name": "Meiya", "image": {"url": "https://img.jpg/c1.jpg"}, "vns": [{"id": "v17", "role": "main", "spoiler": 0}]}"#;
        let shown: poketto_core::models::VndbCharacter =
            serde_json::from_str(json).expect("fixture");
        let hidden_json = r#"{"id": "c2", "name": "No Face", "vns": []}"#;
        let hidden: poketto_core::models::VndbCharacter =
            serde_json::from_str(hidden_json).expect("fixture");
        let payload = assemble_detail(&detail_game(), None, &[shown, hidden]);
        assert_eq!(
            payload.character_avatars,
            vec![("c1".to_string(), "https://img.jpg/c1.jpg".to_string())]
        );
    }

    #[test]
    fn spoiler_tags_hidden_until_allowed() {
        let tags = vec![("Plot".to_string(), 0), ("Twist".to_string(), 2)];
        let hidden = visible_tags(&tags, false);
        assert_eq!(hidden.len(), 1);
        assert_eq!(hidden[0].name.as_str(), "Plot");
        assert!(!hidden[0].is_spoiler);
        let shown = visible_tags(&tags, true);
        assert_eq!(shown.len(), 2);
        assert_eq!(shown[1].spoiler, 2);
        assert!(shown[1].is_spoiler);
    }

    #[test]
    fn minor_spoiler_characters_stay_visible() {
        let characters = vec![
            ("c1".to_string(), "Meiya".to_string(), "Protagonist".to_string(), 0),
            ("c2".to_string(), "Ghost".to_string(), String::new(), 1),
            ("c3".to_string(), "Traitor".to_string(), "Side Characters".to_string(), 2),
        ];
        let hidden = visible_characters(&characters, false);
        assert_eq!(hidden.len(), 2);
        assert_eq!(hidden[0].id.as_str(), "c1");
        assert_eq!(hidden[1].id.as_str(), "c2");
        let shown = visible_characters(&characters, true);
        assert_eq!(shown.len(), 3);
        assert_eq!(shown[2].spoiler, 2);
    }
}
