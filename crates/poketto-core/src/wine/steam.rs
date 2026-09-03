use std::path::{Path, PathBuf};

const STEAM_RUN_PATHS: &[&str] = &["/usr/bin/steam-run", "/run/current-system/sw/bin/steam-run"];

fn extract_vdf_value(line: &str) -> Option<String> {
    let parts: Vec<&str> = line.split('"').collect();
    if parts.len() >= 4 {
        Some(parts[3].to_string())
    } else {
        None
    }
}

pub fn library_paths_from_vdf(content: &str) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("\"path\"") {
            if let Some(path) = extract_vdf_value(line) {
                let path = PathBuf::from(path);
                if !paths.contains(&path) {
                    paths.push(path);
                }
            }
        }
    }
    paths
}

pub fn steam_library_folders() -> Vec<PathBuf> {
    let mut libraries = vec![PathBuf::from("/usr/share/steam")];

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
                    for lib_path in library_paths_from_vdf(&content) {
                        if lib_path.exists() && !libraries.contains(&lib_path) {
                            libraries.push(lib_path);
                        }
                    }
                }
                break;
            }
        }
    }

    libraries
}

pub fn steam_compat_paths() -> Vec<PathBuf> {
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

    for lib_path in steam_library_folders() {
        paths.push(lib_path.join("compatibilitytools.d"));
        paths.push(lib_path.join("steam/compatibilitytools.d"));
    }

    paths.into_iter().filter(|p| p.exists()).collect()
}

pub fn steam_proton_paths() -> Vec<PathBuf> {
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

    for lib_path in steam_library_folders() {
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

    #[test]
    fn vdf_value_extracts_fourth_segment() {
        assert_eq!(
            extract_vdf_value("\t\t\"path\"\t\t\"/home/user/.steam/steam\""),
            Some("/home/user/.steam/steam".to_string())
        );
        assert_eq!(
            extract_vdf_value("\t\t\"path\"\t\t\"/home/user/My Games/Steam\""),
            Some("/home/user/My Games/Steam".to_string())
        );
        assert_eq!(extract_vdf_value("no quotes here"), None);
        assert_eq!(extract_vdf_value("\"only\" two"), None);
        assert_eq!(extract_vdf_value("\"key\""), None);
        assert_eq!(extract_vdf_value(""), None);
    }

    #[test]
    fn vdf_parses_multiple_libraries() {
        let content = "\"libraryfolders\"\n{\n\t\"0\"\n\t{\n\t\t\"path\"\t\t\"/home/user/.steam/steam\"\n\t}\n\t\"1\"\n\t{\n\t\t\"path\"\t\t\"/mnt/games/Steam\"\n\t\t\"label\"\t\t\"\"\n\t}\n\t\"garbage without quotes\"\n}\n";
        assert_eq!(
            library_paths_from_vdf(content),
            vec![
                PathBuf::from("/home/user/.steam/steam"),
                PathBuf::from("/mnt/games/Steam"),
            ]
        );
    }

    #[test]
    fn vdf_ignores_duplicates_and_non_path_keys() {
        let content = "\"path\"\t\t\"/a\"\n\"path\"\t\t\"/a\"\n\"label\"\t\t\"/b\"\n";
        assert_eq!(library_paths_from_vdf(content), vec![PathBuf::from("/a")]);
    }

    #[test]
    fn vdf_empty_content_yields_no_paths() {
        assert_eq!(library_paths_from_vdf(""), Vec::<PathBuf>::new());
    }
}
