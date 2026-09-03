use poketto_core::models::{AppSettings, Game};

pub fn plain_description(bbcode: &str) -> String {
    let mut text = String::with_capacity(bbcode.len());
    let mut in_tag = false;
    for c in bbcode.chars() {
        match c {
            '[' => in_tag = true,
            ']' => in_tag = false,
            _ if !in_tag => text.push(c),
            _ => {}
        }
    }
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn format_playtime(minutes: u64) -> String {
    if minutes >= 60 {
        format!("{}h {}m", minutes / 60, minutes % 60)
    } else {
        format!("{minutes}m")
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

pub struct DetailPayload {
    pub id: String,
    pub title: String,
    pub meta: String,
    pub playtime: String,
    pub synopsis: String,
    pub finished: bool,
    pub show_spoilers: bool,
    pub tags: Vec<(String, i32)>,
    pub characters: Vec<(String, String, i32)>,
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
        if let Some(rating) = detail.rating {
            meta_parts.push(format!("{rating:.2}"));
        }
        if let Some(minutes) = detail.length_minutes {
            meta_parts.push(format_playtime(minutes.max(0) as u64));
        }
        if let Some(description) = detail.description.as_deref() {
            let plain = plain_description(description);
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
    let characters = characters
        .iter()
        .take(12)
        .map(|character| {
            let role = character
                .vns
                .as_deref()
                .unwrap_or_default()
                .first()
                .map(|vn| vn.role.clone())
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
            (character.name.clone(), role, spoiler)
        })
        .collect();
    let mut playtime = format_playtime(game.play_time_minutes);
    if game.is_finished {
        playtime.push_str(" · Finished");
    }
    DetailPayload {
        id: game.id.clone(),
        title: game.title.clone(),
        meta: meta_parts.join(" · "),
        playtime: format!("Played {playtime}"),
        synopsis,
        finished: game.is_finished,
        show_spoilers: game.show_spoilers,
        tags,
        characters,
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
            plain_description("A [b]bold[/b] tale with [spoiler]secret[/spoiler]."),
            "A bold tale with secret."
        );
    }

    #[test]
    fn whitespace_collapses() {
        assert_eq!(plain_description("line one\nline   two"), "line one line two");
        assert_eq!(plain_description(""), "");
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
            game_type: None,
            wine_settings: None,
            rating: None,
        }
    }

    #[test]
    fn local_only_payload_has_fallbacks() {
        let payload = assemble_detail(&detail_game(), None, &[]);
        assert_eq!(payload.meta, "");
        assert_eq!(payload.playtime, "Played 2h 5m · Finished");
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
        assert_eq!(payload.meta, "2003-02-28 · 8.55 · 50h 0m");
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
            vec![("Meiya".to_string(), "main".to_string(), 2)]
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
}
