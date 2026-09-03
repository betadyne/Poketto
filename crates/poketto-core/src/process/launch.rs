use std::collections::HashMap;
use std::path::Path;

use super::error::{ProcessError, ProcessResult};
use crate::models::{AppSettings, Game, GameType, WineSettings, WineType};

pub fn resolve_game_type(game: &Game, path: &Path) -> GameType {
    game.game_type.clone().unwrap_or_else(|| {
        if path.extension().and_then(|e| e.to_str()) == Some("exe") {
            GameType::WindowsExe
        } else {
            GameType::LinuxNative
        }
    })
}

pub fn resolve_wine_settings(game: &Game, settings: &AppSettings) -> WineSettings {
    game.wine_settings.clone().unwrap_or_else(|| WineSettings {
        use_global_prefix: true,
        wine_prefix: settings.default_wine_prefix.clone(),
        wine_version: settings.default_wine_binary.clone(),
        wine_type: None,
        use_steam_runtime: settings.use_steam_runtime,
        env_vars: HashMap::new(),
    })
}

pub fn validate_executable(path: &Path) -> ProcessResult<()> {
    if !path.exists() {
        return Err(ProcessError::NotFound(path.display().to_string()));
    }
    if !path.is_file() {
        return Err(ProcessError::NotAFile(path.display().to_string()));
    }
    Ok(())
}

pub fn build_command(
    game: &Game,
    settings: &AppSettings,
) -> ProcessResult<tokio::process::Command> {
    let path = Path::new(&game.path);
    validate_executable(path)?;

    #[cfg(target_os = "linux")]
    {
        match resolve_game_type(game, path) {
            GameType::LinuxNative => Ok(native_command(path)),
            GameType::WindowsExe => wine_command(game, path, settings),
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        let _ = (game, settings);
        Ok(native_command(path))
    }
}

fn native_command(path: &Path) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(path);
    cmd.current_dir(path.parent().unwrap_or(path));
    cmd
}

#[cfg(target_os = "linux")]
fn wine_command(
    game: &Game,
    path: &Path,
    settings: &AppSettings,
) -> ProcessResult<tokio::process::Command> {
    use crate::wine;

    let wine_settings = resolve_wine_settings(game, settings);

    let wine_binary = wine_settings
        .wine_version
        .clone()
        .or_else(|| settings.default_wine_binary.clone())
        .or_else(|| wine::get_default_wine().map(|w| w.binary_path))
        .ok_or(ProcessError::NoWine)?;

    let prefix_path = if wine_settings.use_global_prefix {
        settings.default_wine_prefix.clone().unwrap_or_else(|| {
            wine::global_prefix_path()
                .to_str()
                .unwrap_or("")
                .to_string()
        })
    } else {
        wine_settings.wine_prefix.clone().unwrap_or_else(|| {
            wine::default_prefix_path(&game.id)
                .to_str()
                .unwrap_or("")
                .to_string()
        })
    };

    wine::ensure_prefix_exists(Path::new(&prefix_path))
        .map_err(|e| ProcessError::LaunchFailed(e.to_string()))?;

    let wine_type = wine_settings.wine_type.clone().unwrap_or(WineType::Wine);
    let use_steam_runtime = wine_settings.use_steam_runtime || settings.use_steam_runtime;

    match wine_type {
        WineType::Wine
        | WineType::WineGE
        | WineType::WineStaging
        | WineType::WineTKG
        | WineType::Lutris
        | WineType::Bottles
        | WineType::Custom => Ok(wine_runner_command(
            &wine_binary,
            &prefix_path,
            path,
            use_steam_runtime,
            &wine_settings.env_vars,
            None,
        )),
        WineType::Proton | WineType::ProtonGE | WineType::ProtonCachyOS | WineType::ProtonTKG => {
            Ok(wine_runner_command(
                &wine_binary,
                &prefix_path,
                path,
                use_steam_runtime,
                &wine_settings.env_vars,
                Some(proton_env(&prefix_path)),
            ))
        }
    }
}

#[cfg(target_os = "linux")]
fn proton_env(prefix_path: &str) -> Vec<(String, String)> {
    let steam_path = crate::wine::get_steam_path()
        .map(|p| p.to_str().unwrap_or("").to_string())
        .unwrap_or_default();
    vec![
        ("STEAM_COMPAT_DATA_PATH".to_string(), prefix_path.to_string()),
        ("STEAM_COMPAT_CLIENT_INSTALL_PATH".to_string(), steam_path),
    ]
}

#[cfg(target_os = "linux")]
fn wine_runner_command(
    runner_binary: &str,
    prefix_path: &str,
    game_path: &Path,
    use_steam_runtime: bool,
    extra_env: &HashMap<String, String>,
    proton_env: Option<Vec<(String, String)>>,
) -> tokio::process::Command {
    use crate::wine;

    let mut cmd = if use_steam_runtime && wine::is_steam_runtime_available() {
        if let Some(steam_run) = wine::get_steam_run_path() {
            let mut runner = tokio::process::Command::new(&steam_run);
            runner.arg(runner_binary);
            runner
        } else {
            tokio::process::Command::new(runner_binary)
        }
    } else {
        tokio::process::Command::new(runner_binary)
    };

    match proton_env {
        Some(env) => {
            for (key, value) in env {
                cmd.env(key, value);
            }
            cmd.current_dir(game_path.parent().unwrap_or(game_path));
            cmd.arg("run");
            cmd.arg(game_path);
        }
        None => {
            cmd.env("WINEPREFIX", prefix_path);
            cmd.env("WINEDEBUG", "-all");
            cmd.current_dir(game_path.parent().unwrap_or(game_path));
            cmd.arg(game_path);
        }
    }

    for (key, value) in extra_env {
        cmd.env(key, value);
    }

    cmd
}

pub fn spawn(
    mut cmd: tokio::process::Command,
    path: &Path,
) -> ProcessResult<tokio::process::Child> {
    use std::io::ErrorKind;
    tracing::info!(path = %path.display(), "spawning game process");
    cmd.spawn().map_err(|e| match e.kind() {
        ErrorKind::NotFound => {
            ProcessError::NotFound(format!("executable not found: {}", path.display()))
        }
        ErrorKind::PermissionDenied => {
            ProcessError::PermissionDenied(format!("cannot execute {}", path.display()))
        }
        _ => ProcessError::LaunchFailed(format!("{} ({})", e, path.display())),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game_with(path: &str, game_type: Option<GameType>) -> Game {
        Game {
            id: "g1".to_string(),
            title: "Test".to_string(),
            path: path.to_string(),
            vndb_id: None,
            cover_url: None,
            play_time_minutes: 0,
            is_finished: false,
            last_played: None,
            is_hidden: false,
            show_spoilers: false,
            game_type,
            wine_settings: None,
            rating: None,
        }
    }

    #[test]
    fn explicit_game_type_wins() {
        let game = game_with("/games/a.x86_64", Some(GameType::WindowsExe));
        assert_eq!(
            resolve_game_type(&game, Path::new(&game.path)),
            GameType::WindowsExe
        );
    }

    #[test]
    fn exe_extension_defaults_to_wine() {
        let game = game_with("/games/a.exe", None);
        assert_eq!(
            resolve_game_type(&game, Path::new(&game.path)),
            GameType::WindowsExe
        );
    }
    fn other_extension_defaults_to_native() {
        let game = game_with("/games/a.x86_64", None);
        assert_eq!(
            resolve_game_type(&game, Path::new(&game.path)),
            GameType::LinuxNative
        );
    }

    #[test]
    fn wine_settings_fall_back_to_app_defaults() {
        let game = game_with("/games/a.exe", None);
        let settings = AppSettings {
            default_wine_binary: Some("/usr/bin/wine".to_string()),
            use_steam_runtime: true,
            ..AppSettings::default()
        };
        let resolved = resolve_wine_settings(&game, &settings);
        assert_eq!(resolved.use_global_prefix, true);
        assert_eq!(
            resolved.wine_version.as_deref(),
            Some("/usr/bin/wine")
        );
        assert_eq!(resolved.use_steam_runtime, true);
    }

    #[test]
    fn missing_executable_fails_validation() {
        assert!(matches!(
            validate_executable(Path::new("/nonexistent/game")),
            Err(ProcessError::NotFound(_))
        ));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn native_command_targets_game_dir() {
        let exe = std::env::current_exe().expect("current exe");
        let game = game_with(exe.to_str().expect("unicode"), Some(GameType::LinuxNative));
        let cmd = build_command(&game, &AppSettings::default()).expect("build");
        let std_cmd = cmd.as_std();
        assert_eq!(std_cmd.get_program(), exe.as_os_str());
        assert_eq!(std_cmd.get_current_dir(), exe.parent());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wine_command_sets_prefix_env() {
        let exe = std::env::current_exe().expect("current exe");
        let mut game = game_with(exe.to_str().expect("unicode"), Some(GameType::WindowsExe));
        game.wine_settings = Some(WineSettings {
            wine_version: Some("/bin/true".to_string()),
            wine_prefix: Some("/tmp/poketto-test-prefix".to_string()),
            use_global_prefix: false,
            ..WineSettings::default()
        });
        let cmd = build_command(&game, &AppSettings::default()).expect("build");
        let std_cmd = cmd.as_std();
        let env: Vec<(String, String)> = std_cmd
            .get_envs()
            .filter_map(|(k, v)| Some((k.to_str()?.to_string(), v?.to_str()?.to_string())))
            .collect();
        assert!(env.contains(&(
            "WINEPREFIX".to_string(),
            "/tmp/poketto-test-prefix".to_string()
        )));
        assert!(env.contains(&("WINEDEBUG".to_string(), "-all".to_string())));
        let _ = std::fs::remove_dir_all("/tmp/poketto-test-prefix");
    }
}
