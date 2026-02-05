use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tauri::{Emitter, Manager, State};
use tokio::task;

use crate::database::{disk_cache_get, get_current_timestamp, record_daily_playtime, save_games, VN_CACHE};
use crate::discord;
use crate::models::VndbVnDetail;
use crate::error::{AppError, AppResult};
use crate::models::{AppSettings, GameExitedPayload, GameMetadata, GameType, RunningGame, WineSettings, WineType};
use crate::state::AppState;

#[cfg(target_os = "linux")]
use crate::wine;

// ============================================================================
// Main Launch Command
// ============================================================================

#[tauri::command]
#[specta::specta]
pub fn launch_game(
    id: String,
    app_handle: tauri::AppHandle,
    state: State<AppState>,
) -> AppResult<()> {
    let games = state.games.lock();
    let game = games
        .iter()
        .find(|g| g.id == id)
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

    let path = PathBuf::from(&game.path);

    if !path.exists() {
        return Err(AppError::ProcessLaunch(format!(
            "Game executable not found: {}",
            path.display()
        )));
    }
    if !path.is_file() {
        return Err(AppError::ProcessLaunch(format!(
            "Path is not a file: {}",
            path.display()
        )));
    }

    let settings = state.settings.lock().clone();

    let mut child = spawn_game_process(game, &path, &settings)?;

    let game_title = game.title.clone();
    let cover_url = game.cover_url.clone();
    let discord_start = discord::get_unix_timestamp();

    let start_time = Instant::now();
    {
        let mut running = state.running_game.lock();
        *running = Some(RunningGame {
            id: id.clone(),
            start_time,
            title: game_title.clone(),
            cover_url: cover_url.clone(),
            discord_start_timestamp: discord_start,
        });
    }

    {
        let settings = state.settings.lock();
        if settings.discord_rpc_enabled {
            let developer = game.vndb_id.as_ref().and_then(|vndb_id| {
                let mut mem_cache = state.vn_mem_cache.lock();
                if let Some(vn) = mem_cache.get(vndb_id) {
                    return vn.developers.as_ref().and_then(|devs| {
                        devs.first().map(|d| d.name.clone())
                    });
                }

                if let Some(cached) = disk_cache_get::<VndbVnDetail>(
                    state.db.as_ref(),
                    VN_CACHE,
                    vndb_id,
                ) {
                    let developer_name = cached.developers.as_ref().and_then(|devs| {
                        devs.first().map(|d| d.name.clone())
                    });
                    mem_cache.insert(vndb_id.clone(), cached);
                    return developer_name;
                }

                None
            });

            let vndb_game_url = game.vndb_id.as_ref().map(|id| format!("https://vndb.org/{}", id));
            let vndb_profile_url = settings.vndb_user_id.as_ref().map(|id| format!("https://vndb.org/{}", id));
            const GITHUB_URL: &str = "https://github.com/betadyne/Poketto";

            let mut buttons: Vec<(&str, String)> = Vec::new();

            if settings.discord_btn_vndb_game {
                if let Some(ref url) = vndb_game_url {
                    buttons.push(("View on VNDB", url.clone()));
                }
            }
            if settings.discord_btn_vndb_profile && buttons.len() < 2 {
                if let Some(ref url) = vndb_profile_url {
                    buttons.push(("My VNDB Profile", url.clone()));
                }
            }
            if settings.discord_btn_github && buttons.len() < 2 {
                buttons.push(("GitHub", GITHUB_URL.to_string()));
            }

            let button_refs: Vec<(&str, &str)> = buttons
                .iter()
                .map(|(label, url)| (*label, url.as_str()))
                .collect();

            log::info!("Discord RPC buttons: {:?}", button_refs.iter().map(|(l, _)| *l).collect::<Vec<_>>());

            let _ = state.discord_rpc.set_activity(
                &game_title,
                cover_url.as_deref(),
                developer.as_deref(),
                button_refs,
                discord_start,
            );
        }
    }

    drop(games);

    let app_handle_clone = app_handle.clone();
    let game_id = id.clone();

    tauri::async_runtime::spawn(async move {
        let exit_result = task::spawn_blocking(move || child.wait()).await;

        let minutes = start_time.elapsed().as_secs() / 60;
        let state = app_handle_clone.state::<AppState>();

        let _ = state.discord_rpc.clear_activity();

        {
            let mut games = state.games.lock();
            if let Some(g) = games.iter_mut().find(|g| g.id == game_id) {
                g.play_time += minutes;
                g.last_played = Some(get_current_timestamp());
                let _ = save_games(&games);
            }
        }

        record_daily_playtime(&game_id, minutes);

        {
            let mut running = state.running_game.lock();
            *running = None;
        }

        let _ = app_handle.emit(
            "game-exited",
            GameExitedPayload {
                game_id: game_id.clone(),
                play_minutes: minutes,
            },
        );

        if let Ok(Err(e)) = exit_result {
            eprintln!("Game process error: {}", e);
        }
    });

    Ok(())
}

// ============================================================================
// Platform-specific game spawning
// ============================================================================

fn spawn_game_process(
    game: &GameMetadata,
    path: &Path,
    settings: &AppSettings,
) -> AppResult<std::process::Child> {
    #[cfg(target_os = "linux")]
    {
        spawn_game_linux(game, path, settings)
    }

    #[cfg(target_os = "windows")]
    {
        spawn_game_windows(path)
    }

    #[cfg(target_os = "macos")]
    {
        spawn_game_macos(path)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        Err(AppError::ProcessLaunch("Unsupported platform".into()))
    }
}

#[cfg(target_os = "windows")]
fn spawn_game_windows(path: &Path) -> AppResult<std::process::Child> {
    Command::new(path)
        .current_dir(path.parent().unwrap_or(path))
        .spawn()
        .map_err(|e| map_spawn_error(e, path))
}

#[cfg(target_os = "macos")]
fn spawn_game_macos(path: &Path) -> AppResult<std::process::Child> {
    Command::new(path)
        .current_dir(path.parent().unwrap_or(path))
        .spawn()
        .map_err(|e| map_spawn_error(e, path))
}

#[cfg(target_os = "linux")]
fn spawn_game_linux(
    game: &GameMetadata,
    path: &Path,
    settings: &AppSettings,
) -> AppResult<std::process::Child> {
    let game_type = game.game_type.clone().unwrap_or_else(|| {
        if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            GameType::WindowsExe
        } else {
            GameType::LinuxNative
        }
    });

    match game_type {
        GameType::LinuxNative => spawn_native_linux(path),
        GameType::WindowsExe => spawn_wine_linux(game, path, settings),
    }
}

#[cfg(target_os = "linux")]
fn spawn_native_linux(path: &Path) -> AppResult<std::process::Child> {
    log::info!("Launching native Linux game: {}", path.display());

    Command::new(path)
        .current_dir(path.parent().unwrap_or(path))
        .spawn()
        .map_err(|e| map_spawn_error(e, path))
}

#[cfg(target_os = "linux")]
fn spawn_wine_linux(
    game: &GameMetadata,
    path: &Path,
    app_settings: &AppSettings,
) -> AppResult<std::process::Child> {
    let wine_settings = resolve_wine_settings(game, app_settings);

    let wine_binary = wine_settings
        .wine_version
        .clone()
        .or_else(|| app_settings.default_wine_binary.clone())
        .or_else(|| wine::get_default_wine().map(|w| w.binary_path))
        .ok_or_else(|| {
            AppError::ProcessLaunch(
                "No Wine installation found. Please install Wine or configure Wine settings.".into(),
            )
        })?;

    let prefix_path = if wine_settings.use_global_prefix {
        app_settings
            .default_wine_prefix
            .clone()
            .unwrap_or_else(|| wine::get_global_prefix_path().to_str().unwrap_or("").to_string())
    } else {
        wine_settings
            .wine_prefix
            .clone()
            .unwrap_or_else(|| {
                wine::get_default_prefix_path(&game.id)
                    .to_str()
                    .unwrap_or("")
                    .to_string()
            })
    };

    let prefix_dir = Path::new(&prefix_path);
    wine::ensure_prefix_exists(prefix_dir).map_err(|e| AppError::ProcessLaunch(e))?;

    let wine_type = wine_settings.wine_type.clone().unwrap_or(WineType::Wine);
    let use_steam_runtime = wine_settings.use_steam_runtime || app_settings.use_steam_runtime;

    log::info!(
        "Launching with Wine: binary={}, prefix={}, type={:?}, steam_runtime={}",
        wine_binary,
        prefix_path,
        wine_type,
        use_steam_runtime
    );

    let child = match wine_type {
        WineType::Wine => {
            build_wine_command(&wine_binary, &prefix_path, path, use_steam_runtime, &wine_settings)?
        }
        WineType::Proton | WineType::ProtonGE | WineType::ProtonCachyOS => {
            build_proton_command(&wine_binary, &prefix_path, path, use_steam_runtime, &wine_settings)?
        }
    };

    Ok(child)
}

#[cfg(target_os = "linux")]
fn build_wine_command(
    wine_binary: &str,
    prefix_path: &str,
    game_path: &Path,
    use_steam_runtime: bool,
    wine_settings: &WineSettings,
) -> AppResult<std::process::Child> {
    let working_dir = game_path.parent().unwrap_or(game_path);

    let mut cmd = if use_steam_runtime && wine::is_steam_runtime_available() {
        if let Some(steam_run) = wine::get_steam_run_path() {
            let mut c = Command::new(&steam_run);
            c.arg(wine_binary);
            c
        } else {
            Command::new(wine_binary)
        }
    } else {
        Command::new(wine_binary)
    };

    cmd.env("WINEPREFIX", prefix_path)
        .env("WINEDEBUG", "-all")
        .current_dir(working_dir)
        .arg(game_path);

    for (key, value) in &wine_settings.env_vars {
        cmd.env(key, value);
    }

    cmd.spawn().map_err(|e| map_spawn_error(e, game_path))
}

#[cfg(target_os = "linux")]
fn build_proton_command(
    proton_binary: &str,
    prefix_path: &str,
    game_path: &Path,
    use_steam_runtime: bool,
    wine_settings: &WineSettings,
) -> AppResult<std::process::Child> {
    let working_dir = game_path.parent().unwrap_or(game_path);

    let steam_path = wine::get_steam_path()
        .map(|p| p.to_str().unwrap_or("").to_string())
        .unwrap_or_default();

    let mut cmd = if use_steam_runtime && wine::is_steam_runtime_available() {
        if let Some(steam_run) = wine::get_steam_run_path() {
            let mut c = Command::new(&steam_run);
            c.arg(proton_binary);
            c
        } else {
            Command::new(proton_binary)
        }
    } else {
        Command::new(proton_binary)
    };

    cmd.env("STEAM_COMPAT_DATA_PATH", prefix_path)
        .env("STEAM_COMPAT_CLIENT_INSTALL_PATH", &steam_path)
        .current_dir(working_dir)
        .arg("run")
        .arg(game_path);

    for (key, value) in &wine_settings.env_vars {
        cmd.env(key, value);
    }

    cmd.spawn().map_err(|e| map_spawn_error(e, game_path))
}

#[cfg(target_os = "linux")]
fn resolve_wine_settings(game: &GameMetadata, app_settings: &AppSettings) -> WineSettings {
    game.wine_settings.clone().unwrap_or_else(|| WineSettings {
        use_global_prefix: true,
        wine_prefix: app_settings.default_wine_prefix.clone(),
        wine_version: app_settings.default_wine_binary.clone(),
        wine_type: None,
        use_steam_runtime: app_settings.use_steam_runtime,
        env_vars: std::collections::HashMap::new(),
    })
}

fn map_spawn_error(e: std::io::Error, path: &Path) -> AppError {
    use std::io::ErrorKind;
    match e.kind() {
        ErrorKind::NotFound => AppError::ProcessLaunch(format!(
            "Executable not found or invalid: {}",
            path.display()
        )),
        ErrorKind::PermissionDenied => AppError::ProcessLaunch(format!(
            "Permission denied: cannot execute {}",
            path.display()
        )),
        _ => AppError::ProcessLaunch(format!(
            "Failed to launch game: {} ({})",
            e,
            path.display()
        )),
    }
}

// ============================================================================
// Other Commands
// ============================================================================

#[tauri::command]
#[specta::specta]
pub fn stop_tracking(state: State<AppState>) -> AppResult<u64> {
    let mut running = state.running_game.lock();
    if let Some(game) = running.take() {
        let elapsed = game.start_time.elapsed();
        let minutes = elapsed.as_secs() / 60;
        let game_id = game.id.clone();

        let mut games = state.games.lock();
        if let Some(g) = games.iter_mut().find(|g| g.id == game_id) {
            g.play_time += minutes;
            g.last_played = Some(get_current_timestamp());
            save_games(&games)?;
        }
        drop(games);

        record_daily_playtime(&game_id, minutes);

        return Ok(minutes);
    }
    Ok(0)
}

#[tauri::command]
#[specta::specta]
pub fn poll_running_game(state: State<AppState>) -> Option<String> {
    let running = state.running_game.lock();
    running.as_ref().map(|g| g.id.clone())
}

#[tauri::command]
#[specta::specta]
pub fn get_elapsed_time(state: State<AppState>) -> u64 {
    let running = state.running_game.lock();
    running
        .as_ref()
        .map(|r| r.start_time.elapsed().as_secs())
        .unwrap_or(0)
}
