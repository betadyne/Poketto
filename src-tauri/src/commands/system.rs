use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;
use tauri::{Emitter, Manager, State};
use tokio::task;

use crate::database::{record_daily_playtime, AppDatabase};
use crate::discord;
use crate::error::{AppError, AppResult};
use crate::models::{
    AppSettings, GameExitedPayload, GameMetadata, GameType, RunningGame, WineSettings, WineType,
};
use crate::state::AppState;
use crate::steam_watch::{persist_session, spawn_steam_watcher};

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
    db: State<AppDatabase>,
) -> AppResult<()> {
    let game = db
        .get_game_by_id(&id)?
        .ok_or_else(|| AppError::NotFound("Game not found".into()))?;

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    let steam_app_id = game
        .steam_app_id
        .clone()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    let steam_app_id: Option<String> = None;

    let settings = state.settings.lock().clone();

    let child: Option<std::process::Child> = if let Some(app_id) = steam_app_id {
        write_steam_appid_file(&game.path, &app_id);
        spawn_steam_game(&app_id)?;
        None
    } else {
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

        Some(spawn_game_process(&game, &path, &settings)?)
    };

    let game_title = game.title.clone();
    let cover_url = game.cover_url.clone();
    let discord_start = discord::get_unix_timestamp();

    let start_time = Instant::now();
    {
        let mut running = state.running_game.lock();
        *running = Some(RunningGame {
            id: id.clone(),
            start_time,
        });
    }

    if settings.discord_rpc_enabled {
        let custom_state = game
            .discord_status
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("Idle");

        let vndb_game_url = game
            .vndb_id
            .as_ref()
            .map(|id| format!("https://vndb.org/{}", id));
        let vndb_profile_url = settings
            .vndb_user_id
            .as_ref()
            .map(|id| format!("https://vndb.org/{}", id));
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

        log::info!(
            "Discord RPC buttons: {:?}",
            button_refs.iter().map(|(l, _)| *l).collect::<Vec<_>>()
        );

        let _ = state.discord_rpc.set_activity(
            &game_title,
            cover_url.as_deref(),
            custom_state,
            button_refs,
            discord_start,
        );
    }

    let app_handle_clone = app_handle.clone();
    let game_id = id.clone();
    let launched_via_steam = child.is_none();
    let watcher_handle = app_handle.clone();

    if let Some(mut child) = child {
        tauri::async_runtime::spawn(async move {
            let exit_result = task::spawn_blocking(move || child.wait()).await;

            let minutes = start_time.elapsed().as_secs() / 60;
            let state = app_handle_clone.state::<AppState>();

            if let Err(e) = state.discord_rpc.clear_activity() {
                log::warn!("Failed to clear Discord activity: {}", e);
            }
            let seconds = minutes.saturating_mul(60).min(i64::MAX as u64) as i64;
            persist_session(&app_handle_clone, &game_id, seconds);

            {
                let mut running = state.running_game.lock();
                *running = None;
            }

            if let Err(e) = app_handle.emit(
                "game-exited",
                GameExitedPayload {
                    game_id: game_id.clone(),
                    play_minutes: minutes,
                },
            ) {
                log::error!("Failed to emit game-exited event: {}", e);
            }

            if let Ok(Err(e)) = exit_result {
                eprintln!("Game process error: {}", e);
            }
        });
    }
    if launched_via_steam {
        spawn_steam_watcher(watcher_handle, id.clone(), game.path.clone());
    }

    Ok(())
}

fn write_steam_appid_file(exe_path: &str, app_id: &str) {
    let Some(dir) = Path::new(exe_path).parent() else {
        return;
    };
    if dir.as_os_str().is_empty() {
        return;
    }
    let target = dir.join("steam_appid.txt");
    if let Err(e) = std::fs::write(&target, app_id) {
        log::warn!("Failed to write {}: {e}", target.display());
    }
}

fn steam_protocol_url(app_id: &str) -> String {
    format!("steam://rungameid/{app_id}")
}

#[cfg(target_os = "windows")]
const STEAM_EXE_CANDIDATES: &[&str] = &[
    "C:\\Program Files (x86)\\Steam\\steam.exe",
    "C:\\Program Files\\Steam\\steam.exe",
];

#[cfg(target_os = "windows")]
fn spawn_steam_game(app_id: &str) -> AppResult<()> {
    let url = steam_protocol_url(app_id);
    if Command::new("cmd")
        .args(["/C", "start", "", url.as_str()])
        .spawn()
        .is_ok()
    {
        log::info!("Launched via Steam protocol: {url}");
        return Ok(());
    }
    for candidate in STEAM_EXE_CANDIDATES {
        if Command::new(candidate).arg(&url).spawn().is_ok() {
            log::info!("Launched via Steam client {candidate}: {url}");
            return Ok(());
        }
    }
    Err(AppError::ProcessLaunch(format!(
        "Failed to launch Steam game {app_id}: Steam client not found"
    )))
}

#[cfg(not(target_os = "windows"))]
fn spawn_steam_game(app_id: &str) -> AppResult<()> {
    let url = steam_protocol_url(app_id);
    match Command::new("steam").arg(&url).spawn() {
        Ok(_) => {
            log::info!("Launched via Steam protocol: {url}");
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .map(|_| ())
            .map_err(|e| {
                AppError::ProcessLaunch(format!("Failed to launch Steam game {app_id}: {e}"))
            }),
        Err(e) => Err(AppError::ProcessLaunch(format!(
            "Failed to launch Steam game {app_id}: {e}"
        ))),
    }
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
                "No Wine installation found. Please install Wine or configure Wine settings."
                    .into(),
            )
        })?;

    let prefix_path = if wine_settings.use_global_prefix {
        app_settings.default_wine_prefix.clone().unwrap_or_else(|| {
            wine::get_global_prefix_path()
                .to_str()
                .unwrap_or("")
                .to_string()
        })
    } else {
        wine_settings.wine_prefix.clone().unwrap_or_else(|| {
            wine::get_default_prefix_path(&game.id, game.vndb_id.as_deref())
                .to_str()
                .unwrap_or("")
                .to_string()
        })
    };

    let prefix_dir = Path::new(&prefix_path);
    wine::ensure_prefix_exists(prefix_dir).map_err(AppError::ProcessLaunch)?;

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
        WineType::Wine
        | WineType::WineGE
        | WineType::WineStaging
        | WineType::WineTKG
        | WineType::Lutris
        | WineType::Bottles
        | WineType::Custom => build_wine_command(
            &wine_binary,
            &prefix_path,
            path,
            use_steam_runtime,
            &wine_settings,
        )?,
        WineType::Proton | WineType::ProtonGE | WineType::ProtonCachyOS | WineType::ProtonTKG => {
            build_proton_command(
                &wine_binary,
                &prefix_path,
                path,
                use_steam_runtime,
                &wine_settings,
            )?
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
        _ => AppError::ProcessLaunch(format!("Failed to launch game: {} ({})", e, path.display())),
    }
}

// ============================================================================
// Other Commands
// ============================================================================

#[tauri::command]
#[specta::specta]
pub fn stop_tracking(state: State<AppState>, db: State<AppDatabase>) -> AppResult<u64> {
    let mut running = state.running_game.lock();
    if let Some(game) = running.take() {
        let elapsed = game.start_time.elapsed();
        let minutes = elapsed.as_secs() / 60;
        let game_id = game.id.clone();

        let seconds = minutes.saturating_mul(60).min(i64::MAX as u64) as i64;
        db.add_playtime(&game_id, seconds)?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::ErrorKind;

    mod map_spawn_error_tests {
        use super::*;

        #[test]
        fn test_not_found_error() {
            let err = std::io::Error::new(ErrorKind::NotFound, "file not found");
            let path = Path::new("/path/to/game.exe");
            let app_err = map_spawn_error(err, path);

            match app_err {
                AppError::ProcessLaunch(msg) => {
                    assert!(msg.contains("not found"));
                    assert!(msg.contains("/path/to/game.exe"));
                }
                _ => panic!("Expected ProcessLaunch error, got {:?}", app_err),
            }
        }

        #[test]
        fn test_permission_denied_error() {
            let err = std::io::Error::new(ErrorKind::PermissionDenied, "access denied");
            let path = Path::new("/path/to/game.exe");
            let app_err = map_spawn_error(err, path);

            match app_err {
                AppError::ProcessLaunch(msg) => {
                    assert!(msg.contains("Permission denied"));
                    assert!(msg.contains("/path/to/game.exe"));
                }
                _ => panic!("Expected ProcessLaunch error, got {:?}", app_err),
            }
        }

        #[test]
        fn test_other_io_error() {
            let err = std::io::Error::new(ErrorKind::BrokenPipe, "broken pipe");
            let path = Path::new("/path/to/game.exe");
            let app_err = map_spawn_error(err, path);

            match app_err {
                AppError::ProcessLaunch(msg) => {
                    assert!(msg.contains("Failed to launch game"));
                    assert!(msg.contains("/path/to/game.exe"));
                }
                _ => panic!("Expected ProcessLaunch error, got {:?}", app_err),
            }
        }

        #[test]
        fn test_path_with_spaces() {
            let err = std::io::Error::new(ErrorKind::NotFound, "file not found");
            let path = Path::new("/path/to/My Game/game.exe");
            let app_err = map_spawn_error(err, path);

            match app_err {
                AppError::ProcessLaunch(msg) => {
                    assert!(msg.contains("My Game"));
                }
                _ => panic!("Expected ProcessLaunch error"),
            }
        }
    }

    #[cfg(target_os = "linux")]
    mod resolve_wine_settings_tests {
        use super::*;
        use std::collections::HashMap;

        fn create_test_settings() -> AppSettings {
            AppSettings {
                blur_nsfw: true,
                discord_rpc_enabled: true,
                discord_btn_vndb_game: true,
                discord_btn_vndb_profile: false,
                discord_btn_github: false,
                default_wine_binary: Some("/usr/bin/wine".to_string()),
                default_wine_prefix: Some("/home/user/.wine".to_string()),
                use_steam_runtime: true,
                vndb_token: None,
                vndb_user_id: None,
            }
        }

        fn create_test_game(wine_settings: Option<WineSettings>) -> GameMetadata {
            GameMetadata {
                id: "test-game".to_string(),
                title: "Test Game".to_string(),
                vndb_id: Some("v12345".to_string()),
                steam_app_id: None,
                discord_status: None,
                cover_url: None,
                path: "/path/to/game.exe".to_string(),
                play_time: 0,
                last_played: None,
                game_type: Some(GameType::WindowsExe),
                wine_settings,
                is_finished: false,
                is_hidden: false,
                show_spoilers: false,
            }
        }

        #[test]
        fn test_uses_game_specific_settings() {
            let app_settings = create_test_settings();
            let game_wine_settings = WineSettings {
                use_global_prefix: false,
                wine_prefix: Some("/custom/prefix".to_string()),
                wine_version: Some("/custom/wine".to_string()),
                wine_type: Some(WineType::ProtonGE),
                use_steam_runtime: false,
                env_vars: HashMap::new(),
            };
            let game = create_test_game(Some(game_wine_settings.clone()));

            let result = resolve_wine_settings(&game, &app_settings);

            assert_eq!(result.wine_prefix, Some("/custom/prefix".to_string()));
            assert_eq!(result.wine_version, Some("/custom/wine".to_string()));
            assert!(!result.use_steam_runtime);
        }

        #[test]
        fn test_falls_back_to_app_settings() {
            let app_settings = create_test_settings();
            let game = create_test_game(None);

            let result = resolve_wine_settings(&game, &app_settings);

            assert!(result.use_global_prefix);
            assert_eq!(result.wine_prefix, Some("/home/user/.wine".to_string()));
            assert_eq!(result.wine_version, Some("/usr/bin/wine".to_string()));
            assert!(result.use_steam_runtime);
        }

        #[test]
        fn test_fallback_has_empty_env_vars() {
            let app_settings = create_test_settings();
            let game = create_test_game(None);

            let result = resolve_wine_settings(&game, &app_settings);

            assert!(result.env_vars.is_empty());
        }
    }

    #[test]
    fn test_steam_protocol_url() {
        assert_eq!(steam_protocol_url("412830"), "steam://rungameid/412830");
    }
}
