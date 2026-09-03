use std::path::Path;
use std::process::Command;

use crate::models::{WineType, WineVersion};

use super::classify::{classify_wine_type, format_wine_name};
use super::error::{WineError, WineResult};

const WINE_BINARY_NAMES: &[&str] = &["bin/wine64", "bin/wine", "wine64", "wine"];

const PROTON_BINARY_NAMES: &[&str] = &["proton", "files/bin/wine64", "files/bin/wine"];

pub fn detect_proton_in_folder(folder: &Path) -> Option<WineVersion> {
    for binary_name in PROTON_BINARY_NAMES {
        let binary_path = folder.join(binary_name);
        if binary_path.exists() {
            let folder_name = folder.file_name()?.to_str()?;

            let version = proton_version_from_file(folder).or_else(|| {
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

pub fn detect_wine_in_folder(folder: &Path) -> Option<WineVersion> {
    for binary_name in WINE_BINARY_NAMES {
        let binary_path = folder.join(binary_name);
        if binary_path.exists() && binary_path.is_file() {
            let folder_name = folder.file_name()?.to_str()?;
            let binary_str = binary_path.to_str()?;

            let version = query_wine_version(binary_str);
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

fn proton_version_from_file(folder: &Path) -> Option<String> {
    let content = std::fs::read_to_string(folder.join("version")).ok()?;
    let version = content.trim().to_string();
    if version.is_empty() { None } else { Some(version) }
}

pub fn query_wine_version(binary_path: &str) -> Option<String> {
    let output = Command::new(binary_path).arg("--version").output().ok()?;

    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(
        stdout
            .trim()
            .strip_prefix("wine-")
            .unwrap_or(stdout.trim())
            .split_whitespace()
            .next()
            .unwrap_or("unknown")
            .to_string(),
    )
}

pub fn validate_wine_binary(binary_path: &str) -> WineResult<String> {
    let path = Path::new(binary_path);

    if !path.exists() {
        return Err(WineError::NotFound(binary_path.to_string()));
    }

    if !path.is_file() {
        return Err(WineError::NotAFile(binary_path.to_string()));
    }

    query_wine_version(binary_path)
        .ok_or_else(|| WineError::ExecutionFailed(binary_path.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_dir(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("poketto-wine-test").join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("fixture dir");
        dir
    }

    #[test]
    fn proton_fixture_prefers_version_file() {
        let dir = fixture_dir("proton-version");
        std::fs::write(dir.join("proton"), "").expect("binary stub");
        std::fs::write(dir.join("version"), "9.0-2\n").expect("version file");
        let found = detect_proton_in_folder(&dir).expect("detected");
        assert_eq!(found.version.as_deref(), Some("9.0-2"));
        assert_eq!(found.wine_type, WineType::Proton);
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }

    #[test]
    fn proton_fixture_falls_back_to_folder_name() {
        let dir = fixture_dir("GE-Proton9-7");
        std::fs::write(dir.join("proton"), "").expect("binary stub");
        let found = detect_proton_in_folder(&dir).expect("detected");
        assert_eq!(found.version.as_deref(), Some("9-7"));
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }

    #[test]
    fn empty_folder_detects_nothing() {
        let dir = fixture_dir("empty");
        assert_eq!(detect_proton_in_folder(&dir).is_none(), true);
        assert_eq!(detect_wine_in_folder(&dir).is_none(), true);
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn wine_fixture_reports_script_version() {
        use std::os::unix::fs::PermissionsExt;

        let dir = fixture_dir("wine-script");
        let binary = dir.join("bin").join("wine64");
        std::fs::create_dir_all(binary.parent().expect("bin")).expect("bin dir");
        std::fs::write(&binary, "#!/bin/sh\necho wine-9.0\n").expect("script");
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))
            .expect("executable");
        let found = detect_wine_in_folder(&dir).expect("detected");
        assert_eq!(found.version.as_deref(), Some("9.0"));
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }

    #[test]
    fn missing_binary_fails_validation() {
        assert!(matches!(
            validate_wine_binary("/nonexistent/wine-binary"),
            Err(WineError::NotFound(_))
        ));
    }

    #[test]
    fn directory_fails_validation() {
        let dir = fixture_dir("not-a-file");
        assert!(matches!(
            validate_wine_binary(dir.to_str().expect("unicode")),
            Err(WineError::NotAFile(_))
        ));
        let _ = std::fs::remove_dir_all(dir.parent().expect("parent"));
    }
}
