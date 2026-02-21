use std::path::Path;
use std::process::Command;

use crate::models::{WineType, WineVersion};

use super::classify::{classify_wine_type, format_wine_name};

const WINE_BINARY_NAMES: &[&str] = &["bin/wine64", "bin/wine", "wine64", "wine"];

const PROTON_BINARY_NAMES: &[&str] = &["proton", "files/bin/wine64", "files/bin/wine"];

pub(super) fn detect_proton_in_folder(folder: &Path) -> Option<WineVersion> {
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
                source: None,
            });
        }
    }

    None
}

pub(super) fn detect_wine_in_folder(folder: &Path) -> Option<WineVersion> {
    for binary_name in WINE_BINARY_NAMES {
        let binary_path = folder.join(binary_name);
        if binary_path.exists() && binary_path.is_file() {
            let folder_name = folder.file_name()?.to_str()?;
            let binary_str = binary_path.to_str()?;

            let version = validate_and_get_wine_version(binary_str);
            let wine_type = classify_wine_type(folder_name);

            let lib_path = folder.join("lib64");
            let lib_path_str = if lib_path.exists() {
                lib_path.to_str().map(|s| s.to_string())
            } else {
                None
            };

            return Some(WineVersion {
                name: format_wine_name(folder_name),
                binary_path: binary_str.to_string(),
                lib_path: lib_path_str,
                wine_type,
                version,
                source: None,
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

pub(super) fn validate_and_get_wine_version(binary_path: &str) -> Option<String> {
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
