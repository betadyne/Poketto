use std::path::{Path, PathBuf};
use std::process::Command;

use crate::models::{WineType, WineVersion};

const SYSTEM_WINE_PATHS: &[&str] = &[
    "/usr/bin/wine",
    "/usr/bin/wine64",
    "/opt/wine-stable/bin/wine",
    "/opt/wine-staging/bin/wine",
];

const STEAM_RUN_PATHS: &[&str] = &[
    "/usr/bin/steam-run",
    "/run/current-system/sw/bin/steam-run", // NixOS
];

const AUR_PROTON_LOCATIONS: &[(&str, WineType)] = &[
    ("/opt/proton-ge-custom", WineType::ProtonGE),
    (
        "/usr/share/steam/compatibilitytools.d/proton-ge-custom",
        WineType::ProtonGE,
    ),
    ("/opt/proton-cachyos", WineType::ProtonCachyOS),
    (
        "/usr/share/steam/compatibilitytools.d/proton-cachyos",
        WineType::ProtonCachyOS,
    ),
];

const PROTON_BINARY_NAMES: &[&str] = &["proton", "files/bin/wine64", "files/bin/wine"];

// ============================================================================
// Wine Detection Functions
// ============================================================================

fn get_steam_compat_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".steam/root/compatibilitytools.d"));
        paths.push(home.join(".local/share/Steam/compatibilitytools.d"));
        paths.push(home.join(".steam/steam/compatibilitytools.d"));
    }

    paths.push(PathBuf::from("/usr/share/steam/compatibilitytools.d"));

    paths
}

fn get_steam_proton_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(home) = dirs::home_dir() {
        paths.push(home.join(".steam/root/steamapps/common"));
        paths.push(home.join(".local/share/Steam/steamapps/common"));
        paths.push(home.join(".steam/steam/steamapps/common"));
    }

    paths
}

pub fn get_system_wine() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    for path_str in SYSTEM_WINE_PATHS {
        let path = Path::new(path_str);
        if path.exists() && path.is_file() {
            if let Some(version) = validate_and_get_wine_version(path_str) {
                let name = if path_str.contains("wine64") {
                    format!("Wine {} (64-bit)", version)
                } else if path_str.contains("staging") {
                    format!("Wine Staging {}", version)
                } else {
                    format!("Wine {}", version)
                };

                versions.push(WineVersion {
                    name,
                    binary_path: path_str.to_string(),
                    lib_path: None,
                    wine_type: WineType::Wine,
                    version: Some(version),
                });
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let local_wine = home.join(".local/bin/wine");
        if local_wine.exists() {
            if let Some(path_str) = local_wine.to_str() {
                if let Some(version) = validate_and_get_wine_version(path_str) {
                    versions.push(WineVersion {
                        name: format!("Wine {} (local)", version),
                        binary_path: path_str.to_string(),
                        lib_path: None,
                        wine_type: WineType::Wine,
                        version: Some(version),
                    });
                }
            }
        }
    }

    versions
}

pub fn get_steam_proton() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    for compat_path in get_steam_compat_paths() {
        if !compat_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&compat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(version) = detect_proton_in_folder(&path) {
                        let name_lower = version.name.to_lowercase();
                        if !name_lower.contains("ge-proton") && !name_lower.contains("cachyos") {
                            versions.push(version);
                        }
                    }
                }
            }
        }
    }

    for common_path in get_steam_proton_paths() {
        if !common_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&common_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("Proton") {
                            if let Some(version) = detect_proton_in_folder(&path) {
                                versions.push(version);
                            }
                        }
                    }
                }
            }
        }
    }

    versions
}

pub fn get_aur_proton() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    for (path_str, wine_type) in AUR_PROTON_LOCATIONS {
        let path = Path::new(path_str);
        if path.exists() && path.is_dir() {
            if let Some(mut version) = detect_proton_in_folder(path) {
                version.wine_type = wine_type.clone();
                version.name = match wine_type {
                    WineType::ProtonGE => {
                        if let Some(ref v) = version.version {
                            format!("GE-Proton {}", v)
                        } else {
                            "GE-Proton".to_string()
                        }
                    }
                    WineType::ProtonCachyOS => {
                        if let Some(ref v) = version.version {
                            format!("Proton CachyOS {}", v)
                        } else {
                            "Proton CachyOS".to_string()
                        }
                    }
                    _ => version.name,
                };
                versions.push(version);
            }
        }
    }

    for compat_path in get_steam_compat_paths() {
        if !compat_path.exists() {
            continue;
        }

        if let Ok(entries) = std::fs::read_dir(&compat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("GE-Proton") {
                            if let Some(mut version) = detect_proton_in_folder(&path) {
                                version.wine_type = WineType::ProtonGE;
                                version.name = name.to_string();
                                if let Some(v) = name.strip_prefix("GE-Proton") {
                                    version.version = Some(v.to_string());
                                }
                                versions.push(version);
                            }
                        }
                    }
                }
            }
        }
    }

    versions
}

fn detect_proton_in_folder(folder: &Path) -> Option<WineVersion> {
    for binary_name in PROTON_BINARY_NAMES {
        let binary_path = folder.join(binary_name);
        if binary_path.exists() {
            let folder_name = folder.file_name()?.to_str()?;

            let version = get_proton_version(folder).or_else(|| {
                folder_name
                    .strip_prefix("Proton ")
                    .or_else(|| folder_name.strip_prefix("Proton-"))
                    .or_else(|| folder_name.strip_prefix("GE-Proton"))
                    .map(|s| s.to_string())
            });

            let lib_path = folder.join("files/lib64");
            let lib_path_str = if lib_path.exists() {
                lib_path.to_str().map(|s| s.to_string())
            } else {
                None
            };

            return Some(WineVersion {
                name: folder_name.to_string(),
                binary_path: binary_path.to_str()?.to_string(),
                lib_path: lib_path_str,
                wine_type: WineType::Proton,
                version,
            });
        }
    }

    None
}

fn get_proton_version(folder: &Path) -> Option<String> {
    let version_file = folder.join("version");
    if version_file.exists() {
        if let Ok(content) = std::fs::read_to_string(&version_file) {
            let version = content.trim().to_string();
            if !version.is_empty() {
                return Some(version);
            }
        }
    }
    None
}

pub fn get_all_wine_versions() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    versions.extend(get_system_wine());

    versions.extend(get_aur_proton());

    versions.extend(get_steam_proton());

    versions.dedup_by(|a, b| a.binary_path == b.binary_path);

    versions
}

pub fn get_default_wine() -> Option<WineVersion> {
    for path_str in ["/usr/bin/wine", "/usr/bin/wine64"] {
        let path = Path::new(path_str);
        if path.exists() {
            if let Some(version) = validate_and_get_wine_version(path_str) {
                return Some(WineVersion {
                    name: format!("Wine {}", version),
                    binary_path: path_str.to_string(),
                    lib_path: None,
                    wine_type: WineType::Wine,
                    version: Some(version),
                });
            }
        }
    }

    get_all_wine_versions().into_iter().next()
}

pub fn validate_and_get_wine_version(binary_path: &str) -> Option<String> {
    let output = Command::new(binary_path).arg("--version").output().ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        let version = version_str
            .trim()
            .strip_prefix("wine-")
            .unwrap_or(version_str.trim())
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string();
        Some(version)
    } else {
        None
    }
}

pub fn validate_wine_binary(binary_path: &str) -> Result<String, String> {
    let path = Path::new(binary_path);

    if !path.exists() {
        return Err(format!("Wine binary not found: {}", binary_path));
    }

    if !path.is_file() {
        return Err(format!("Path is not a file: {}", binary_path));
    }

    validate_and_get_wine_version(binary_path)
        .ok_or_else(|| format!("Failed to execute Wine binary: {}", binary_path))
}

pub fn get_default_prefix_path(game_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Games")
        .join("Poketto")
        .join("Prefixes")
        .join(game_id)
}

pub fn get_global_prefix_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Games")
        .join("Poketto")
        .join("GlobalPrefix")
}

pub fn ensure_prefix_exists(prefix_path: &Path) -> Result<(), String> {
    if !prefix_path.exists() {
        std::fs::create_dir_all(prefix_path)
            .map_err(|e| format!("Failed to create prefix directory: {}", e))?;
    }
    Ok(())
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
    fn test_default_prefix_path() {
        let path = get_default_prefix_path("test-game-123");
        assert!(path.to_str().unwrap().contains("Poketto"));
        assert!(path.to_str().unwrap().contains("Prefixes"));
        assert!(path.to_str().unwrap().contains("test-game-123"));
    }

    #[test]
    fn test_global_prefix_path() {
        let path = get_global_prefix_path();
        assert!(path.to_str().unwrap().contains("Poketto"));
        assert!(path.to_str().unwrap().contains("GlobalPrefix"));
    }
}
