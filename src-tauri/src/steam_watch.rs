use std::path::Path;
use std::time::{Duration, Instant};

use sysinfo::{Pid, ProcessesToUpdate, System};
use tauri::{AppHandle, Emitter, Manager};

use crate::database::{record_daily_playtime, AppDatabase};
use crate::models::GameExitedPayload;
use crate::state::{AppState, Settle};

const WAIT_TIMEOUT: Duration = Duration::from_secs(60);
const POLL_INTERVAL: Duration = Duration::from_secs(2);

pub fn spawn_steam_watcher(
    app_handle: AppHandle,
    game_id: String,
    exe_path: String,
    start_time: Instant,
) {
    tauri::async_runtime::spawn(async move {
        let Some(binary) = binary_file_name(&exe_path) else {
            log::warn!("Steam watcher aborted, cannot read binary name from: {exe_path}");
            return;
        };
        let mut system = System::new();
        if !wait_for_process(&mut system, &binary).await {
            log::warn!("Steam watcher timed out waiting for process: {binary}");
            if app_handle.state::<AppState>().settle_running(&game_id, start_time) == Settle::Mine {
                clear_presence(&app_handle);
                emit_exited(&app_handle, &game_id, 0);
            }
            return;
        }
        let started = Instant::now();
        watch_until_exit(&mut system, &binary).await;
        let seconds = started.elapsed().as_secs().min(i64::MAX as u64) as i64;
        match app_handle.state::<AppState>().settle_running(&game_id, start_time) {
            Settle::Mine => {
                persist_session(&app_handle, &game_id, seconds);
                clear_presence(&app_handle);
                emit_exited(&app_handle, &game_id, seconds.max(0) as u64 / 60);
            }
            Settle::Replaced => {
                persist_session(&app_handle, &game_id, seconds);
            }
            Settle::Gone => {}
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

pub(crate) fn kill_session_processes(pid: Option<u32>, binary: Option<&str>) {
    let mut system = System::new();
    system.refresh_processes(ProcessesToUpdate::All, false);
    if let Some(pid) = pid {
        kill_process_tree(&system, Pid::from_u32(pid));
    }
    if let Some(binary) = binary {
        for process in system.processes().values() {
            if process_matches(&process.name().to_string_lossy(), binary) {
                process.kill();
            }
        }
    }
}

fn kill_process_tree(system: &System, root: Pid) {
    let processes = system.processes();
    if !processes.contains_key(&root) {
        return;
    }
    let mut order = vec![root];
    let mut index = 0;
    while index < order.len() {
        let parent = order[index];
        index += 1;
        for (pid, process) in processes.iter() {
            if process.parent() == Some(parent) && !order.contains(pid) {
                order.push(*pid);
            }
        }
    }
    for pid in order.iter().rev() {
        if let Some(process) = processes.get(pid) {
            process.kill();
        }
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

async fn wait_for_process(system: &mut System, binary: &str) -> bool {
    let deadline = Instant::now() + WAIT_TIMEOUT;
    while Instant::now() < deadline {
        if refresh_and_match(system, binary).await {
            return true;
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }
    false
}

async fn watch_until_exit(system: &mut System, binary: &str) {
    while refresh_and_match(system, binary).await {
        tokio::time::sleep(POLL_INTERVAL).await;
    }
}

async fn refresh_and_match(system: &mut System, binary: &str) -> bool {
    let mut owned = std::mem::replace(system, System::new());
    let binary = binary.to_string();
    let (back, running) = tokio::task::spawn_blocking(move || {
        owned.refresh_processes(ProcessesToUpdate::All, true);
        let running = owned
            .processes()
            .values()
            .any(|process| process_matches(&process.name().to_string_lossy(), &binary));
        (owned, running)
    })
    .await
    .unwrap_or_else(|_| (System::new(), false));
    *system = back;
    running
}

pub(crate) fn binary_file_name(exe_path: &str) -> Option<String> {
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

pub(crate) fn process_matches(process_name: &str, binary: &str) -> bool {
    let process_name = process_name.to_lowercase();
    let binary = binary.to_lowercase();
    process_name == binary || linux_comm_matches(&process_name, &binary)
}

#[cfg(target_os = "linux")]
fn linux_comm_matches(process_name: &str, binary: &str) -> bool {
    process_name == truncated_comm(binary)
}

#[cfg(not(target_os = "linux"))]
fn linux_comm_matches(_process_name: &str, _binary: &str) -> bool {
    false
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
    fn test_process_matches_case_insensitive() {
        assert!(process_matches("Game.EXE", "game.exe"));
        assert!(process_matches("game.exe", "GAME.EXE"));
        assert!(!process_matches("other.exe", "game.exe"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_process_matches_truncated_comm() {
        assert!(process_matches(
            "abcdefghijklmno",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ.EXE"
        ));
        assert!(!process_matches(
            "abcdefghijklmnX",
            "abcdefghijklmnopqrstuvwxyz.exe"
        ));
    }

    #[cfg(not(target_os = "linux"))]
    #[test]
    fn test_process_requires_exact_name() {
        assert!(!process_matches(
            "abcdefghijklmno",
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
        let mut system = System::new();
        assert!(!refresh_and_match(&mut system, "poketto-definitely-not-running-9f8c").await);
    }

    #[cfg(unix)]
    #[test]
    fn test_kill_process_tree_terminates_descendants() {
        use std::process::Command;

        let mut child = Command::new("sh")
            .args(["-c", "sleep 60"])
            .spawn()
            .expect("spawn sleeper");
        let mut system = System::new();
        system.refresh_processes(ProcessesToUpdate::All, false);
        kill_process_tree(&system, Pid::from_u32(child.id()));
        let deadline = Instant::now() + Duration::from_secs(10);
        let exited = loop {
            if Instant::now() >= deadline {
                break false;
            }
            match child.try_wait() {
                Ok(Some(_)) => break true,
                Ok(None) => std::thread::sleep(Duration::from_millis(100)),
                Err(_) => break false,
            }
        };
        let _ = child.kill();
        let _ = child.wait();
        assert!(exited, "process tree survived kill");
    }
}
