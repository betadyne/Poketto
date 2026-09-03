use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    pub vndb_token: Option<String>,
    pub vndb_user_id: Option<String>,
    #[serde(default)]
    pub blur_nsfw: bool,
    #[serde(default = "default_discord_enabled")]
    pub discord_rpc_enabled: bool,
    #[serde(default = "default_true")]
    pub discord_btn_vndb_game: bool,
    #[serde(default)]
    pub discord_btn_vndb_profile: bool,
    #[serde(default)]
    pub discord_btn_github: bool,
    #[serde(default)]
    pub default_wine_prefix: Option<String>,
    #[serde(default)]
    pub default_wine_binary: Option<String>,
    #[serde(default)]
    pub use_steam_runtime: bool,
}

fn default_discord_enabled() -> bool {
    true
}

fn default_true() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_settings_json_keeps_defaults() {
        let settings: AppSettings = serde_json::from_str("{}").expect("empty JSON parses");
        assert_eq!(settings.discord_rpc_enabled, true);
        assert_eq!(settings.discord_btn_vndb_game, true);
        assert_eq!(settings.blur_nsfw, false);
        assert_eq!(settings.vndb_token, None);
    }
}
