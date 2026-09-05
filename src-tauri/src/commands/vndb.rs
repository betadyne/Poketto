use tauri::State;

use crate::database::{save_settings, AppDatabase, CHAR_CACHE_PREFIX, VN_CACHE_PREFIX};
use crate::error::{AppError, AppResult};
use crate::models::{
    VndbAuthInfo, VndbCharacter, VndbResponse, VndbSearchResult, VndbUserListItem, VndbVnDetail,
};
use crate::state::AppState;

fn extract_token(state: &AppState) -> AppResult<String> {
    state
        .settings
        .lock()
        .vndb_token
        .clone()
        .ok_or_else(|| AppError::AuthRequired("No VNDB token".into()))
}

async fn vndb_authenticated_patch(
    state: &AppState,
    url: &str,
    body: serde_json::Value,
) -> AppResult<()> {
    let token = extract_token(state)?;

    let response = state
        .http_client
        .patch(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Token {}", token))
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let err = response.text().await.unwrap_or_default();
        return Err(AppError::VndbApi(format!("VNDB API error: {}", err)));
    }
    Ok(())
}

fn vn_cache_key(vndb_id: &str) -> String {
    format!("{VN_CACHE_PREFIX}{vndb_id}")
}

fn char_cache_key(vndb_id: &str) -> String {
    format!("{CHAR_CACHE_PREFIX}{vndb_id}")
}

fn read_json_cache<T: serde::de::DeserializeOwned + Clone>(
    db: &AppDatabase,
    key: &str,
) -> Option<T> {
    match db.get_vndb_cache(key) {
        Ok(Some(json)) => match serde_json::from_str(&json) {
            Ok(value) => Some(value),
            Err(e) => {
                log::warn!("Discarding corrupt VNDB cache entry {key}: {e}");
                None
            }
        },
        Ok(None) => None,
        Err(e) => {
            log::warn!("VNDB disk cache read failed: {e}");
            None
        }
    }
}

fn write_json_cache<T: serde::Serialize>(db: &AppDatabase, key: &str, value: &T) {
    match serde_json::to_string(value) {
        Ok(json) => {
            if let Err(e) = db.set_vndb_cache(key, &json) {
                log::warn!("VNDB disk cache write failed: {e}");
            }
        }
        Err(e) => log::warn!("VNDB cache serialization failed: {e}"),
    }
}

#[tauri::command]
#[specta::specta]
pub async fn search_vndb(
    query: String,
    state: State<'_, AppState>,
) -> AppResult<Vec<VndbSearchResult>> {
    let body = serde_json::json!({
        "filters": ["search", "=", query],
        "fields": "id, title, image.url, released, rating",
        "results": 10
    });

    let response = state
        .http_client
        .post("https://api.vndb.org/kana/vn")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let vndb_response: VndbResponse<VndbSearchResult> = response.json().await?;
    Ok(vndb_response.results)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_vndb_detail(
    vndb_id: String,
    force_refresh: Option<bool>,
    state: State<'_, AppState>,
    db: State<'_, AppDatabase>,
) -> AppResult<VndbVnDetail> {
    let refresh = force_refresh.unwrap_or(false);

    if !refresh {
        if let Some(cached) = state.vn_mem_cache.lock().get(&vndb_id) {
            return Ok(cached.clone());
        }
    }

    if !refresh {
        if let Some(cached) = read_json_cache::<VndbVnDetail>(&db, &vn_cache_key(&vndb_id)) {
            state
                .vn_mem_cache
                .lock()
                .insert(vndb_id.clone(), cached.clone());
            return Ok(cached);
        }
    }

    let body = serde_json::json!({
        "filters": ["id", "=", vndb_id],
        "fields": "id, title, image.url, image.sexual, image.violence, released, rating, description, length, length_minutes, tags.id, tags.name, tags.rating, tags.spoiler, developers.id, developers.name",
        "results": 1
    });

    let response = state
        .http_client
        .post("https://api.vndb.org/kana/vn")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let vndb_response: VndbResponse<VndbVnDetail> = response.json().await?;
    let detail = vndb_response
        .results
        .into_iter()
        .next()
        .ok_or_else(|| AppError::NotFound("VN not found".into()))?;

    state
        .vn_mem_cache
        .lock()
        .insert(vndb_id.clone(), detail.clone());

    write_json_cache(&db, &vn_cache_key(&vndb_id), &detail);

    Ok(detail)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_vndb_characters(
    vndb_id: String,
    force_refresh: Option<bool>,
    state: State<'_, AppState>,
    db: State<'_, AppDatabase>,
) -> AppResult<Vec<VndbCharacter>> {
    let refresh = force_refresh.unwrap_or(false);

    if !refresh {
        if let Some(cached) = state.char_mem_cache.lock().get(&vndb_id) {
            return Ok(cached.clone());
        }
    }

    if !refresh {
        if let Some(cached) = read_json_cache::<Vec<VndbCharacter>>(&db, &char_cache_key(&vndb_id))
        {
            state
                .char_mem_cache
                .lock()
                .insert(vndb_id.clone(), cached.clone());
            return Ok(cached);
        }
    }

    let body = serde_json::json!({
        "filters": ["vn", "=", ["id", "=", vndb_id]],
        "fields": "id, name, original, aliases, image.url, image.sexual, image.violence, description, blood_type, height, weight, bust, waist, hips, cup, age, birthday, sex, vns.id, vns.role, vns.spoiler, traits.id, traits.name, traits.group_id, traits.group_name, traits.spoiler",
        "results": 50
    });

    let response = state
        .http_client
        .post("https://api.vndb.org/kana/character")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let vndb_response: VndbResponse<VndbCharacter> = response.json().await?;
    let chars = vndb_response.results;

    state
        .char_mem_cache
        .lock()
        .insert(vndb_id.clone(), chars.clone());

    write_json_cache(&db, &char_cache_key(&vndb_id), &chars);

    Ok(chars)
}

#[tauri::command]
#[specta::specta]
pub fn clear_vndb_cache(
    vndb_id: String,
    state: State<AppState>,
    db: State<AppDatabase>,
) -> AppResult<()> {
    state.vn_mem_cache.lock().remove(&vndb_id);
    state.char_mem_cache.lock().remove(&vndb_id);

    db.delete_vndb_cache(&vn_cache_key(&vndb_id))?;
    db.delete_vndb_cache(&char_cache_key(&vndb_id))?;
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn clear_all_cache(state: State<AppState>, db: State<AppDatabase>) -> AppResult<()> {
    state.vn_mem_cache.lock().clear();
    state.char_mem_cache.lock().clear();

    db.clear_vndb_cache()?;
    log::info!("VNDB cache cleared successfully");
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub async fn vndb_auth_check(state: State<'_, AppState>) -> AppResult<VndbAuthInfo> {
    let token = {
        let settings = state.settings.lock();
        settings
            .vndb_token
            .clone()
            .ok_or_else(|| AppError::AuthRequired("No VNDB token configured".into()))?
    };

    let response = state
        .http_client
        .get("https://api.vndb.org/kana/authinfo")
        .header("Authorization", format!("Token {}", token))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(AppError::AuthRequired("Invalid token".into()));
    }

    let auth_info: VndbAuthInfo = response.json().await?;

    let mut settings = state.settings.lock();
    settings.vndb_user_id = Some(auth_info.id.clone());
    let _ = save_settings(&settings);

    Ok(auth_info)
}

#[tauri::command]
#[specta::specta]
pub async fn vndb_get_user_vn(
    vndb_id: String,
    state: State<'_, AppState>,
) -> AppResult<Option<VndbUserListItem>> {
    let token = extract_token(&state)?;
    let user_id = state
        .settings
        .lock()
        .vndb_user_id
        .clone()
        .ok_or_else(|| AppError::AuthRequired("Not authenticated".into()))?;

    let body = serde_json::json!({
        "user": user_id,
        "filters": ["id", "=", vndb_id],
        "fields": "id, vote, labels.id, labels.label, started, finished",
        "results": 1
    });

    let response = state
        .http_client
        .post("https://api.vndb.org/kana/ulist")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Token {}", token))
        .json(&body)
        .send()
        .await?;

    let vndb_response: VndbResponse<VndbUserListItem> = response.json().await?;
    Ok(vndb_response.results.into_iter().next())
}

#[tauri::command]
#[specta::specta]
pub async fn vndb_set_status(
    vndb_id: String,
    label_id: i32,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let labels_unset: Vec<i32> = [1, 2, 3, 4, 5]
        .into_iter()
        .filter(|&x| x != label_id)
        .collect();
    let body = serde_json::json!({
        "labels_set": [label_id],
        "labels_unset": labels_unset
    });

    vndb_authenticated_patch(
        &state,
        &format!("https://api.vndb.org/kana/ulist/{}", vndb_id),
        body,
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn vndb_set_vote(
    vndb_id: String,
    vote: i32,
    state: State<'_, AppState>,
) -> AppResult<()> {
    vndb_authenticated_patch(
        &state,
        &format!("https://api.vndb.org/kana/ulist/{}", vndb_id),
        serde_json::json!({ "vote": vote }),
    )
    .await
}

#[tauri::command]
#[specta::specta]
pub async fn vndb_remove_vote(vndb_id: String, state: State<'_, AppState>) -> AppResult<()> {
    vndb_authenticated_patch(
        &state,
        &format!("https://api.vndb.org/kana/ulist/{}", vndb_id),
        serde_json::json!({ "vote": null }),
    )
    .await
}
