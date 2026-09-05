use std::path::{Path, PathBuf};

pub fn get_default_prefix_path(game_id: &str, vndb_id: Option<&str>) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Games")
        .join("Poketto")
        .join("Prefixes")
        .join(default_prefix_dir_name(game_id, vndb_id))
}

fn default_prefix_dir_name(game_id: &str, vndb_id: Option<&str>) -> String {
    if let Some(name) = vndb_id.map(str::trim).filter(|s| !s.is_empty()) {
        if let Some(sanitized) = sanitize_dir_name(name) {
            return sanitized;
        }
    }
    short_game_id(game_id)
}

fn sanitize_dir_name(name: &str) -> Option<String> {
    let cleaned: String = name
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-'))
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        None
    } else {
        Some(cleaned)
    }
}

fn short_game_id(game_id: &str) -> String {
    let short = game_id.split('-').next().unwrap_or_default();
    if short.is_empty() {
        "game".to_string()
    } else {
        short.to_string()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_prefix_path() {
        let path = get_default_prefix_path("test-game-123", None);
        assert!(path.to_str().unwrap().contains("Poketto"));
        assert!(path.to_str().unwrap().contains("Prefixes"));
        assert!(path.to_str().unwrap().ends_with("test"));
    }

    #[test]
    fn test_default_prefix_prefers_vndb_id() {
        let path = get_default_prefix_path("9f3a2c1e-7b4a-4d9c-8f2a-1b3c5d7e9f01", Some("v412830"));
        assert!(path.to_str().unwrap().ends_with("v412830"));
    }

    #[test]
    fn test_default_prefix_sanitizes_vndb_id() {
        let path = get_default_prefix_path("abc12345-uuid", Some("../v 99"));
        assert!(path.to_str().unwrap().ends_with("v99"));
        let dotdot = get_default_prefix_path("abc12345-uuid", Some(".."));
        assert!(dotdot.to_str().unwrap().ends_with("abc12345"));
        let empty = get_default_prefix_path("abc12345-uuid", Some("   "));
        assert!(empty.to_str().unwrap().ends_with("abc12345"));
    }

    #[test]
    fn test_default_prefix_path_with_special_chars() {
        let path = get_default_prefix_path("game-with-uuid-a1b2c3d4", None);
        assert!(path.to_str().unwrap().ends_with("game"));
    }

    #[test]
    fn test_global_prefix_path() {
        let path = get_global_prefix_path();
        assert!(path.to_str().unwrap().contains("Poketto"));
        assert!(path.to_str().unwrap().contains("GlobalPrefix"));
    }

    #[test]
    fn test_paths_are_different() {
        let game_path = get_default_prefix_path("test-game", None);
        let global_path = get_global_prefix_path();
        assert_ne!(game_path, global_path);
    }
}
