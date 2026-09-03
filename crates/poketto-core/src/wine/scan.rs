use std::path::Path;

use crate::models::{WineSource, WineType, WineVersion};

use super::classify::{classify_wine_type, format_version_name};
use super::detect::{detect_proton_in_folder, detect_wine_in_folder, query_wine_version};
use super::steam::{steam_compat_paths, steam_proton_paths};

const SYSTEM_WINE_PATHS: &[&str] = &[
    "/usr/bin/wine",
    "/usr/bin/wine64",
    "/opt/wine-stable/bin/wine",
    "/opt/wine-staging/bin/wine",
];

fn flatpak_source(path: &Path) -> WineSource {
    if path
        .to_str()
        .unwrap_or("")
        .contains(".var/app/com.valvesoftware.Steam")
    {
        WineSource::SteamFlatpak
    } else {
        WineSource::Steam
    }
}

fn system_wine() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    for path_str in SYSTEM_WINE_PATHS {
        let path = Path::new(path_str);
        if path.exists() && path.is_file() {
            if let Some(version) = query_wine_version(path_str) {
                let name = if path_str.contains("wine64") {
                    format!("Wine {version} (64-bit)")
                } else if path_str.contains("staging") {
                    format!("Wine Staging {version}")
                } else {
                    format!("Wine {version}")
                };

                versions.push(WineVersion {
                    name,
                    binary_path: path_str.to_string(),
                    lib_path: None,
                    wine_type: WineType::Wine,
                    version: Some(version),
                    source: Some(WineSource::System),
                });
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let local_wine = home.join(".local/bin/wine");
        if local_wine.exists() {
            if let Some(path_str) = local_wine.to_str() {
                if let Some(version) = query_wine_version(path_str) {
                    versions.push(WineVersion {
                        name: format!("Wine {version} (local)"),
                        binary_path: path_str.to_string(),
                        lib_path: None,
                        wine_type: WineType::Wine,
                        version: Some(version),
                        source: Some(WineSource::System),
                    });
                }
            }
        }
    }

    versions
}

fn scan_opt_directory() -> Vec<WineVersion> {
    let opt_dir = Path::new("/opt");
    let mut versions = Vec::new();

    if !opt_dir.exists() {
        return versions;
    }

    if let Ok(entries) = std::fs::read_dir(opt_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let folder_name = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name,
                None => continue,
            };

            let name_lower = folder_name.to_lowercase();

            if name_lower.starts_with("proton") || name_lower.starts_with("wine") {
                if let Some(mut version) = detect_proton_in_folder(&path) {
                    version.wine_type = classify_wine_type(folder_name);
                    version.name = format_version_name(folder_name);
                    version.source = Some(WineSource::Opt);
                    versions.push(version);
                } else if let Some(mut version) = detect_wine_in_folder(&path) {
                    version.source = Some(WineSource::Opt);
                    versions.push(version);
                }
            }
        }
    }

    versions
}

fn installed_proton() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    versions.extend(scan_opt_directory());

    for compat_path in steam_compat_paths() {
        let source = flatpak_source(&compat_path);

        if let Ok(entries) = std::fs::read_dir(&compat_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(mut version) = detect_proton_in_folder(&path) {
                        let folder_name =
                            path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                        version.wine_type = classify_wine_type(folder_name);
                        version.name = format_version_name(folder_name);
                        version.source = Some(source.clone());
                        versions.push(version);
                    }
                }
            }
        }
    }

    versions
}

fn steam_proton() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    for common_path in steam_proton_paths() {
        let source = flatpak_source(&common_path);

        if let Ok(entries) = std::fs::read_dir(&common_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if name.starts_with("Proton") {
                            if let Some(mut version) = detect_proton_in_folder(&path) {
                                version.source = Some(source.clone());
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

fn lutris_wine() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let lutris_path = home.join(".local/share/lutris/runners/wine");

        if lutris_path.exists() {
            if let Ok(entries) = std::fs::read_dir(&lutris_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        if let Some(mut version) = detect_wine_in_folder(&path) {
                            version.wine_type = WineType::Lutris;
                            version.source = Some(WineSource::Lutris);
                            let folder_name = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            version.name = format!("Lutris: {folder_name}");
                            versions.push(version);
                        }
                    }
                }
            }
        }
    }

    versions
}

fn bottles_wine() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    if let Some(home) = dirs::home_dir() {
        let bottles_paths = [
            (
                home.join(".local/share/bottles/runners"),
                WineSource::Bottles,
            ),
            (
                home.join(".var/app/com.usebottles.bottles/data/bottles/runners"),
                WineSource::BottlesFlatpak,
            ),
        ];

        for (runners_path, source) in bottles_paths {
            if !runners_path.exists() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&runners_path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let folder_name = path
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or("unknown");

                        if let Some(mut version) = detect_wine_in_folder(&path) {
                            version.wine_type = WineType::Bottles;
                            version.source = Some(source.clone());
                            version.name = format!("Bottles: {folder_name}");
                            versions.push(version);
                        } else if let Some(mut version) = detect_proton_in_folder(&path) {
                            version.wine_type = WineType::Bottles;
                            version.source = Some(source.clone());
                            version.name = format!("Bottles: {folder_name}");
                            versions.push(version);
                        }
                    }
                }
            }
        }
    }

    versions
}

pub fn get_all_wine_versions() -> Vec<WineVersion> {
    let mut versions = Vec::new();

    versions.extend(system_wine());
    versions.extend(installed_proton());
    versions.extend(steam_proton());
    versions.extend(lutris_wine());
    versions.extend(bottles_wine());

    versions.dedup_by(|a, b| a.binary_path == b.binary_path);

    versions
}

pub fn get_default_wine() -> Option<WineVersion> {
    for path_str in ["/usr/bin/wine", "/usr/bin/wine64"] {
        let path = Path::new(path_str);
        if path.exists() {
            if let Some(version) = query_wine_version(path_str) {
                return Some(WineVersion {
                    name: format!("Wine {version}"),
                    binary_path: path_str.to_string(),
                    lib_path: None,
                    wine_type: WineType::Wine,
                    version: Some(version),
                    source: Some(WineSource::System),
                });
            }
        }
    }

    get_all_wine_versions().into_iter().next()
}
