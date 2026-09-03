use std::path::{Path, PathBuf};

use super::error::{WineError, WineResult};

pub fn default_prefix_path(game_id: &str) -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Games")
        .join("Poketto")
        .join("Prefixes")
        .join(game_id)
}

pub fn global_prefix_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("Games")
        .join("Poketto")
        .join("GlobalPrefix")
}

pub fn ensure_prefix_exists(prefix_path: &Path) -> WineResult<()> {
    if !prefix_path.exists() {
        std::fs::create_dir_all(prefix_path).map_err(WineError::from)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_prefix_contains_game_id() {
        let path = default_prefix_path("game-with-uuid-a1b2c3d4");
        let text = path.to_str().expect("unicode path");
        assert!(text.contains("Poketto"));
        assert!(text.contains("Prefixes"));
        assert!(text.contains("game-with-uuid-a1b2c3d4"));
    }

    #[test]
    fn global_prefix_differs_from_game_prefix() {
        assert_ne!(default_prefix_path("test-game"), global_prefix_path());
        assert!(global_prefix_path()
            .to_str()
            .expect("unicode path")
            .contains("GlobalPrefix"));
    }

    #[test]
    fn ensure_prefix_creates_nested_dirs() {
        let base = std::env::temp_dir().join("poketto-prefix-test");
        let nested = base.join("nested").join("pfx");
        let _ = std::fs::remove_dir_all(&base);
        ensure_prefix_exists(&nested).expect("create");
        assert!(nested.exists());
        ensure_prefix_exists(&nested).expect("idempotent");
        let _ = std::fs::remove_dir_all(&base);
    }
}
