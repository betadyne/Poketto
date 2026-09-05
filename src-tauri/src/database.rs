use rusqlite::{named_params, params, Connection, OptionalExtension, Row};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, DailyPlaytimeData, GameMetadata};

pub const VN_CACHE_PREFIX: &str = "vn:";
pub const CHAR_CACHE_PREFIX: &str = "chars:";

const GAMES_JSON_IMPORTED: &str = "games_json_imported";

const SCHEMA_SQL: &str = "
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    exe_path TEXT NOT NULL,
    prefix_path TEXT,
    runner TEXT,
    playtime_seconds INTEGER NOT NULL DEFAULT 0,
    last_played INTEGER,
    cover_path TEXT,
    vndb_id TEXT,
    steam_app_id TEXT,
    discord_status TEXT,
    created_at INTEGER NOT NULL,
    is_finished INTEGER NOT NULL DEFAULT 0,
    is_hidden INTEGER NOT NULL DEFAULT 0,
    show_spoilers INTEGER NOT NULL DEFAULT 0,
    game_type TEXT,
    wine_settings_json TEXT
);
CREATE TABLE IF NOT EXISTS playtime_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    start_time INTEGER NOT NULL,
    end_time INTEGER NOT NULL,
    duration_seconds INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS vndb_cache (
    id TEXT PRIMARY KEY,
    data_json TEXT NOT NULL,
    updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS schema_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const GAME_COLUMNS: &str = "id, title, exe_path, prefix_path, runner, playtime_seconds, last_played, cover_path, vndb_id, steam_app_id, discord_status, is_finished, is_hidden, show_spoilers, game_type, wine_settings_json";

pub struct AppDatabase {
    pub conn: Mutex<Connection>,
}

impl AppDatabase {
    pub fn open() -> AppResult<Self> {
        Self::open_at(&get_db_path())
    }

    pub fn open_at(path: &Path) -> AppResult<Self> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(path)?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.apply_pragmas()?;
        db.apply_schema()?;
        db.import_games_json_once()?;
        Ok(db)
    }

    pub fn open_in_memory() -> AppResult<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.apply_pragmas()?;
        db.apply_schema()?;
        Ok(db)
    }

    fn lock(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.conn
            .lock()
            .map_err(|e| AppError::Database(format!("database lock poisoned: {e}")))
    }

    fn apply_pragmas(&self) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL; PRAGMA foreign_keys = ON; PRAGMA synchronous = NORMAL;",
        )?;
        Ok(())
    }

    fn apply_schema(&self) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute_batch(SCHEMA_SQL)?;
        ensure_games_column(&conn, "steam_app_id", "TEXT")?;
        ensure_games_column(&conn, "discord_status", "TEXT")?;
        Ok(())
    }

    fn import_games_json_once(&self) -> AppResult<()> {
        let mut conn = self.lock()?;
        let imported: bool = conn
            .query_row(
                "SELECT value FROM schema_meta WHERE key = ?1",
                params![GAMES_JSON_IMPORTED],
                |row| row.get::<_, String>(0),
            )
            .optional()?
            .is_some();
        if imported {
            return Ok(());
        }

        let existing: i64 = conn.query_row("SELECT COUNT(*) FROM games", [], |row| row.get(0))?;
        let tx = conn.transaction()?;
        if existing == 0 {
            let games = read_legacy_games_json();
            let created_at = now_epoch();
            for game in &games {
                insert_game_row(&tx, game, created_at)?;
            }
            if !games.is_empty() {
                log::info!("Imported {} games from games.json", games.len());
            }
        }
        tx.execute(
            "INSERT OR REPLACE INTO schema_meta (key, value) VALUES (?1, '1')",
            params![GAMES_JSON_IMPORTED],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_all_games(&self) -> AppResult<Vec<GameMetadata>> {
        let conn = self.lock()?;
        let mut stmt = conn.prepare(&format!("SELECT {GAME_COLUMNS} FROM games ORDER BY rowid"))?;
        let rows = stmt
            .query_map([], game_from_row)?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn get_game_by_id(&self, id: &str) -> AppResult<Option<GameMetadata>> {
        let conn = self.lock()?;
        Ok(conn
            .query_row(
                &format!("SELECT {GAME_COLUMNS} FROM games WHERE id = ?1"),
                params![id],
                game_from_row,
            )
            .optional()?)
    }

    pub fn insert_game(&self, game: &GameMetadata) -> AppResult<()> {
        let conn = self.lock()?;
        insert_game_row(&conn, game, now_epoch())?;
        Ok(())
    }

    pub fn update_game(&self, game: &GameMetadata) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "UPDATE games SET title = :title, exe_path = :exe_path, prefix_path = :prefix_path, \
             runner = :runner, playtime_seconds = :playtime_seconds, last_played = :last_played, \
             cover_path = :cover_path, vndb_id = :vndb_id, steam_app_id = :steam_app_id, \
             discord_status = :discord_status, is_finished = :is_finished, \
             is_hidden = :is_hidden, show_spoilers = :show_spoilers, \
             game_type = :game_type, wine_settings_json = :wine_settings_json WHERE id = :id",
            named_params! {
                ":id": game.id,
                ":title": game.title,
                ":exe_path": game.path,
                ":prefix_path": wine_prefix(game),
                ":runner": wine_runner(game),
                ":playtime_seconds": playtime_minutes_to_seconds(game.play_time),
                ":last_played": last_played_to_epoch(game.last_played.as_deref()),
                ":cover_path": game.cover_url,
                ":vndb_id": game.vndb_id,
                ":steam_app_id": game.steam_app_id,
                ":discord_status": game.discord_status,
                ":is_finished": game.is_finished,
                ":is_hidden": game.is_hidden,
                ":show_spoilers": game.show_spoilers,
                ":game_type": game_type_json(game),
                ":wine_settings_json": wine_settings_json(game),
            },
        )?;
        Ok(())
    }

    pub fn delete_game(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM games WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_playtime(&self, game_id: &str, seconds: i64) -> AppResult<()> {
        if seconds <= 0 {
            return Ok(());
        }
        let mut conn = self.lock()?;
        let now = now_epoch();
        let tx = conn.transaction()?;
        let affected = tx.execute(
            "UPDATE games SET playtime_seconds = playtime_seconds + ?1, last_played = ?2 WHERE id = ?3",
            params![seconds, now, game_id],
        )?;
        if affected == 0 {
            return Ok(());
        }
        tx.execute(
            "INSERT INTO playtime_sessions (game_id, start_time, end_time, duration_seconds) VALUES (?1, ?2, ?3, ?4)",
            params![game_id, now - seconds, now, seconds],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_vndb_cache(&self, id: &str) -> AppResult<Option<String>> {
        let conn = self.lock()?;
        Ok(conn
            .query_row(
                "SELECT data_json FROM vndb_cache WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_vndb_cache(&self, id: &str, json: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute(
            "INSERT OR REPLACE INTO vndb_cache (id, data_json, updated_at) VALUES (?1, ?2, ?3)",
            params![id, json, now_epoch()],
        )?;
        Ok(())
    }

    pub fn delete_vndb_cache(&self, id: &str) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM vndb_cache WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn clear_vndb_cache(&self) -> AppResult<()> {
        let conn = self.lock()?;
        conn.execute("DELETE FROM vndb_cache", [])?;
        Ok(())
    }
}

fn ensure_games_column(conn: &Connection, name: &str, ddl: &str) -> AppResult<()> {
    let names: Vec<String> = conn
        .prepare("PRAGMA table_info(games)")?
        .query_map([], |row| row.get(1))?
        .filter_map(|name| name.ok())
        .collect();
    if !names.iter().any(|existing| existing == name) {
        conn.execute_batch(&format!("ALTER TABLE games ADD COLUMN {name} {ddl}"))?;
    }
    Ok(())
}

fn insert_game_row(
    conn: &Connection,
    game: &GameMetadata,
    created_at: i64,
) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO games (id, title, exe_path, prefix_path, runner, playtime_seconds, \
         last_played, cover_path, vndb_id, steam_app_id, discord_status, created_at, \
         is_finished, is_hidden, show_spoilers, game_type, wine_settings_json) \
         VALUES (:id, :title, :exe_path, :prefix_path, :runner, :playtime_seconds, \
         :last_played, :cover_path, :vndb_id, :steam_app_id, :discord_status, :created_at, \
         :is_finished, :is_hidden, :show_spoilers, :game_type, :wine_settings_json)",
        named_params! {
            ":id": game.id,
            ":title": game.title,
            ":exe_path": game.path,
            ":prefix_path": wine_prefix(game),
            ":runner": wine_runner(game),
            ":playtime_seconds": playtime_minutes_to_seconds(game.play_time),
            ":last_played": last_played_to_epoch(game.last_played.as_deref()),
            ":cover_path": game.cover_url,
            ":vndb_id": game.vndb_id,
            ":steam_app_id": game.steam_app_id,
            ":discord_status": game.discord_status,
            ":created_at": created_at,
            ":is_finished": game.is_finished,
            ":is_hidden": game.is_hidden,
            ":show_spoilers": game.show_spoilers,
            ":game_type": game_type_json(game),
            ":wine_settings_json": wine_settings_json(game),
        },
    )?;
    Ok(())
}

fn wine_prefix(game: &GameMetadata) -> Option<String> {
    game.wine_settings
        .as_ref()
        .and_then(|w| w.wine_prefix.clone())
}

fn wine_runner(game: &GameMetadata) -> Option<String> {
    game.wine_settings
        .as_ref()
        .and_then(|w| w.wine_version.clone())
}

fn game_type_json(game: &GameMetadata) -> Option<String> {
    game.game_type
        .as_ref()
        .and_then(|t| serde_json::to_string(t).ok())
}

fn wine_settings_json(game: &GameMetadata) -> Option<String> {
    game.wine_settings
        .as_ref()
        .and_then(|w| serde_json::to_string(w).ok())
}

fn game_from_row(row: &Row) -> rusqlite::Result<GameMetadata> {
    let playtime_seconds: i64 = row.get("playtime_seconds")?;
    let last_played_epoch: Option<i64> = row.get("last_played")?;
    let game_type_json: Option<String> = row.get("game_type")?;
    let wine_settings_json: Option<String> = row.get("wine_settings_json")?;
    Ok(GameMetadata {
        id: row.get("id")?,
        title: row.get("title")?,
        path: row.get("exe_path")?,
        vndb_id: row.get("vndb_id")?,
        steam_app_id: row.get("steam_app_id")?,
        discord_status: row.get("discord_status")?,
        cover_url: row.get("cover_path")?,
        play_time: seconds_to_playtime_minutes(playtime_seconds),
        is_finished: row.get::<_, i64>("is_finished")? != 0,
        last_played: last_played_epoch.map(epoch_to_rfc3339),
        is_hidden: row.get::<_, i64>("is_hidden")? != 0,
        show_spoilers: row.get::<_, i64>("show_spoilers")? != 0,
        game_type: game_type_json.and_then(|s| serde_json::from_str(&s).ok()),
        wine_settings: wine_settings_json.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

fn playtime_minutes_to_seconds(minutes: u64) -> i64 {
    minutes.saturating_mul(60).min(i64::MAX as u64) as i64
}

fn seconds_to_playtime_minutes(seconds: i64) -> u64 {
    seconds.max(0) as u64 / 60
}

fn last_played_to_epoch(value: Option<&str>) -> Option<i64> {
    value
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.timestamp())
}

fn epoch_to_rfc3339(epoch: i64) -> String {
    chrono::DateTime::from_timestamp(epoch, 0)
        .map(|dt| dt.with_timezone(&chrono::Local).to_rfc3339())
        .unwrap_or_default()
}

fn now_epoch() -> i64 {
    chrono::Utc::now().timestamp()
}

fn read_legacy_games_json() -> Vec<GameMetadata> {
    let path = get_data_path();
    if !path.exists() {
        return Vec::new();
    }
    match fs::read_to_string(&path) {
        Ok(content) => match serde_json::from_str::<Vec<GameMetadata>>(&content) {
            Ok(games) => games,
            Err(e) => {
                log::error!("Failed to parse games.json for import: {e}");
                let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
                let backup_path = get_data_dir().join(format!("games.json.corrupted.{timestamp}"));
                if let Err(backup_err) = fs::copy(&path, &backup_path) {
                    log::error!("Failed to backup corrupted games.json: {backup_err}");
                }
                Vec::new()
            }
        },
        Err(e) => {
            log::error!("Failed to read games.json for import: {e}");
            Vec::new()
        }
    }
}

pub fn get_data_dir() -> PathBuf {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("Poketto");
    fs::create_dir_all(&data_dir).ok();
    data_dir
}

pub fn get_data_path() -> PathBuf {
    get_data_dir().join("games.json")
}

pub fn get_db_path() -> PathBuf {
    get_data_dir().join("poketto.db")
}

pub fn get_settings_path() -> PathBuf {
    get_data_dir().join("settings.json")
}

pub fn get_daily_playtime_path() -> PathBuf {
    get_data_dir().join("daily_playtime.json")
}

fn atomic_write(path: &Path, content: &str) -> AppResult<()> {
    let tmp_path = path.with_extension("json.tmp");
    let file = fs::File::create(&tmp_path)?;
    {
        let mut writer = std::io::BufWriter::new(&file);
        writer.write_all(content.as_bytes())?;
        writer.flush()?;
    }
    file.sync_all()?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn save_settings(settings: &AppSettings) -> AppResult<()> {
    let path = get_settings_path();
    let json = serde_json::to_string_pretty(settings)?;
    atomic_write(&path, &json)
}

pub fn load_settings() -> AppSettings {
    let path = get_settings_path();
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        AppSettings::default()
    }
}

pub fn load_daily_playtime() -> DailyPlaytimeData {
    let path = get_daily_playtime_path();
    if path.exists() {
        fs::read_to_string(path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    } else {
        DailyPlaytimeData::default()
    }
}

pub fn save_daily_playtime(data: &DailyPlaytimeData) -> AppResult<()> {
    let path = get_daily_playtime_path();
    let json = serde_json::to_string_pretty(data)?;
    atomic_write(&path, &json)
}

pub fn record_daily_playtime(game_id: &str, minutes: u64) {
    if minutes == 0 {
        return;
    }

    let date_str = chrono::Local::now().format("%Y-%m-%d").to_string();
    let mut data = load_daily_playtime();

    let game_data = data.games.entry(game_id.to_string()).or_default();
    let current = game_data.entry(date_str).or_insert(0);
    *current += minutes;

    if let Err(e) = save_daily_playtime(&data) {
        log::error!("Failed to save daily playtime: {e}");
    }
}

pub fn create_http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .pool_idle_timeout(Duration::from_secs(90))
        .user_agent(format!("Poketto/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap_or_else(|e| {
            log::warn!("Failed to build HTTP client with custom config: {e}");
            reqwest::Client::new()
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{GameType, WineSettings, WineType};

    fn sample_game(id: &str) -> GameMetadata {
        GameMetadata {
            id: id.to_string(),
            title: format!("Game {id}"),
            path: format!("/games/{id}.exe"),
            vndb_id: Some("v42".to_string()),
            steam_app_id: Some("412830".to_string()),
            discord_status: Some("Shiro route".to_string()),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            play_time: 125,
            is_finished: true,
            last_played: Some("2026-09-01T12:00:00+07:00".to_string()),
            is_hidden: true,
            show_spoilers: false,
            game_type: Some(GameType::WindowsExe),
            wine_settings: Some(WineSettings {
                use_global_prefix: false,
                wine_prefix: Some("/prefix".to_string()),
                wine_version: Some("wine-9.0".to_string()),
                wine_type: Some(WineType::Wine),
                use_steam_runtime: true,
                env_vars: [("WINEDEBUG".to_string(), "-all".to_string())]
                    .into_iter()
                    .collect(),
            }),
        }
    }

    fn assert_game_matches(expected: &GameMetadata, actual: &GameMetadata) {
        assert_eq!(actual.id, expected.id);
        assert_eq!(actual.title, expected.title);
        assert_eq!(actual.path, expected.path);
        assert_eq!(actual.vndb_id, expected.vndb_id);
        assert_eq!(actual.cover_url, expected.cover_url);
        assert_eq!(actual.steam_app_id, expected.steam_app_id);
        assert_eq!(actual.discord_status, expected.discord_status);
        assert_eq!(actual.play_time, expected.play_time);
        assert_eq!(actual.is_finished, expected.is_finished);
        assert_eq!(actual.is_hidden, expected.is_hidden);
        assert_eq!(actual.show_spoilers, expected.show_spoilers);
        assert_eq!(actual.game_type, expected.game_type);
        assert_eq!(
            actual
                .last_played
                .as_deref()
                .map(|s| last_played_to_epoch(Some(s))),
            expected
                .last_played
                .as_deref()
                .map(|s| last_played_to_epoch(Some(s)))
        );
        let actual_wine = actual.wine_settings.as_ref().map(|w| {
            (
                w.use_global_prefix,
                w.wine_prefix.clone(),
                w.wine_version.clone(),
                w.wine_type.clone(),
                w.use_steam_runtime,
                w.env_vars.clone(),
            )
        });
        let expected_wine = expected.wine_settings.as_ref().map(|w| {
            (
                w.use_global_prefix,
                w.wine_prefix.clone(),
                w.wine_version.clone(),
                w.wine_type.clone(),
                w.use_steam_runtime,
                w.env_vars.clone(),
            )
        });
        assert_eq!(actual_wine, expected_wine);
    }

    #[test]
    fn test_game_crud_round_trip() {
        let db = AppDatabase::open_in_memory().expect("in-memory database");
        let game = sample_game("game-1");

        assert!(db.get_game_by_id("game-1").unwrap().is_none());

        db.insert_game(&game).unwrap();
        let loaded = db.get_game_by_id("game-1").unwrap().expect("game exists");
        assert_game_matches(&game, &loaded);

        let all = db.get_all_games().unwrap();
        assert_eq!(all.len(), 1);
        assert_game_matches(&game, &all[0]);

        let mut updated = game.clone();
        updated.title = "Renamed".to_string();
        updated.is_hidden = false;
        db.update_game(&updated).unwrap();
        let reloaded = db.get_game_by_id("game-1").unwrap().expect("game exists");
        assert_game_matches(&updated, &reloaded);

        db.delete_game("game-1").unwrap();
        assert!(db.get_game_by_id("game-1").unwrap().is_none());
        assert!(db.get_all_games().unwrap().is_empty());
    }

    #[test]
    fn test_playtime_minutes_survive_storage() {
        let db = AppDatabase::open_in_memory().expect("in-memory database");
        let game = sample_game("game-2");
        db.insert_game(&game).unwrap();

        db.add_playtime("game-2", 3600).unwrap();
        let loaded = db.get_game_by_id("game-2").unwrap().expect("game exists");
        assert_eq!(loaded.play_time, 125 + 60);
        assert!(loaded.last_played.is_some());

        let conn = db.lock().unwrap();
        let seconds: i64 = conn
            .query_row(
                "SELECT playtime_seconds FROM games WHERE id = 'game-2'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(seconds, 125 * 60 + 3600);
        let sessions: i64 = conn
            .query_row("SELECT COUNT(*) FROM playtime_sessions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(sessions, 1);
    }

    #[test]
    fn test_sub_minute_playtime_stamps_last_played() {
        let db = AppDatabase::open_in_memory().expect("in-memory database");
        db.insert_game(&sample_game("game-4")).unwrap();
        db.add_playtime("game-4", 45).unwrap();
        let loaded = db.get_game_by_id("game-4").unwrap().expect("game exists");
        assert!(loaded.last_played.is_some());
        assert_eq!(loaded.play_time, 125);
    }

    #[test]
    fn test_delete_cascades_playtime_sessions() {
        let db = AppDatabase::open_in_memory().expect("in-memory database");
        db.insert_game(&sample_game("game-3")).unwrap();
        db.add_playtime("game-3", 600).unwrap();
        db.delete_game("game-3").unwrap();

        let conn = db.lock().unwrap();
        let sessions: i64 = conn
            .query_row("SELECT COUNT(*) FROM playtime_sessions", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(sessions, 0);
    }

    #[test]
    fn test_vndb_cache_round_trip() {
        let db = AppDatabase::open_in_memory().expect("in-memory database");
        assert_eq!(db.get_vndb_cache("vn:v1").unwrap(), None);

        db.set_vndb_cache("vn:v1", "{\"id\":\"v1\"}").unwrap();
        assert_eq!(
            db.get_vndb_cache("vn:v1").unwrap(),
            Some("{\"id\":\"v1\"}".to_string())
        );

        db.set_vndb_cache("vn:v1", "{\"id\":\"v1b\"}").unwrap();
        assert_eq!(
            db.get_vndb_cache("vn:v1").unwrap(),
            Some("{\"id\":\"v1b\"}".to_string())
        );

        db.delete_vndb_cache("vn:v1").unwrap();
        assert_eq!(db.get_vndb_cache("vn:v1").unwrap(), None);

        db.set_vndb_cache("vn:v1", "a").unwrap();
        db.set_vndb_cache("chars:v1", "b").unwrap();
        db.clear_vndb_cache().unwrap();
        assert_eq!(db.get_vndb_cache("vn:v1").unwrap(), None);
        assert_eq!(db.get_vndb_cache("chars:v1").unwrap(), None);
    }

    #[test]
    fn test_steam_column_migrates_existing_database() {
        let path = std::env::temp_dir().join(format!(
            "poketto-migration-test-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        {
            let conn = Connection::open(&path).expect("legacy database");
            conn.execute_batch(
                "CREATE TABLE games (
                    id TEXT PRIMARY KEY,
                    title TEXT NOT NULL,
                    exe_path TEXT NOT NULL,
                    prefix_path TEXT,
                    runner TEXT,
                    playtime_seconds INTEGER NOT NULL DEFAULT 0,
                    last_played INTEGER,
                    cover_path TEXT,
                    vndb_id TEXT,
                    created_at INTEGER NOT NULL,
                    is_finished INTEGER NOT NULL DEFAULT 0,
                    is_hidden INTEGER NOT NULL DEFAULT 0,
                    show_spoilers INTEGER NOT NULL DEFAULT 0,
                    game_type TEXT,
                    wine_settings_json TEXT
                );
                INSERT INTO games (id, title, exe_path, created_at)
                VALUES ('mig-1', 'Legacy', '/games/legacy.exe', 0);",
            )
            .expect("legacy seed");
        }

        let db = AppDatabase::open_at(&path).expect("migrated database");
        let game = db
            .get_game_by_id("mig-1")
            .expect("read migrated row")
            .expect("game exists");
        assert_eq!(game.steam_app_id, None);

        let mut updated = game.clone();
        updated.steam_app_id = Some("412830".to_string());
        db.update_game(&updated).expect("update migrated row");
        let reloaded = db
            .get_game_by_id("mig-1")
            .expect("reread migrated row")
            .expect("game exists");
        assert_eq!(reloaded.steam_app_id, Some("412830".to_string()));

        let db = AppDatabase::open_at(&path).expect("reopened database");
        let reopened = db
            .get_game_by_id("mig-1")
            .expect("reread reopened row")
            .expect("game exists");
        assert_eq!(reopened.steam_app_id, Some("412830".to_string()));

        std::fs::remove_file(&path).ok();
    }
}
