use std::path::{Path, PathBuf};

const STEAM_RUN_PATHS: &[&str] = &["/usr/bin/steam-run", "/run/current-system/sw/bin/steam-run"];

pub(super) fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

pub(super) fn parse_steam_library_folders() -> Vec<PathBuf> {
    let mut libraries = Vec::new();

    libraries.push(PathBuf::from("/usr/share/steam"));

    if let Some(home) = dirs::home_dir() {
        let steam_paths = [
            home.join(".steam/steam"),
            home.join(".local/share/Steam"),
            home.join(".var/app/com.valvesoftware.Steam/.steam/steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ];

        for steam_path in steam_paths {
            let vdf_path = steam_path.join("steamapps/libraryfolders.vdf");

            if vdf_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&vdf_path) {
                    for line in content.lines() {
                        let line = line.trim();
                        if line.starts_with("\"path\"") {
                            if let Some(path_str) = extract_vdf_value(line) {
                                let lib_path = PathBuf::from(path_str);
                                if lib_path.exists() && !libraries.contains(&lib_path) {
                                    libraries.push(lib_path);
                                }
                            }
                        }
                    }
                }
                break;
            }
        }
    }

    libraries
}

pub(super) fn get_steam_compat_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".steam/root/compatibilitytools.d"));
        paths.push(home.join(".local/share/Steam/compatibilitytools.d"));
        paths.push(home.join(".steam/steam/compatibilitytools.d"));

        paths.push(home.join(".var/app/com.valvesoftware.Steam/.steam/root/compatibilitytools.d"));
        paths.push(
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/compatibilitytools.d"),
        );
    }

    paths.push(PathBuf::from("/usr/share/steam/compatibilitytools.d"));

    for lib_path in parse_steam_library_folders() {
        paths.push(lib_path.join("compatibilitytools.d"));
        paths.push(lib_path.join("steam/compatibilitytools.d"));
    }

    paths.into_iter().filter(|p| p.exists()).collect()
}

pub(super) fn get_steam_proton_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".steam/root/steamapps/common"));
        paths.push(home.join(".local/share/Steam/steamapps/common"));
        paths.push(home.join(".steam/steam/steamapps/common"));

        paths.push(home.join(".var/app/com.valvesoftware.Steam/.steam/root/steamapps/common"));
        paths.push(
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam/steamapps/common"),
        );
    }

    for lib_path in parse_steam_library_folders() {
        paths.push(lib_path.join("steamapps/common"));
        paths.push(lib_path.join("steam/steamapps/common"));
    }

    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn is_steam_runtime_available() -> bool {
    STEAM_RUN_PATHS.iter().any(|path| Path::new(path).exists())
}

pub fn get_steam_run_path() -> Option<String> {
    STEAM_RUN_PATHS
        .iter()
        .find(|path| Path::new(path).exists())
        .map(|s| s.to_string())
}

pub fn get_steam_path() -> Option<PathBuf> {
    let possible_paths = [
        dirs::home_dir().map(|h| h.join(".steam/steam")),
        dirs::home_dir().map(|h| h.join(".local/share/Steam")),
        dirs::home_dir().map(|h| h.join(".var/app/com.valvesoftware.Steam/.steam/steam")),
        Some(PathBuf::from("/usr/share/steam")),
    ];

    for path_opt in possible_paths.iter().flatten() {
        if path_opt.exists() {
            return Some(path_opt.clone());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    mod extract_vdf_value_tests {
        use super::*;

        #[test]
        fn test_valid_vdf_line() {
            let line = r#"		"path"		"/home/user/.steam/steam""#;
            assert_eq!(
                extract_vdf_value(line),
                Some("/home/user/.steam/steam".to_string())
            );
        }

        #[test]
        fn test_vdf_line_with_different_path() {
            let line = r#"		"path"		"/mnt/games/Steam""#;
            assert_eq!(
                extract_vdf_value(line),
                Some("/mnt/games/Steam".to_string())
            );
        }

        #[test]
        fn test_vdf_line_with_spaces_in_path() {
            let line = r#"		"path"		"/home/user/My Games/Steam""#;
            assert_eq!(
                extract_vdf_value(line),
                Some("/home/user/My Games/Steam".to_string())
            );
        }

        #[test]
        fn test_malformed_no_quotes() {
            assert_eq!(extract_vdf_value("no quotes here"), None);
        }

        #[test]
        fn test_malformed_only_two_quotes() {
            assert_eq!(extract_vdf_value(r#""only" two"#), None);
        }

        #[test]
        fn test_malformed_only_three_parts() {
            assert_eq!(extract_vdf_value(r#""key""#), None);
        }

        #[test]
        fn test_four_parts_returns_value() {
            assert_eq!(
                extract_vdf_value(r#""key" "value""#),
                Some("value".to_string())
            );
        }

        #[test]
        fn test_empty_value() {
            let line = r#"		"path"		"""#;
            assert_eq!(extract_vdf_value(line), Some("".to_string()));
        }

        #[test]
        fn test_empty_line() {
            assert_eq!(extract_vdf_value(""), None);
        }
    }
}
