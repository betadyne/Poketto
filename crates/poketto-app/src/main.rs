slint::include_modules!();

mod adapters;
mod image_loader;
mod log_buffer;

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use slint::{Model, ModelRc, VecModel, Weak};

use adapters::{assemble_detail, log_lines, presence_buttons, DetailPayload, LibraryFilter};
use copypasta::ClipboardProvider;
use image_loader::{ImageLoader, LoadedCover};
use log_buffer::{LogBuffer, LogBufferLayer};
use poketto_core::discord::{PresenceHandle, PresenceUpdate};
use poketto_core::process::{self, LocalTimestamps, RunTracker};
use poketto_core::vndb::VndbClient;
use tracing_subscriber::{prelude::*, util::SubscriberInitExt};

const SCREEN_LIBRARY: i32 = 0;
const SCREEN_DETAIL: i32 = 1;
const SCREEN_SETTINGS: i32 = 2;
const SCREEN_LOGS: i32 = 3;

fn data_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Poketto")
}

fn db_path() -> std::path::PathBuf {
    data_dir().join("library.db")
}

fn cover_dir() -> std::path::PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(data_dir)
        .join("poketto")
        .join("covers")
}

fn refresh(
    app: &AppWindow,
    model: &VecModel<GameCardData>,
    conn: &poketto_core::db::Connection,
    filter: LibraryFilter,
    sort: (poketto_core::db::SortBy, poketto_core::db::SortOrder),
    loader: &ImageLoader,
) {
    let query = app.get_query().to_string();
    match adapters::refresh_library(model, conn, filter, &query, sort) {
        Ok(games) => {
            loader.next_generation();
            for game in &games {
                if let Some(url) = game.cover_url.as_deref() {
                    loader.request(&game.id, url);
                }
            }
        }
        Err(e) => tracing::warn!("library refresh failed: {e}"),
    }
}

fn slint_image(image: &image_loader::DecodedImage) -> slint::Image {
    let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
        &image.pixels,
        image.width,
        image.height,
    );
    slint::Image::from_rgba8(buffer)
}

fn apply_cover(model: &VecModel<GameCardData>, loaded: &LoadedCover) {
    let Some(image) = &loaded.image else {
        return;
    };
    for row in 0..model.row_count() {
        let Some(entry) = model.row_data(row) else {
            continue;
        };
        if entry.id.as_str() == loaded.game_id {
            model.set_row_data(
                row,
                GameCardData {
                    cover: slint_image(image),
                    show_cover: true,
                    ..entry
                },
            );
            return;
        }
    }
}

fn persist_fresh_detail(
    conn: &poketto_core::db::Connection,
    game_id: &str,
    vndb_id: &str,
    detail: &poketto_core::models::VndbVnDetail,
    characters: &[poketto_core::models::VndbCharacter],
) -> Result<(poketto_core::models::Game, Option<String>), String> {
    let game = poketto_core::db::get_game(conn, game_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Game not found.".to_string())?;
    let old_cover = game.cover_url.clone().or_else(|| {
        poketto_core::vndb::cached_detail_sync(conn, vndb_id)
            .ok()
            .flatten()
            .and_then(|cached| cached.image.map(|image| image.url))
    });
    poketto_core::vndb::store_detail_sync(conn, vndb_id, detail).map_err(|e| e.to_string())?;
    poketto_core::vndb::store_characters_sync(conn, vndb_id, characters)
        .map_err(|e| e.to_string())?;
    let mut updated = poketto_core::db::get_game(conn, game_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Game not found.".to_string())?;
    if updated.cover_url.is_none() {
        if let Some(url) = detail.image.as_ref().map(|image| image.url.clone()) {
            updated.cover_url = Some(url);
            poketto_core::db::update_game(conn, &updated).map_err(|e| e.to_string())?;
        }
    }
    let evict = match (old_cover, updated.cover_url.clone()) {
        (Some(old), Some(new)) if old != new => Some(old),
        _ => None,
    };
    Ok((updated, evict))
}

fn refresh_detail(
    rt: &tokio::runtime::Handle,
    client: &Arc<VndbClient>,
    loader: &Arc<ImageLoader>,
    app: Weak<AppWindow>,
    game_id: String,
) {
    let client = client.clone();
    let loader = loader.clone();
    rt.spawn(async move {
        let id = game_id.clone();
        let local = tokio::task::spawn_blocking(move || {
            let conn = poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
            poketto_core::db::get_game(&conn, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Game not found.".to_string())
        })
        .await;
        let fail = |target: &Weak<AppWindow>, message: String| {
            let target = target.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = target.upgrade() {
                    app.set_detail_error(message.into());
                    app.set_detail_refreshing(false);
                }
            });
        };
        let game = match local {
            Ok(Ok(game)) => game,
            Ok(Err(message)) => {
                fail(&app, format!("Refresh failed: {message}"));
                return;
            }
            Err(e) => {
                tracing::warn!("refresh task failed: {e}");
                fail(&app, "Refresh failed: task cancelled.".to_string());
                return;
            }
        };
        let Some(vndb_id) = game.vndb_id.clone() else {
            fail(&app, "Not linked to a VNDB entry.".to_string());
            return;
        };
        let fetched = tokio::join!(client.detail(&vndb_id), client.characters(&vndb_id));
        let (fresh_detail, fresh_characters) = match fetched {
            (Ok(detail), Ok(characters)) => (detail, characters),
            (Err(e), _) => {
                fail(&app, format!("Refresh failed: {e}"));
                return;
            }
            (_, Err(e)) => {
                fail(&app, format!("Refresh failed: {e}"));
                return;
            }
        };
        let game_id = game.id.clone();
        let stored = tokio::task::spawn_blocking(move || {
            let conn = poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
            let persisted =
                persist_fresh_detail(&conn, &game_id, &vndb_id, &fresh_detail, &fresh_characters)?;
            Ok::<_, String>((persisted, (fresh_detail, fresh_characters)))
        })
        .await;
        let ((game, evict), (fresh_detail, fresh_characters)) = match stored {
            Ok(Ok(done)) => done,
            Ok(Err(message)) => {
                fail(&app, format!("Refresh failed: {message}"));
                return;
            }
            Err(e) => {
                tracing::warn!("refresh store failed: {e}");
                fail(&app, "Refresh failed: task cancelled.".to_string());
                return;
            }
        };
        if let Some(old) = evict {
            loader.evict_url(&old);
        }
        let payload = assemble_detail(&game, Some(&fresh_detail), &fresh_characters);
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app.upgrade() {
                show_detail(&app, &loader, payload, false);
                app.set_detail_refreshing(false);
            }
        });
    });
}

fn open_detail(
    rt: &tokio::runtime::Handle,
    client: &Arc<VndbClient>,
    loader: &Arc<ImageLoader>,
    app: Weak<AppWindow>,
    game_id: String,
) {
    let client = client.clone();
    let loader = loader.clone();
    rt.spawn(async move {
        let id = game_id.clone();
        let local = tokio::task::spawn_blocking(move || {
            let conn = poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
            let game = poketto_core::db::get_game(&conn, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Game not found".to_string())?;
            let detail = game
                .vndb_id
                .as_deref()
                .and_then(|vndb_id| {
                    poketto_core::vndb::cached_detail_sync(&conn, vndb_id)
                        .ok()
                        .flatten()
                });
            let characters = game
                .vndb_id
                .as_deref()
                .and_then(|vndb_id| {
                    poketto_core::vndb::cached_characters_sync(&conn, vndb_id)
                        .ok()
                        .flatten()
                })
                .unwrap_or_default();
            Ok::<_, String>((game, detail, characters))
        })
        .await;
        let (game, mut detail, mut characters) = match local {
            Ok(Ok(loaded)) => loaded,
            Ok(Err(message)) => {
                tracing::warn!("detail load failed: {message}");
                return;
            }
            Err(e) => {
                tracing::warn!("detail task failed: {e}");
                return;
            }
        };
        if let Some(vndb_id) = game.vndb_id.clone() {
            let fetched = tokio::join!(client.detail(&vndb_id), client.characters(&vndb_id));
            if let (Ok(fresh_detail), Ok(fresh_characters)) = fetched {
                let game_id = game.id.clone();
                let stored = tokio::task::spawn_blocking(move || {
                    let conn =
                        poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
                    persist_fresh_detail(&conn, &game_id, &vndb_id, &fresh_detail, &fresh_characters)?;
                    Ok::<_, String>((fresh_detail, fresh_characters))
                })
                .await;
                if let Ok(Ok((fresh_detail, fresh_characters))) = stored {
                    detail = Some(fresh_detail);
                    characters = fresh_characters;
                }
            }
        }
        let payload = assemble_detail(&game, detail.as_ref(), &characters);
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app.upgrade() {
                show_detail(&app, &loader, payload, true);
            }
        });
    });
}

fn show_detail(app: &AppWindow, loader: &ImageLoader, payload: DetailPayload, navigate: bool) {
    app.set_detail_id(payload.id.clone().into());
    app.set_detail_title(payload.title.into());
    app.set_detail_meta(payload.meta.into());
    app.set_detail_playtime(payload.playtime.into());
    app.set_detail_synopsis(payload.synopsis.into());
    app.set_detail_finished(payload.finished);
    app.set_detail_nsfw(payload.nsfw);
    app.set_detail_cover_revealed(false);
    app.set_detail_show_spoilers(payload.show_spoilers);
    app.set_detail_playing(app.get_playing_id().as_str() == payload.id);
    app.set_detail_error("".into());
    let tags: Vec<DetailTag> = payload
        .tags
        .into_iter()
        .map(|(name, spoiler)| DetailTag {
            name: name.into(),
            spoiler,
        })
        .collect();
    app.set_detail_tags(ModelRc::from(Rc::new(VecModel::from(tags))));
    let characters: Vec<DetailCharacter> = payload
        .characters
        .into_iter()
        .map(|(name, role, spoiler)| DetailCharacter {
            name: name.into(),
            role: role.into(),
            spoiler,
        })
        .collect();
    app.set_detail_characters(ModelRc::from(Rc::new(VecModel::from(characters))));
    if let Some(url) = payload.cover_url {
        loader.request(&payload.id, &url);
    }
    app.set_library_rev(app.get_library_rev() + 1);
    if navigate {
        app.set_screen(SCREEN_DETAIL);
    }
}

fn sync_logs(app: &AppWindow, buffer: &LogBuffer, seen: &Cell<u64>) {
    let gen = buffer.generation();
    if gen == seen.get() {
        return;
    }
    seen.set(gen);
    let rows: Vec<LogLine> = log_lines(buffer)
        .into_iter()
        .map(|(text, level)| LogLine {
            text: text.into(),
            level,
        })
        .collect();
    app.set_logs_lines(ModelRc::from(Rc::new(VecModel::from(rows))));
}

fn begin_launch(
    app: &AppWindow,
    presence: &Arc<PresenceHandle>,
    rt_handle: &tokio::runtime::Handle,
    handle: &Weak<AppWindow>,
    id: &slint::SharedString,
) {
    if !app.get_playing_id().is_empty() {
        return;
    }
    app.set_detail_error("".into());
    app.set_playing_id(id.clone());
    if app.get_detail_id() == *id {
        app.set_detail_playing(true);
    }
    launch_game(rt_handle, presence, handle.clone(), id.to_string());
}

fn launch_game(
    rt: &tokio::runtime::Handle,
    presence: &Arc<PresenceHandle>,
    app: Weak<AppWindow>,
    game_id: String,
) {
    let presence = presence.clone();
    rt.spawn(async move {
        let id = game_id.clone();
        let prep = tokio::task::spawn_blocking(move || {
            let conn = poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
            let game = poketto_core::db::get_game(&conn, &id)
                .map_err(|e| e.to_string())?
                .ok_or_else(|| "Game not found".to_string())?;
            let settings = poketto_core::db::load_settings(&conn).unwrap_or_default();
            let developer = game
                .vndb_id
                .as_deref()
                .and_then(|vndb_id| {
                    poketto_core::vndb::cached_detail_sync(&conn, vndb_id)
                        .ok()
                        .flatten()
                })
                .and_then(|detail| detail.developers)
                .and_then(|developers| developers.into_iter().next())
                .map(|developer| developer.name);
            let cmd = process::build_command(&game, &settings).map_err(|e| e.to_string())?;
            Ok::<_, String>((game, settings, developer, cmd))
        })
        .await;
        let fail = |message: String| {
            let app = app.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(app) = app.upgrade() {
                    app.set_detail_error(message.into());
                    app.set_playing_id("".into());
                    app.set_detail_playing(false);
                }
            });
        };
        let (game, settings, developer, cmd) = match prep {
            Ok(Ok(prep)) => prep,
            Ok(Err(message)) => {
                fail(format!("Could not launch game: {message}"));
                return;
            }
            Err(e) => {
                fail(format!("Launch task failed: {e}"));
                return;
            }
        };
        let game_path = std::path::PathBuf::from(&game.path);
        let child = match process::spawn(cmd, &game_path) {
            Ok(child) => child,
            Err(e) => {
                fail(format!("Could not launch game: {e}"));
                return;
            }
        };
        presence.set_playing(PresenceUpdate::playing(
            &game.title,
            developer.as_deref(),
            game.cover_url.as_deref(),
            presence_buttons(&game, &settings),
            poketto_core::discord::unix_timestamp(),
        ));
        let tracker = RunTracker::start(&game.id, &game.title);
        let minutes = tracker
            .wait_for_exit(child)
            .await
            .map(|run| run.play_minutes())
            .unwrap_or(0);
        let timestamps = LocalTimestamps::now();
        let id = game.id.clone();
        let _ = tokio::task::spawn_blocking(move || {
            let conn = poketto_core::db::open(&db_path()).map_err(|e| e.to_string())?;
            if let Some(mut game) =
                poketto_core::db::get_game(&conn, &id).map_err(|e| e.to_string())?
            {
                game.play_time_minutes += minutes;
                game.last_played = Some(timestamps.rfc3339.clone());
                poketto_core::db::update_game(&conn, &game).map_err(|e| e.to_string())?;
            }
            poketto_core::db::record_play_session(
                &conn,
                &id,
                &timestamps.rfc3339,
                &timestamps.date,
                minutes,
            )
            .map_err(|e| e.to_string())
        })
        .await;
        presence.clear();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(app) = app.upgrade() {
                app.set_playing_id("".into());
                app.set_detail_playing(false);
                app.set_library_rev(app.get_library_rev() + 1);
            }
        });
    });
}

fn load_settings_into(app: &AppWindow) {
    let conn = match poketto_core::db::open(&db_path()) {
        Ok(conn) => conn,
        Err(e) => {
            tracing::warn!("settings load failed: {e}");
            return;
        }
    };
    let settings = poketto_core::db::load_settings(&conn).unwrap_or_default();
    app.set_set_wine_binary(settings.default_wine_binary.unwrap_or_default().into());
    app.set_set_wine_prefix(settings.default_wine_prefix.unwrap_or_default().into());
    app.set_set_steam_runtime(settings.use_steam_runtime);
    app.set_set_discord_enabled(settings.discord_rpc_enabled);
    app.set_set_btn_game(settings.discord_btn_vndb_game);
    app.set_set_btn_profile(settings.discord_btn_vndb_profile);
    app.set_set_btn_github(settings.discord_btn_github);
    app.set_set_blur(settings.blur_nsfw);
}

fn main() -> Result<(), slint::PlatformError> {
    let log_buffer = LogBuffer::new();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .with(LogBufferLayer::new(log_buffer.clone()))
        .init();
    if let Err(e) = std::fs::create_dir_all(data_dir()) {
        tracing::warn!("data dir unavailable: {e}");
    }
    let conn = Rc::new(RefCell::new(
        poketto_core::db::open(&db_path()).expect("open library database"),
    ));

    let rt = tokio::runtime::Runtime::new().expect("start background runtime");
    let rt_handle = rt.handle().clone();
    let loader = Arc::new(ImageLoader::new(rt.handle(), cover_dir()).expect("start image loader"));
    let client = Arc::new(VndbClient::new());
    let (presence, _presence_worker) =
        poketto_core::discord::spawn_presence_worker(poketto_core::discord::DISCORD_CLIENT_ID);
    let presence = Arc::new(presence);
    let last_rev = Rc::new(Cell::new(0));
    let logs_seen = Rc::new(Cell::new(0u64));

    let app = AppWindow::new()?;
    let model: Rc<VecModel<GameCardData>> = Rc::new(VecModel::default());
    app.set_games(ModelRc::from(model.clone()));
    let filter = Rc::new(RefCell::new(LibraryFilter::default()));
    let sort = Rc::new(Cell::new((
        poketto_core::db::SortBy::Title,
        poketto_core::db::SortOrder::Asc,
    )));
    load_settings_into(&app);
    if let Ok(saved) = poketto_core::db::load_sort_pref(&conn.borrow()) {
        sort.set(saved);
        app.set_sort_index(adapters::sort_option_index(saved.0, saved.1));
    }

    refresh(&app, &model, &conn.borrow(), *filter.borrow(), sort.get(), &loader);
    {
        let handle = app.as_weak();
        app.on_open_settings(move || {
            if let Some(app) = handle.upgrade() {
                app.set_screen(SCREEN_SETTINGS);
            }
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        let client = client.clone();
        let loader = loader.clone();
        app.on_game_clicked(move |id| {
            open_detail(&rt_handle, &client, &loader, handle.clone(), id.to_string());
        });
    }
    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        let loader = loader.clone();
        let sort = sort.clone();
        app.on_search_accepted(move |_| {
            if let Some(app) = handle.upgrade() {
                refresh(&app, &model, &conn.borrow(), *filter.borrow(), sort.get(), &loader);
            }
        });
    }
    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        let loader = loader.clone();
        let sort = sort.clone();
        app.on_filter_changed(move |index| {
            if let Some(app) = handle.upgrade() {
                *filter.borrow_mut() = LibraryFilter::from_index(index);
                app.set_active_filter(index);
                refresh(&app, &model, &conn.borrow(), *filter.borrow(), sort.get(), &loader);
            }
        });
    }
    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        let sort = sort.clone();
        let loader = loader.clone();
        app.on_sort_changed(move |field, asc| {
            let order = if asc {
                poketto_core::db::SortOrder::Asc
            } else {
                poketto_core::db::SortOrder::Desc
            };
            let sort_by = match field.as_str() {
                "title" => poketto_core::db::SortBy::Title,
                "last_played" => poketto_core::db::SortBy::LastPlayed,
                "play_time" => poketto_core::db::SortBy::PlayTime,
                _ => {
                    tracing::warn!("unknown sort field: {field}");
                    return;
                }
            };
            sort.set((sort_by, order));
            if let Err(e) = poketto_core::db::save_sort_pref(&conn.borrow(), sort_by, order) {
                tracing::warn!("sort preference save failed: {e}");
            }
            if let Some(app) = handle.upgrade() {
                refresh(&app, &model, &conn.borrow(), *filter.borrow(), sort.get(), &loader);
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_detail_back(move || {
            if let Some(app) = handle.upgrade() {
                app.set_screen(SCREEN_LIBRARY);
            }
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        let presence = presence.clone();
        app.on_launch_clicked(move |id| {
            if let Some(app) = handle.upgrade() {
                begin_launch(&app, &presence, &rt_handle, &handle, &id);
            }
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        let presence = presence.clone();
        app.on_play_game(move |id| {
            if let Some(app) = handle.upgrade() {
                begin_launch(&app, &presence, &rt_handle, &handle, &id);
            }
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_edit_game(move |id| {
            let Some(app) = handle.upgrade() else {
                return;
            };
            match poketto_core::db::get_game(&conn.borrow(), id.as_str()) {
                Ok(Some(game)) => open_editor_for_edit(&app, &game),
                Ok(None) => tracing::warn!("edit requested for missing game"),
                Err(e) => tracing::warn!("edit load failed: {e}"),
            }
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_toggle_hide(move |id| {
            let result = (|| {
                let conn = conn.borrow();
                let mut game = poketto_core::db::get_game(&conn, id.as_str())?
                    .ok_or_else(|| poketto_core::db::DbError::GameNotFound(id.to_string()))?;
                game.is_hidden = !game.is_hidden;
                poketto_core::db::update_game(&conn, &game)?;
                Ok::<_, poketto_core::db::DbError>(())
            })();
            if let Err(e) = result {
                tracing::warn!("toggle hide failed: {e}");
                return;
            }
            if let Some(app) = handle.upgrade() {
                app.set_library_rev(app.get_library_rev() + 1);
            }
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_remove_game(move |id| {
            let removed = (|| {
                let conn = conn.borrow();
                let game = poketto_core::db::get_game(&conn, id.as_str())?
                    .ok_or_else(|| poketto_core::db::DbError::GameNotFound(id.to_string()))?;
                let cover = game.cover_url.clone();
                poketto_core::db::delete_game(&conn, id.as_str())?;
                Ok::<_, poketto_core::db::DbError>(cover)
            })();
            let cover = match removed {
                Ok(cover) => cover,
                Err(e) => {
                    tracing::warn!("remove game failed: {e}");
                    return;
                }
            };
            if let Some(url) = cover {
                let path = image_loader::thumbnail_path(&cover_dir(), &url);
                let _ = std::fs::remove_file(&path);
            }
            if let Some(app) = handle.upgrade() {
                if app.get_detail_id().as_str() == id.as_str() {
                    app.set_screen(SCREEN_LIBRARY);
                }
                app.set_library_rev(app.get_library_rev() + 1);
            }
        });
    }
    {
        let model = model.clone();
        app.on_reveal_cover(move |id| {
            for row in 0..model.row_count() {
                let Some(entry) = model.row_data(row) else {
                    continue;
                };
                if entry.id.as_str() == id.as_str() {
                    model.set_row_data(
                        row,
                        GameCardData {
                            revealed: true,
                            ..entry
                        },
                    );
                    return;
                }
            }
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_spoilers_toggled(move |allowed| {
            let Some(app) = handle.upgrade() else {
                return;
            };
            let id = app.get_detail_id().to_string();
            let updated = (|| {
                let conn = conn.borrow();
                let mut game = poketto_core::db::get_game(&conn, &id)?
                    .ok_or_else(|| poketto_core::db::DbError::GameNotFound(id.clone()))?;
                game.show_spoilers = allowed;
                poketto_core::db::update_game(&conn, &game)?;
                Ok::<_, poketto_core::db::DbError>(())
            })();
            if let Err(e) = updated {
                tracing::warn!("spoiler preference save failed: {e}");
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_configure_clicked(move || {
            if let Some(app) = handle.upgrade() {
                app.set_screen(SCREEN_SETTINGS);
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_settings_back(move || {
            if let Some(app) = handle.upgrade() {
                app.set_screen(SCREEN_LIBRARY);
            }
        });
    }
    {
        let handle = app.as_weak();
        let log_buffer = log_buffer.clone();
        let logs_seen = logs_seen.clone();
        app.on_open_logs(move || {
            if let Some(app) = handle.upgrade() {
                sync_logs(&app, &log_buffer, &logs_seen);
                app.set_screen(SCREEN_LOGS);
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_logs_back(move || {
            if let Some(app) = handle.upgrade() {
                app.set_screen(SCREEN_SETTINGS);
            }
        });
    }
    {
        let handle = app.as_weak();
        let log_buffer = log_buffer.clone();
        app.on_logs_copy(move || {
            let Some(app) = handle.upgrade() else {
                return;
            };
            let text = log_buffer.export_text();
            let count = text.lines().count();
            match copypasta::ClipboardContext::new().and_then(|mut ctx| ctx.set_contents(text)) {
                Ok(()) => app.set_logs_status(format!("Copied {count} lines to clipboard.").into()),
                Err(e) => app.set_logs_status(format!("Copy failed: {e}").into()),
            }
        });
    }
    {
        let handle = app.as_weak();
        let log_buffer = log_buffer.clone();
        let logs_seen = logs_seen.clone();
        app.on_logs_clear(move || {
            let Some(app) = handle.upgrade() else {
                return;
            };
            log_buffer.clear();
            sync_logs(&app, &log_buffer, &logs_seen);
            app.set_logs_status("Log cleared.".into());
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_settings_save(move || {
            let Some(app) = handle.upgrade() else {
                return;
            };
            let mut stored = poketto_core::db::load_settings(&conn.borrow()).unwrap_or_default();
            let text = |value: slint::SharedString| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            };
            stored.default_wine_binary = text(app.get_set_wine_binary());
            stored.default_wine_prefix = text(app.get_set_wine_prefix());
            stored.use_steam_runtime = app.get_set_steam_runtime();
            stored.discord_rpc_enabled = app.get_set_discord_enabled();
            stored.discord_btn_vndb_game = app.get_set_btn_game();
            stored.discord_btn_vndb_profile = app.get_set_btn_profile();
            stored.discord_btn_github = app.get_set_btn_github();
            stored.blur_nsfw = app.get_set_blur();
            if let Err(e) = poketto_core::db::save_settings(&conn.borrow(), &stored) {
                tracing::warn!("settings save failed: {e}");
            } else {
                tracing::info!("settings saved");
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_detect_wine(move || {
            let handle = handle.clone();
            std::thread::spawn(move || {
                let runners = poketto_core::wine::get_all_wine_versions();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = handle.upgrade() {
                        let rows: Vec<WineRow> = runners
                            .into_iter()
                            .map(|runner| WineRow {
                                name: runner.name.into(),
                                binary: runner.binary_path.into(),
                            })
                            .collect();
                        if app.get_set_wine_binary().is_empty() {
                            if let Some(first) = rows.first() {
                                app.set_set_wine_binary(first.binary.clone());
                            }
                        }
                        app.set_wine_runners(ModelRc::from(Rc::new(VecModel::from(rows))));
                    }
                });
            });
        });
    }
    {
        let handle = app.as_weak();
        app.on_select_wine(move |binary| {
            if let Some(app) = handle.upgrade() {
                app.set_set_wine_binary(binary);
            }
        });
    }
fn open_editor_for_add(app: &AppWindow) {
    app.set_edit_heading("Add game".into());
    app.set_edit_id("".into());
    app.set_edit_title("".into());
    app.set_edit_vndb_id("".into());
    app.set_edit_exec_path("".into());
    app.set_edit_work_dir("".into());
    app.set_edit_cover_url("".into());
    app.set_edit_platform(0);
    app.set_edit_wine_prefix("".into());
    app.set_edit_wine_binary("".into());
    app.set_edit_error("".into());
    app.set_edit_vndb_query("".into());
    app.set_edit_vndb_hits(ModelRc::from(Rc::new(VecModel::<VndbHit>::default())));
    app.set_edit_searching(false);
    app.set_edit_open(true);
}

fn open_editor_for_edit(app: &AppWindow, game: &poketto_core::models::Game) {
    app.set_edit_heading("Edit game".into());
    app.set_edit_id(game.id.clone().into());
    app.set_edit_title(game.title.clone().into());
    app.set_edit_vndb_id(game.vndb_id.clone().unwrap_or_default().into());
    app.set_edit_exec_path(game.path.clone().into());
    app.set_edit_work_dir(game.work_dir.clone().unwrap_or_default().into());
    app.set_edit_cover_url(game.cover_url.clone().unwrap_or_default().into());
    app.set_edit_platform(adapters::platform_index(game));
    let (prefix, binary) = game
        .wine_settings
        .as_ref()
        .map(|wine| {
            (
                wine.wine_prefix.clone().unwrap_or_default(),
                wine.wine_version.clone().unwrap_or_default(),
            )
        })
        .unwrap_or_default();
    app.set_edit_wine_prefix(prefix.into());
    app.set_edit_wine_binary(binary.into());
    app.set_edit_error("".into());
    app.set_edit_vndb_query("".into());
    app.set_edit_vndb_hits(ModelRc::from(Rc::new(VecModel::<VndbHit>::default())));
    app.set_edit_searching(false);
    app.set_edit_open(true);
}

    {
        let handle = app.as_weak();
        app.on_add_clicked(move || {
            if let Some(app) = handle.upgrade() {
                open_editor_for_add(&app);
            }
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        app.on_edit_clicked(move || {
            let Some(app) = handle.upgrade() else {
                return;
            };
            let id = app.get_detail_id().to_string();
            match poketto_core::db::get_game(&conn.borrow(), &id) {
                Ok(Some(game)) => open_editor_for_edit(&app, &game),
                Ok(None) => tracing::warn!("edit requested for missing game"),
                Err(e) => tracing::warn!("edit load failed: {e}"),
            }
        });
    }
    {
        let handle = app.as_weak();
        app.on_edit_cancel(move || {
            if let Some(app) = handle.upgrade() {
                app.set_edit_open(false);
            }
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        let client = client.clone();
        let loader = loader.clone();
        app.on_refresh_metadata(move || {
            if let Some(app) = handle.upgrade() {
                if app.get_detail_refreshing() {
                    return;
                }
                app.set_detail_refreshing(true);
                app.set_detail_error("".into());
                refresh_detail(
                    &rt_handle,
                    &client,
                    &loader,
                    handle.clone(),
                    app.get_detail_id().to_string(),
                );
            }
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        app.on_edit_browse(move || {
            let handle = handle.clone();
            rt_handle.spawn(async move {
                let picked = rfd::AsyncFileDialog::new()
                    .set_title("Select game executable")
                    .pick_file()
                    .await;
                if let Some(file) = picked {
                    let path = file.path().to_path_buf();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(app) = handle.upgrade() {
                            app.set_edit_exec_path(
                                path.to_str().unwrap_or_default().into(),
                            );
                        }
                    });
                }
            });
        });
    }
    {
        let handle = app.as_weak();
        let rt_handle = rt_handle.clone();
        let client = client.clone();
        app.on_edit_search(move |query| {
            let handle = handle.clone();
            let client = client.clone();
            let Some(app) = handle.upgrade() else {
                return;
            };
            app.set_edit_searching(true);
            let query = query.to_string();
            rt_handle.spawn(async move {
                let hits = match client.search(&query).await {
                    Ok(results) => results
                        .into_iter()
                        .take(8)
                        .map(|hit| VndbHit {
                            id: hit.id.into(),
                            title: hit.title.into(),
                            cover_url: hit
                                .image
                                .map(|image| image.url)
                                .unwrap_or_default()
                                .into(),
                        })
                        .collect(),
                    Err(e) => {
                        tracing::warn!("vndb search failed: {e}");
                        Vec::new()
                    }
                };
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(app) = handle.upgrade() {
                        app.set_edit_vndb_hits(ModelRc::from(Rc::new(VecModel::from(hits))));
                        app.set_edit_searching(false);
                    }
                });
            });
        });
    }
    {
        let handle = app.as_weak();
        let conn = conn.clone();
        let rt_handle = rt_handle.clone();
        let client = client.clone();
        let loader = loader.clone();
        app.on_edit_save(move |form| {
            let Some(app) = handle.upgrade() else {
                return;
            };
            if let Some(error) =
                adapters::validate_game_form(form.title.as_str(), form.vndb_id.as_str())
            {
                app.set_edit_error(error.into());
                return;
            }
            if form.exec_path.trim().is_empty() {
                app.set_edit_error("Executable path is required.".into());
                return;
            }
            let is_new = form.id.is_empty();
            let existing = if is_new {
                None
            } else {
                match poketto_core::db::get_game(&conn.borrow(), form.id.as_str()) {
                    Ok(game) => game,
                    Err(e) => {
                        app.set_edit_error(format!("Save failed: {e}").into());
                        return;
                    }
                }
            };
            if !is_new && existing.is_none() {
                app.set_edit_error("Game no longer exists.".into());
                return;
            }
            let steam = poketto_core::db::load_settings(&conn.borrow())
                .map(|settings| settings.use_steam_runtime)
                .unwrap_or(false);
            let game = adapters::apply_form(
                existing.as_ref(),
                &form,
                steam,
                &uuid::Uuid::new_v4().to_string(),
            );
            let saved_id = game.id.clone();
            let stored = if existing.is_some() {
                poketto_core::db::update_game(&conn.borrow(), &game)
            } else {
                poketto_core::db::insert_game(&conn.borrow(), &game)
            };
            match stored {
                Ok(()) => {
                    app.set_edit_open(false);
                    app.set_library_rev(app.get_library_rev() + 1);
                    if app.get_screen() == SCREEN_DETAIL
                        && app.get_detail_id().as_str() == saved_id
                    {
                        open_detail(
                            &rt_handle,
                            &client,
                            &loader,
                            handle.clone(),
                            saved_id,
                        );
                    }
                }
                Err(e) => app.set_edit_error(format!("Save failed: {e}").into()),
            }
        });
    }
    let drain_model = model.clone();
    let drain_loader = loader.clone();
    let drain_conn = conn.clone();
    let drain_sort = sort.clone();
    let drain_filter = filter.clone();
    let drain_handle = app.as_weak();
    let drain_rev = last_rev.clone();
    let drain_buffer = log_buffer.clone();
    let drain_seen = logs_seen.clone();
    let _cover_timer = slint::Timer::default();
    _cover_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(150),
        move || {
            for loaded in drain_loader.poll() {
                apply_cover(&drain_model, &loaded);
                if let Some(app) = drain_handle.upgrade() {
                    if loaded.game_id == app.get_detail_id().as_str() {
                        if let Some(image) = &loaded.image {
                            app.set_detail_cover(slint_image(image));
                            app.set_detail_show_cover(true);
                        }
                    }
                }
            }
            if let Some(app) = drain_handle.upgrade() {
                let rev = app.get_library_rev();
                if rev != drain_rev.get() {
                    drain_rev.set(rev);
                    refresh(
                        &app,
                        &drain_model,
                        &drain_conn.borrow(),
                        *drain_filter.borrow(),
                        drain_sort.get(),
                        &drain_loader,
                    );
                }
                if app.get_screen() == SCREEN_LOGS {
                    sync_logs(&app, &drain_buffer, &drain_seen);
                }
            }
        },
    );

    app.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_detail(image_url: Option<&str>) -> poketto_core::models::VndbVnDetail {
        poketto_core::models::VndbVnDetail {
            id: "v17".to_string(),
            title: "Muv-Luv".to_string(),
            image: image_url.map(|url| poketto_core::models::VndbImage {
                url: url.to_string(),
                sexual: 0.0,
                violence: 0.0,
            }),
            released: None,
            rating: None,
            description: None,
            length: None,
            length_minutes: None,
            devstatus: None,
            tags: None,
            developers: None,
        }
    }

    fn test_game(conn: &poketto_core::db::Connection, cover: Option<&str>) {
        let game = poketto_core::models::Game {
            id: "g1".to_string(),
            title: "Muv-Luv".to_string(),
            path: "/games/g1".to_string(),
            work_dir: None,
            vndb_id: Some("v17".to_string()),
            cover_url: cover.map(str::to_string),
            play_time_minutes: 0,
            is_finished: false,
            last_played: None,
            is_hidden: false,
            show_spoilers: false,
            game_type: None,
            wine_settings: None,
            rating: None,
        };
        poketto_core::db::insert_game(conn, &game).expect("insert");
    }

    #[test]
    fn persist_adopt_returns_old_url_for_eviction() {
        let conn = poketto_core::db::open_in_memory().expect("db");
        test_game(&conn, None);
        let cached = test_detail(Some("https://old.jpg"));
        poketto_core::vndb::store_detail_sync(&conn, "v17", &cached).expect("seed");
        let fresh = test_detail(Some("https://new.jpg"));
        let (game, evict) =
            persist_fresh_detail(&conn, "g1", "v17", &fresh, &[]).expect("persist");
        assert_eq!(game.cover_url.as_deref(), Some("https://new.jpg"));
        assert_eq!(evict.as_deref(), Some("https://old.jpg"));
    }

    #[test]
    fn persist_no_change_returns_none() {
        let conn = poketto_core::db::open_in_memory().expect("db");
        test_game(&conn, Some("https://cover.jpg"));
        let fresh = test_detail(Some("https://cover.jpg"));
        let (game, evict) =
            persist_fresh_detail(&conn, "g1", "v17", &fresh, &[]).expect("persist");
        assert_eq!(game.cover_url.as_deref(), Some("https://cover.jpg"));
        assert_eq!(evict, None);
    }
}
