use redb::ReadableTable;
use tauri::State;

use crate::database::{
    disk_cache_get_async, disk_cache_set_async, save_settings, CHAR_CACHE, VN_CACHE,
};
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
) -> AppResult<VndbVnDetail> {
    let refresh = force_refresh.unwrap_or(false);

    if !refresh {
        if let Some(cached) = state.vn_mem_cache.lock().get(&vndb_id) {
            return Ok(cached.clone());
        }
    }

    if !refresh {
        let cached = disk_cache_get_async::<VndbVnDetail>(
            state.db.clone(),
            VN_CACHE,
            vndb_id.clone(),
        )
        .await;

        if let Some(cached) = cached {
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

    disk_cache_set_async(
        state.db.clone(),
        VN_CACHE,
        vndb_id.clone(),
        detail.clone(),
    );

    Ok(detail)
}

#[tauri::command]
#[specta::specta]
pub async fn fetch_vndb_characters(
    vndb_id: String,
    force_refresh: Option<bool>,
    state: State<'_, AppState>,
) -> AppResult<Vec<VndbCharacter>> {
    let refresh = force_refresh.unwrap_or(false);

    if !refresh {
        if let Some(cached) = state.char_mem_cache.lock().get(&vndb_id) {
            return Ok(cached.clone());
        }
    }

    if !refresh {
        let cached = disk_cache_get_async::<Vec<VndbCharacter>>(
            state.db.clone(),
            CHAR_CACHE,
            vndb_id.clone(),
        )
        .await;

        if let Some(cached) = cached {
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

    disk_cache_set_async(
        state.db.clone(),
        CHAR_CACHE,
        vndb_id.clone(),
        chars.clone(),
    );

    Ok(chars)
}

#[tauri::command]
#[specta::specta]
pub fn clear_vndb_cache(vndb_id: String, state: State<AppState>) -> AppResult<()> {
    state.vn_mem_cache.lock().remove(&vndb_id);
    state.char_mem_cache.lock().remove(&vndb_id);

    if let Some(ref db) = state.db {
        if let Ok(write_txn) = db.begin_write() {
            if let Ok(mut t) = write_txn.open_table(VN_CACHE) {
                let _ = t.remove(vndb_id.as_str());
            }
            if let Ok(mut t) = write_txn.open_table(CHAR_CACHE) {
                let _ = t.remove(vndb_id.as_str());
            }
            let _ = write_txn.commit();
        }
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn clear_all_cache(state: State<AppState>) -> AppResult<()> {
    state.vn_mem_cache.lock().clear();
    state.char_mem_cache.lock().clear();

    if let Some(ref db) = state.db {
        let write_txn = db.begin_write()?;
        {
            if let Ok(mut table) = write_txn.open_table(VN_CACHE) {
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e: redb::StorageError| AppError::Database(e.to_string()))?
                    .filter_map(|entry: Result<(redb::AccessGuard<'_, &str>, redb::AccessGuard<'_, &[u8]>), redb::StorageError>| {
                        entry.ok().map(|(k, _)| k.value().to_string())
                    })
                    .collect();
                for key in keys {
                    let _ = table.remove(key.as_str());
                }
            }
        }
        {
            if let Ok(mut table) = write_txn.open_table(CHAR_CACHE) {
                let keys: Vec<String> = table
                    .iter()
                    .map_err(|e: redb::StorageError| AppError::Database(e.to_string()))?
                    .filter_map(|entry: Result<(redb::AccessGuard<'_, &str>, redb::AccessGuard<'_, &[u8]>), redb::StorageError>| {
                        entry.ok().map(|(k, _)| k.value().to_string())
                    })
                    .collect();
                for key in keys {
                    let _ = table.remove(key.as_str());
                }
            }
        }
        write_txn.commit()?;
        log::info!("VNDB cache cleared successfully");
    }
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
