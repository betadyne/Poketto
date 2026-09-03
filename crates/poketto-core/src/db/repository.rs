use rusqlite::{params, Connection, OptionalExtension, Row};
use std::collections::HashMap;

use super::error::{DbError, DbResult};
use crate::models::{AppSettings, Game, Tag, WineSettings};

const GAME_COLUMNS: &str = "id, title, path, vndb_id, cover_url, play_time_minutes, is_finished, last_played, is_hidden, show_spoilers, game_type, wine_settings, rating";

fn game_from_row(row: &Row) -> rusqlite::Result<Game> {
    let game_type: Option<String> = row.get(10)?;
    let wine_settings: Option<String> = row.get(11)?;
    let play_time_raw: i64 = row.get(5)?;
    let play_time_minutes = u64::try_from(play_time_raw).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Integer,
            Box::new(e),
        )
    })?;
    Ok(Game {
        id: row.get(0)?,
        title: row.get(1)?,
        path: row.get(2)?,
        vndb_id: row.get(3)?,
        cover_url: row.get(4)?,
        play_time_minutes,
        is_finished: row.get(6)?,
        last_played: row.get(7)?,
        is_hidden: row.get(8)?,
        show_spoilers: row.get(9)?,
        game_type: game_type.and_then(|value| value.parse().ok()),
        rating: row.get(12)?,
        wine_settings: wine_settings
            .map(|value| serde_json::from_str(&value))
            .transpose()
            .map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    11,
                    rusqlite::types::Type::Text,
                    Box::new(e),
                )
            })?,
    })
}

fn wine_blob(settings: &Option<WineSettings>) -> DbResult<Option<String>> {
    settings
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(DbError::from)
}

pub fn get_all_games(conn: &Connection) -> DbResult<Vec<Game>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {GAME_COLUMNS} FROM games ORDER BY title COLLATE NOCASE"
    ))?;
    let games = stmt
        .query_map([], game_from_row)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(games)
}

pub fn get_game(conn: &Connection, id: &str) -> DbResult<Option<Game>> {
    let mut stmt = conn.prepare(&format!("SELECT {GAME_COLUMNS} FROM games WHERE id = ?1"))?;
    let game = stmt.query_row(params![id], game_from_row).optional()?;
    Ok(game)
}

pub fn insert_game(conn: &Connection, game: &Game) -> DbResult<()> {
    let play_time = i64::try_from(game.play_time_minutes)
        .map_err(|_| DbError::OutOfRange(game.play_time_minutes))?;
    conn.execute(
        "INSERT INTO games (id, title, path, vndb_id, cover_url, play_time_minutes, is_finished, last_played, is_hidden, show_spoilers, game_type, wine_settings, rating)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)",
        params![
            game.id,
            game.title,
            game.path,
            game.vndb_id,
            game.cover_url,
            play_time,
            game.is_finished,
            game.last_played,
            game.is_hidden,
            game.show_spoilers,
            game.game_type.as_ref().map(|kind| kind.as_str()),
            wine_blob(&game.wine_settings)?,
            game.rating,
        ],
    )?;
    Ok(())
}

pub fn update_game(conn: &Connection, game: &Game) -> DbResult<()> {
    let play_time = i64::try_from(game.play_time_minutes)
        .map_err(|_| DbError::OutOfRange(game.play_time_minutes))?;
    let changed = conn.execute(
        "UPDATE games SET title = ?2, path = ?3, vndb_id = ?4, cover_url = ?5, play_time_minutes = ?6,
         is_finished = ?7, last_played = ?8, is_hidden = ?9, show_spoilers = ?10, game_type = ?11,
         wine_settings = ?12, rating = ?13 WHERE id = ?1",
        params![
            game.id,
            game.title,
            game.path,
            game.vndb_id,
            game.cover_url,
            play_time,
            game.is_finished,
            game.last_played,
            game.is_hidden,
            game.show_spoilers,
            game.game_type.as_ref().map(|kind| kind.as_str()),
            wine_blob(&game.wine_settings)?,
            game.rating,
        ],
    )?;
    if changed == 0 {
        return Err(DbError::GameNotFound(game.id.clone()));
    }
    Ok(())
}

pub fn delete_game(conn: &Connection, id: &str) -> DbResult<()> {
    let changed = conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
    if changed == 0 {
        return Err(DbError::GameNotFound(id.to_string()));
    }
    Ok(())
}

fn setting_pairs(settings: &AppSettings) -> Vec<(&'static str, Option<String>)> {
    let flag = |value: bool| Some(if value { "1".to_string() } else { "0".to_string() });
    vec![
        ("vndb_token", settings.vndb_token.clone()),
        ("vndb_user_id", settings.vndb_user_id.clone()),
        ("blur_nsfw", flag(settings.blur_nsfw)),
        ("discord_rpc_enabled", flag(settings.discord_rpc_enabled)),
        ("discord_btn_vndb_game", flag(settings.discord_btn_vndb_game)),
        (
            "discord_btn_vndb_profile",
            flag(settings.discord_btn_vndb_profile),
        ),
        ("discord_btn_github", flag(settings.discord_btn_github)),
        (
            "default_wine_prefix",
            settings.default_wine_prefix.clone(),
        ),
        (
            "default_wine_binary",
            settings.default_wine_binary.clone(),
        ),
        ("use_steam_runtime", flag(settings.use_steam_runtime)),
    ]
}

pub fn save_settings(conn: &Connection, settings: &AppSettings) -> DbResult<()> {
    for (key, value) in setting_pairs(settings) {
        match value {
            Some(value) => {
                conn.execute(
                    "INSERT INTO settings (key, value) VALUES (?1, ?2)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                    params![key, value],
                )?;
            }
            None => {
                conn.execute("DELETE FROM settings WHERE key = ?1", params![key])?;
            }
        }
    }
    Ok(())
}

pub fn load_settings(conn: &Connection) -> DbResult<AppSettings> {
    let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
    let pairs = stmt.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    let mut map = HashMap::new();
    for pair in pairs {
        let (key, value) = pair?;
        map.insert(key, value);
    }
    let text = |key: &str| map.get(key).cloned();
    let flag = |key: &str| map.get(key).is_some_and(|value| value == "1");
    let flag_default_true = |key: &str| map.get(key).is_none_or(|value| value == "1");
    Ok(AppSettings {
        vndb_token: text("vndb_token"),
        vndb_user_id: text("vndb_user_id"),
        blur_nsfw: flag("blur_nsfw"),
        discord_rpc_enabled: flag_default_true("discord_rpc_enabled"),
        discord_btn_vndb_game: flag_default_true("discord_btn_vndb_game"),
        discord_btn_vndb_profile: flag("discord_btn_vndb_profile"),
        discord_btn_github: flag("discord_btn_github"),
        default_wine_prefix: text("default_wine_prefix"),
        default_wine_binary: text("default_wine_binary"),
        use_steam_runtime: flag("use_steam_runtime"),
    })
}

pub fn record_play_session(
    conn: &Connection,
    game_id: &str,
    started_at: &str,
    play_date: &str,
    minutes: u64,
) -> DbResult<()> {
    if minutes == 0 {
        return Ok(());
    }
    conn.execute(
        "INSERT INTO play_sessions (game_id, started_at, play_date, minutes) VALUES (?1, ?2, ?3, ?4)",
        params![
            game_id,
            started_at,
            play_date,
            i64::try_from(minutes).map_err(|_| DbError::OutOfRange(minutes))?,
        ],
    )?;
    Ok(())
}

pub fn daily_playtime(conn: &Connection, game_id: &str) -> DbResult<HashMap<String, u64>> {
    let mut stmt = conn.prepare(
        "SELECT play_date, SUM(minutes) FROM play_sessions WHERE game_id = ?1 GROUP BY play_date",
    )?;
    let rows = stmt.query_map(params![game_id], |row| {
        Ok((
            row.get::<_, String>(0)?,
            u64::try_from(row.get::<_, i64>(1)?).map_err(|e| {
                rusqlite::Error::FromSqlConversionFailure(
                    1,
                    rusqlite::types::Type::Integer,
                    Box::new(e),
                )
            })?,
        ))
    })?;
    let mut totals = HashMap::new();
    for row in rows {
        let (date, minutes) = row?;
        totals.insert(date, minutes);
    }
    Ok(totals)
}

pub struct CacheEntry {
    pub value: String,
    pub fetched_at: i64,
}

pub fn cache_put(conn: &Connection, kind: &str, key: &str, value: &str, fetched_at: i64) -> DbResult<()> {
    conn.execute(
        "INSERT INTO vndb_cache (kind, key, value, fetched_at) VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(kind, key) DO UPDATE SET value = excluded.value, fetched_at = excluded.fetched_at",
        params![kind, key, value, fetched_at],
    )?;
    Ok(())
}

pub fn cache_get(conn: &Connection, kind: &str, key: &str) -> DbResult<Option<CacheEntry>> {
    let mut stmt =
        conn.prepare("SELECT value, fetched_at FROM vndb_cache WHERE kind = ?1 AND key = ?2")?;
    let entry = stmt
        .query_row(params![kind, key], |row| {
            Ok(CacheEntry {
                value: row.get(0)?,
                fetched_at: row.get(1)?,
            })
        })
        .optional()?;
    Ok(entry)
}

pub fn cache_remove(conn: &Connection, key: &str) -> DbResult<()> {
    conn.execute("DELETE FROM vndb_cache WHERE key = ?1", params![key])?;
    Ok(())
}

pub fn cache_clear(conn: &Connection) -> DbResult<()> {
    conn.execute("DELETE FROM vndb_cache", [])?;
    Ok(())
}

pub fn add_tag(conn: &Connection, id: &str, name: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO tags (id, name) VALUES (?1, ?2)",
        params![id, name],
    )?;
    Ok(())
}

pub fn tag_game(conn: &Connection, game_id: &str, tag_id: &str) -> DbResult<()> {
    conn.execute(
        "INSERT INTO game_tags (game_id, tag_id) VALUES (?1, ?2)",
        params![game_id, tag_id],
    )?;
    Ok(())
}

pub fn untag_game(conn: &Connection, game_id: &str, tag_id: &str) -> DbResult<()> {
    conn.execute(
        "DELETE FROM game_tags WHERE game_id = ?1 AND tag_id = ?2",
        params![game_id, tag_id],
    )?;
    Ok(())
}
pub fn tags_for_game(conn: &Connection, game_id: &str) -> DbResult<Vec<Tag>> {
    let mut stmt = conn.prepare(
        "SELECT tags.id, tags.name FROM tags
         INNER JOIN game_tags ON tags.id = game_tags.tag_id
         WHERE game_tags.game_id = ?1 ORDER BY tags.name COLLATE NOCASE",
    )?;
    let tags = stmt
        .query_map(params![game_id], |row| {
            Ok(Tag {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::schema::open_in_memory;
    use crate::models::{GameType, WineSettings};

    fn setup() -> Connection {
        open_in_memory().expect("in-memory database opens")
    }

    fn sample_game(id: &str, title: &str) -> Game {
        Game {
            id: id.to_string(),
            title: title.to_string(),
            path: format!("/games/{id}"),
            vndb_id: Some("v17".to_string()),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            play_time_minutes: 42,
            is_finished: false,
            last_played: Some("2026-09-03T00:00:00+00:00".to_string()),
            is_hidden: false,
            show_spoilers: true,
            game_type: Some(GameType::WindowsExe),
            wine_settings: Some(WineSettings {
                use_global_prefix: true,
                wine_version: Some("Proton-GE".to_string()),
                ..WineSettings::default()
            }),
            rating: Some(8.55),
        }
    }

    #[test]
    fn insert_and_get_round_trip() {
        let conn = setup();
        let game = sample_game("g1", "Round Trip");
        insert_game(&conn, &game).expect("insert");
        let loaded = get_game(&conn, "g1").expect("get").expect("found");
        assert_eq!(loaded.title, "Round Trip");
        assert_eq!(loaded.play_time_minutes, 42);
        assert_eq!(loaded.rating, Some(8.55));
        assert_eq!(loaded.game_type, Some(GameType::WindowsExe));
        let wine = loaded.wine_settings.expect("wine settings");
        assert_eq!(wine.use_global_prefix, true);
        assert_eq!(wine.wine_version.as_deref(), Some("Proton-GE"));
    }

    #[test]
    fn get_all_orders_by_title() {
        let conn = setup();
        insert_game(&conn, &sample_game("b", "Zulu")).expect("insert");
        insert_game(&conn, &sample_game("a", "Alpha")).expect("insert");
        let games = get_all_games(&conn).expect("list");
        assert_eq!(games.len(), 2);
        assert_eq!(games[0].title, "Alpha");
        assert_eq!(games[1].title, "Zulu");
    }

    #[test]
    fn get_missing_returns_none() {
        let conn = setup();
        assert_eq!(get_game(&conn, "nope").expect("get").is_none(), true);
    }

    #[test]
    fn duplicate_insert_fails() {
        let conn = setup();
        insert_game(&conn, &sample_game("g1", "First")).expect("insert");
        assert_eq!(insert_game(&conn, &sample_game("g1", "Second")).is_err(), true);
    }

    #[test]
    fn update_persists_and_missing_errors() {
        let conn = setup();
        let mut game = sample_game("g1", "Before");
        insert_game(&conn, &game).expect("insert");
        game.title = "After".to_string();
        game.is_finished = true;
        game.wine_settings = None;
        update_game(&conn, &game).expect("update");
        let loaded = get_game(&conn, "g1").expect("get").expect("found");
        assert_eq!(loaded.title, "After");
        assert_eq!(loaded.is_finished, true);
        assert_eq!(loaded.wine_settings, None);
        let missing = sample_game("ghost", "Ghost");
        assert!(matches!(
            update_game(&conn, &missing),
            Err(DbError::GameNotFound(_))
        ));
    }

    #[test]
    fn delete_removes_and_missing_errors() {
        let conn = setup();
        insert_game(&conn, &sample_game("g1", "Gone")).expect("insert");
        delete_game(&conn, "g1").expect("delete");
        assert_eq!(get_game(&conn, "g1").expect("get").is_none(), true);
        assert!(matches!(
            delete_game(&conn, "g1"),
            Err(DbError::GameNotFound(_))
        ));
    }

    #[test]
    fn delete_cascades_tags_and_sessions() {
        let conn = setup();
        insert_game(&conn, &sample_game("g1", "Cascade")).expect("insert");
        add_tag(&conn, "t1", "Favorites").expect("tag");
        tag_game(&conn, "g1", "t1").expect("link");
        record_play_session(&conn, "g1", "2026-09-03T10:00:00+00:00", "2026-09-03", 30)
            .expect("session");
        delete_game(&conn, "g1").expect("delete");
        assert_eq!(tags_for_game(&conn, "g1").expect("tags").len(), 0);
        assert_eq!(daily_playtime(&conn, "g1").expect("playtime").len(), 0);
    }

    #[test]
    fn settings_round_trip() {
        let conn = setup();
        let settings = AppSettings {
            vndb_token: Some("token-123".to_string()),
            blur_nsfw: true,
            discord_rpc_enabled: false,
            discord_btn_vndb_game: false,
            discord_btn_github: true,
            default_wine_binary: Some("/usr/bin/wine".to_string()),
            ..AppSettings::default()
        };
        save_settings(&conn, &settings).expect("save");
        let loaded = load_settings(&conn).expect("load");
        assert_eq!(loaded.vndb_token.as_deref(), Some("token-123"));
        assert_eq!(loaded.blur_nsfw, true);
        assert_eq!(loaded.discord_rpc_enabled, false);
        assert_eq!(loaded.discord_btn_vndb_game, false);
        assert_eq!(loaded.discord_btn_github, true);
        assert_eq!(loaded.vndb_user_id, None);
        assert_eq!(
            loaded.default_wine_binary.as_deref(),
            Some("/usr/bin/wine")
        );
    }

    #[test]
    fn settings_default_when_empty() {
        let conn = setup();
        let loaded = load_settings(&conn).expect("load");
        assert_eq!(loaded.discord_rpc_enabled, true);
        assert_eq!(loaded.discord_btn_vndb_game, true);
        assert_eq!(loaded.blur_nsfw, false);
    }

    #[test]
    fn play_sessions_aggregate_by_day() {
        let conn = setup();
        insert_game(&conn, &sample_game("g1", "Played")).expect("insert");
        record_play_session(&conn, "g1", "2026-09-03T10:00:00+00:00", "2026-09-03", 20)
            .expect("first");
        record_play_session(&conn, "g1", "2026-09-03T20:00:00+00:00", "2026-09-03", 25)
            .expect("second");
        record_play_session(&conn, "g1", "2026-09-02T10:00:00+00:00", "2026-09-02", 0)
            .expect("zero ignored");
        let totals = daily_playtime(&conn, "g1").expect("totals");
        assert_eq!(totals.get("2026-09-03"), Some(&45));
        assert_eq!(totals.contains_key("2026-09-02"), false);
    }

    #[test]
    fn tag_flow_links_and_unlinks() {
        let conn = setup();
        insert_game(&conn, &sample_game("g1", "Tagged")).expect("insert");
        add_tag(&conn, "t1", "Favorites").expect("tag");
        tag_game(&conn, "g1", "t1").expect("link");
        let tags = tags_for_game(&conn, "g1").expect("list");
        assert_eq!(tags.len(), 1);
        assert_eq!(tags[0].name, "Favorites");
        untag_game(&conn, "g1", "t1").expect("unlink");
        assert_eq!(tags_for_game(&conn, "g1").expect("list").len(), 0);
        untag_game(&conn, "g1", "t1").expect("unlink stays idempotent");
    }

    #[test]
    fn tag_missing_game_violates_foreign_key() {
        let conn = setup();
        add_tag(&conn, "t1", "Favorites").expect("tag");
        assert_eq!(tag_game(&conn, "ghost", "t1").is_err(), true);
    }
}
