use std::path::{Path, PathBuf};

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
    fn test_default_prefix_path_with_special_chars() {
        let path = get_default_prefix_path("game-with-uuid-a1b2c3d4");
        assert!(path.to_str().unwrap().contains("game-with-uuid-a1b2c3d4"));
    }

    #[test]
    fn test_global_prefix_path() {
        let path = get_global_prefix_path();
        assert!(path.to_str().unwrap().contains("Poketto"));
        assert!(path.to_str().unwrap().contains("GlobalPrefix"));
    }

    #[test]
    fn test_paths_are_different() {
        let game_path = get_default_prefix_path("test-game");
        let global_path = get_global_prefix_path();
        assert_ne!(game_path, global_path);
    }
}
