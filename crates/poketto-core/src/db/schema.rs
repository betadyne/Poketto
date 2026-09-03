use rusqlite::Connection;
use std::path::Path;
use std::time::Duration;

use super::error::{DbError, DbResult};

pub const SCHEMA_VERSION: i64 = 1;

const MIGRATION_V1: &str = "
CREATE TABLE IF NOT EXISTS games (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    path TEXT NOT NULL,
    vndb_id TEXT,
    cover_url TEXT,
    play_time_minutes INTEGER NOT NULL DEFAULT 0,
    is_finished INTEGER NOT NULL DEFAULT 0,
    last_played TEXT,
    is_hidden INTEGER NOT NULL DEFAULT 0,
    show_spoilers INTEGER NOT NULL DEFAULT 0,
    game_type TEXT,
    wine_settings TEXT
);
CREATE TABLE IF NOT EXISTS settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS tags (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE IF NOT EXISTS game_tags (
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (game_id, tag_id)
);
CREATE TABLE IF NOT EXISTS play_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    game_id TEXT NOT NULL REFERENCES games(id) ON DELETE CASCADE,
    started_at TEXT NOT NULL,
    play_date TEXT NOT NULL,
    minutes INTEGER NOT NULL CHECK (minutes > 0)
);
CREATE INDEX IF NOT EXISTS idx_play_sessions_game_date ON play_sessions(game_id, play_date);
CREATE INDEX IF NOT EXISTS idx_game_tags_tag ON game_tags(tag_id);
";

pub fn open(path: &Path) -> DbResult<Connection> {
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    let mode: String = conn.query_row("PRAGMA journal_mode", [], |row| row.get(0))?;
    if mode.to_lowercase() != "wal" {
        return Err(DbError::Config(format!("journal_mode is {mode}, expected WAL")));
    }
    configure(&conn)?;
    apply_migrations(&conn)?;
    Ok(conn)
}

pub fn open_in_memory() -> DbResult<Connection> {
    let conn = Connection::open_in_memory()?;
    configure(&conn)?;
    apply_migrations(&conn)?;
    Ok(conn)
}

fn configure(conn: &Connection) -> DbResult<()> {
    conn.busy_timeout(Duration::from_millis(5000))?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    Ok(())
}

fn apply_migrations(conn: &Connection) -> DbResult<()> {
    let version: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if version < 1 {
        conn.execute_batch(MIGRATION_V1)?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_database_reports_version_and_pragmas() {
        let conn = open_in_memory().expect("open");
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        assert_eq!(version, SCHEMA_VERSION);
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .expect("foreign_keys");
        assert_eq!(foreign_keys, 1);
    }

    #[test]
    fn migrations_are_idempotent() {
        let conn = open_in_memory().expect("open");
        apply_migrations(&conn).expect("re-apply");
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("version");
        assert_eq!(version, SCHEMA_VERSION);
    }
}
