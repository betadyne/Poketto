use rusqlite::Connection;
use std::time::{SystemTime, UNIX_EPOCH};

use super::client::VndbClient;
use super::error::{VndbError, VndbResult};
use crate::db;
use crate::models::{VndbCharacter, VndbVnDetail};

pub const KIND_DETAIL: &str = "vn_detail";
pub const KIND_CHARACTERS: &str = "characters";

fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}

fn db_error(e: db::DbError) -> VndbError {
    VndbError::Cache(e.to_string())
}

fn load_cached<T: serde::de::DeserializeOwned>(
    conn: &Connection,
    kind: &str,
    key: &str,
) -> VndbResult<Option<T>> {
    match db::cache_get(conn, kind, key).map_err(db_error)? {
        Some(entry) => serde_json::from_str(&entry.value)
            .map(Some)
            .map_err(|e| VndbError::Cache(e.to_string())),
        None => Ok(None),
    }
}

fn store_cached<T: serde::Serialize>(
    conn: &Connection,
    kind: &str,
    key: &str,
    value: &T,
) -> VndbResult<()> {
    let json = serde_json::to_string(value).map_err(|e| VndbError::Cache(e.to_string()))?;
    db::cache_put(conn, kind, key, &json, now_unix()).map_err(db_error)
}

pub async fn detail_cached(
    conn: &Connection,
    client: &VndbClient,
    vndb_id: &str,
    force_refresh: bool,
) -> VndbResult<VndbVnDetail> {
    if !force_refresh {
        if let Some(cached) = load_cached(conn, KIND_DETAIL, vndb_id)? {
            return Ok(cached);
        }
    }
    let detail = client.detail(vndb_id).await?;
    store_cached(conn, KIND_DETAIL, vndb_id, &detail)?;
    Ok(detail)
}

pub async fn characters_cached(
    conn: &Connection,
    client: &VndbClient,
    vndb_id: &str,
    force_refresh: bool,
) -> VndbResult<Vec<VndbCharacter>> {
    if !force_refresh {
        if let Some(cached) = load_cached(conn, KIND_CHARACTERS, vndb_id)? {
            return Ok(cached);
        }
    }
    let characters = client.characters(vndb_id).await?;
    store_cached(conn, KIND_CHARACTERS, vndb_id, &characters)?;
    Ok(characters)
}

pub fn invalidate(conn: &Connection, vndb_id: &str) -> VndbResult<()> {
    db::cache_remove(conn, vndb_id).map_err(db_error)
}

pub fn clear(conn: &Connection) -> VndbResult<()> {
    db::cache_clear(conn).map_err(db_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::open_in_memory;

    const DETAIL_JSON: &str = r#"{"id": "v17", "title": "Muv-Luv", "description": "A story."}"#;
    const CHARACTERS_JSON: &str = r#"[{"id": "c1", "name": "Meiya"}]"#;

    fn seeded() -> Connection {
        let conn = open_in_memory().expect("open");
        db::cache_put(&conn, KIND_DETAIL, "v17", DETAIL_JSON, 0).expect("seed detail");
        db::cache_put(&conn, KIND_CHARACTERS, "v17", CHARACTERS_JSON, 0).expect("seed chars");
        conn
    }

    fn offline_client() -> VndbClient {
        VndbClient::new().with_base_url("http://127.0.0.1:9")
    }

    #[tokio::test]
    async fn seeded_detail_serves_without_network() {
        let conn = seeded();
        let detail = detail_cached(&conn, &offline_client(), "v17", false)
            .await
            .expect("cached");
        assert_eq!(detail.title, "Muv-Luv");
    }

    #[tokio::test]
    async fn seeded_characters_serve_without_network() {
        let conn = seeded();
        let characters = characters_cached(&conn, &offline_client(), "v17", false)
            .await
            .expect("cached");
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].name, "Meiya");
    }

    #[tokio::test]
    async fn force_refresh_bypasses_seed() {
        let conn = seeded();
        assert_eq!(
            detail_cached(&conn, &offline_client(), "v17", true)
                .await
                .is_err(),
            true
        );
    }

    #[tokio::test]
    async fn missing_entry_attempts_network() {
        let conn = open_in_memory().expect("open");
        assert_eq!(
            detail_cached(&conn, &offline_client(), "v999", false)
                .await
                .is_err(),
            true
        );
    }

    #[test]
    fn invalidate_removes_both_kinds() {
        let conn = seeded();
        invalidate(&conn, "v17").expect("invalidate");
        assert_eq!(
            db::cache_get(&conn, KIND_DETAIL, "v17")
                .expect("get")
                .is_none(),
            true
        );
        assert_eq!(
            db::cache_get(&conn, KIND_CHARACTERS, "v17")
                .expect("get")
                .is_none(),
            true
        );
    }

    #[test]
    fn cache_put_overwrites_and_clear_empties() {
        let conn = open_in_memory().expect("open");
        db::cache_put(&conn, KIND_DETAIL, "v1", "{\"a\":1}", 1).expect("put");
        db::cache_put(&conn, KIND_DETAIL, "v1", "{\"a\":2}", 2).expect("overwrite");
        let entry = db::cache_get(&conn, KIND_DETAIL, "v1")
            .expect("get")
            .expect("found");
        assert_eq!(entry.value, "{\"a\":2}");
        assert_eq!(entry.fetched_at, 2);
        clear(&conn).expect("clear");
        assert_eq!(
            db::cache_get(&conn, KIND_DETAIL, "v1")
                .expect("get")
                .is_none(),
            true
        );
    }
}
