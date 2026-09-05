use std::path::Path;
use std::time::{Duration, Instant};

use sysinfo::{ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

use crate::database::{record_daily_playtime, AppDatabase};
use crate::models::GameExitedPayload;
use crate::state::AppState;

const WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn_steam_watcher(app_handle: AppHandle, game_id: String, exe_path: String) {
    tauri::async_runtime::spawn(async move {
        let Some(binary) = binary_file_name(&exe_path) else {
            log::warn!("Steam watcher aborted, cannot read binary name from: {exe_path}");
            return;
        };
        if !wait_for_process(&binary).await {
            log::warn!("Steam watcher timed out waiting for process: {binary}");
            if release_running(&app_handle, &game_id) {
                clear_presence(&app_handle);
                emit_exited(&app_handle, &game_id, 0);
            }
            return;
        }
        let started = Instant::now();
        watch_until_exit(&binary).await;
        let seconds = started.elapsed().as_secs().min(i64::MAX as u64) as i64;
        persist_session(&app_handle, &game_id, seconds);
        if release_running(&app_handle, &game_id) {
            clear_presence(&app_handle);
            emit_exited(&app_handle, &game_id, seconds.max(0) as u64 / 60);
        }
    });
}

pub(crate) fn persist_session(app_handle: &AppHandle, game_id: &str, seconds: i64) {
    let seconds = seconds.max(0);
    let minutes = seconds as u64 / 60;
    let db = app_handle.state::<AppDatabase>();
    if let Err(e) = db.add_playtime(game_id, seconds) {
        log::error!("Failed to save game playtime: {e}");
    }
    record_daily_playtime(game_id, minutes);
}

pub(crate) fn release_running(app_handle: &AppHandle, game_id: &str) -> bool {
    let state = app_handle.state::<AppState>();
    let mut running = state.running_game.lock();
    if running.as_ref().is_some_and(|game| game.id == game_id) {
        *running = None;
        true
    } else {
        false
    }
}

fn clear_presence(app_handle: &AppHandle) {
    let state = app_handle.state::<AppState>();
    if let Err(e) = state.discord_rpc.clear_activity() {
        log::warn!("Failed to clear Discord activity: {e}");
    }
}

fn emit_exited(app_handle: &AppHandle, game_id: &str, play_minutes: u64) {
    if let Err(e) = app_handle.emit(
        "game-exited",
        GameExitedPayload {
            game_id: game_id.to_string(),
            play_minutes,
        },
    ) {
        log::error!("Failed to emit game-exited event: {e}");
    }
}

async fn wait_for_process(binary: &str) -> bool {
    let deadline = Instant::now() + WAIT_TIMEOUT;
    while Instant::now() < deadline {
        if is_process_running(binary).await {
            return true;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    false
}

async fn watch_until_exit(binary: &str) {
    while is_process_running(binary).await {
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn is_process_running(binary: &str) -> bool {
    let binary = binary.to_string();
    tokio::task::spawn_blocking(move || process_running(&binary))
        .await
        .unwrap_or(false)
}

fn process_running(binary: &str) -> bool {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, true);
    system
        .processes()
        .values()
        .any(|process| process_matches(&process.name().to_string_lossy(), binary))
}

fn binary_file_name(exe_path: &str) -> Option<String> {
    Path::new(exe_path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
}

fn truncated_comm(binary: &str) -> &str {
    let mut end = binary.len().min(15);
    while !binary.is_char_boundary(end) {
        end -= 1;
    }
    &binary[..end]
}

fn process_matches(process_name: &str, binary: &str) -> bool {
    process_name == binary || process_name == truncated_comm(binary)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_file_name() {
        assert_eq!(
            binary_file_name("/home/user/games/game.exe"),
            Some("game.exe".to_string())
        );
        assert_eq!(binary_file_name("game.exe"), Some("game.exe".to_string()));
        assert_eq!(binary_file_name(""), None);
        assert_eq!(
            binary_file_name("/home/user/games/"),
            Some("games".to_string())
        );
    }

    #[test]
    fn test_process_matches_exact_and_truncated() {
        assert!(process_matches("game.exe", "game.exe"));
        assert!(!process_matches("other.exe", "game.exe"));
        assert!(process_matches(
            "abcdefghijklmno",
            "abcdefghijklmnopqrstuvwxyz.exe"
        ));
        assert!(!process_matches(
            "abcdefghijklmnX",
            "abcdefghijklmnopqrstuvwxyz.exe"
        ));
    }

    #[test]
    fn test_truncated_comm_respects_char_boundary() {
        assert_eq!(truncated_comm("short.exe"), "short.exe");
        assert_eq!(truncated_comm("1234567890123456"), "123456789012345");
        let wide = "éééééééé.exe";
        let cut = truncated_comm(wide);
        assert!(wide.starts_with(cut));
        assert!(cut.len() <= 15);
    }

    #[tokio::test]
    async fn test_absent_process_reports_not_running() {
        assert!(!is_process_running("poketto-definitely-not-running-9f8c").await);
    }
}
