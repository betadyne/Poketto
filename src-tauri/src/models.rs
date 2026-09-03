use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Wine/Proton Types for Linux Support
// ============================================================================

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum WineType {
    Wine,
    WineGE,
    WineStaging,
    WineTKG,
    Proton,
    ProtonGE,
    ProtonCachyOS,
    ProtonTKG,
    Lutris,
    Bottles,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum WineSource {
    System,
    Opt,
    Steam,
    SteamFlatpak,
    Lutris,
    Bottles,
    BottlesFlatpak,
    Custom,
}

impl Default for WineType {
    fn default() -> Self {
        WineType::Wine
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, specta::Type)]
pub enum GameType {
    WindowsExe,
    LinuxNative,
}

impl Default for GameType {
    fn default() -> Self {
        GameType::WindowsExe
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct WineVersion {
    pub name: String,
    pub binary_path: String,
    pub lib_path: Option<String>,
    pub wine_type: WineType,
    pub version: Option<String>,
    pub source: Option<WineSource>,
}

/// Per-game Wine/Proton settings
#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct WineSettings {
    pub use_global_prefix: bool,
    pub wine_prefix: Option<String>,
    pub wine_version: Option<String>,
    pub wine_type: Option<WineType>,
    pub use_steam_runtime: bool,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct GameMetadata {
    pub id: String,
    pub title: String,
    pub path: String,
    pub vndb_id: Option<String>,
    pub cover_url: Option<String>,
    pub play_time: u64,
    pub is_finished: bool,
    #[serde(default)]
    pub last_played: Option<String>,
    #[serde(default)]
    pub is_hidden: bool,
    #[serde(default)]
    pub show_spoilers: bool,
    #[serde(default)]
    pub game_type: Option<GameType>,
    #[serde(default)]
    pub wine_settings: Option<WineSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct DailyPlaytimeData {
    pub games: HashMap<String, HashMap<String, u64>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbSearchResult {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub released: Option<String>,
    pub rating: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbImage {
    pub url: String,
    #[serde(default)]
    pub sexual: f64,
    #[serde(default)]
    pub violence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbResponse<T> {
    pub results: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbVnDetail {
    pub id: String,
    pub title: String,
    pub image: Option<VndbImage>,
    pub released: Option<String>,
    pub rating: Option<f64>,
    pub description: Option<String>,
    pub length: Option<i32>,
    pub length_minutes: Option<i32>,
    pub tags: Option<Vec<VndbTag>>,
    pub developers: Option<Vec<VndbProducer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbTag {
    pub id: String,
    pub name: String,
    pub rating: f64,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbProducer {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbCharacter {
    pub id: String,
    pub name: String,
    pub original: Option<String>,
    pub aliases: Option<Vec<String>>,
    pub image: Option<VndbImage>,
    pub description: Option<String>,
    pub blood_type: Option<String>,
    pub height: Option<i32>,
    pub weight: Option<i32>,
    pub bust: Option<i32>,
    pub waist: Option<i32>,
    pub hips: Option<i32>,
    pub cup: Option<String>,
    pub age: Option<i32>,
    pub birthday: Option<Vec<i32>>,
    pub sex: Option<Vec<String>>,
    pub vns: Option<Vec<VndbCharacterVn>>,
    pub traits: Option<Vec<VndbTrait>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbTrait {
    pub id: String,
    pub name: String,
    pub group_id: Option<String>,
    pub group_name: Option<String>,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbCharacterVn {
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub spoiler: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbUserListItem {
    pub id: String,
    pub vote: Option<i32>,
    pub labels: Option<Vec<VndbLabel>>,
    pub started: Option<String>,
    pub finished: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbLabel {
    pub id: i32,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
pub struct VndbAuthInfo {
    pub id: String,
    pub username: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, specta::Type)]
pub struct AppSettings {
    pub vndb_token: Option<String>,
    pub vndb_user_id: Option<String>,
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

#[derive(Debug, Clone, Serialize)]
pub struct GameExitedPayload {
    pub game_id: String,
    pub play_minutes: u64,
}

pub struct RunningGame {
    pub id: String,
    pub start_time: Instant,
    pub title: String,
    pub cover_url: Option<String>,
    pub discord_start_timestamp: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    mod default_trait_tests {
        use super::*;

        #[test]
        fn test_wine_type_default() {
            let default = WineType::default();
            assert_eq!(default, WineType::Wine);
        }

        #[test]
        fn test_game_type_default() {
            let default = GameType::default();
            assert_eq!(default, GameType::WindowsExe);
        }

        #[test]
        fn test_wine_settings_default() {
            let default = WineSettings::default();
            assert!(!default.use_global_prefix);
            assert!(default.wine_prefix.is_none());
            assert!(default.wine_version.is_none());
            assert!(default.wine_type.is_none());
            assert!(!default.use_steam_runtime);
            assert!(default.env_vars.is_empty());
        }

        #[test]
        fn test_daily_playtime_data_default() {
            let default = DailyPlaytimeData::default();
            assert!(default.games.is_empty());
        }
    }

    mod default_helper_tests {
        use super::*;

        #[test]
        fn test_default_discord_enabled_returns_true() {
            assert!(default_discord_enabled());
        }

        #[test]
        fn test_default_true_returns_true() {
            assert!(default_true());
        }
    }

    mod serialization_tests {
        use super::*;

        #[test]
        fn test_wine_type_serializes() {
            let wine_type = WineType::ProtonGE;
            let json = serde_json::to_string(&wine_type).unwrap();
            assert_eq!(json, "\"ProtonGE\"");
        }

        #[test]
        fn test_wine_type_deserializes() {
            let json = "\"ProtonGE\"";
            let wine_type: WineType = serde_json::from_str(json).unwrap();
            assert_eq!(wine_type, WineType::ProtonGE);
        }

        #[test]
        fn test_game_type_serializes() {
            let game_type = GameType::LinuxNative;
            let json = serde_json::to_string(&game_type).unwrap();
            assert_eq!(json, "\"LinuxNative\"");
        }

        #[test]
        fn test_game_type_deserializes() {
            let json = "\"WindowsExe\"";
            let game_type: GameType = serde_json::from_str(json).unwrap();
            assert_eq!(game_type, GameType::WindowsExe);
        }

        #[test]
        fn test_wine_source_serializes() {
            let source = WineSource::SteamFlatpak;
            let json = serde_json::to_string(&source).unwrap();
            assert_eq!(json, "\"SteamFlatpak\"");
        }

        #[test]
        fn test_wine_settings_roundtrip() {
            let mut env_vars = HashMap::new();
            env_vars.insert("WINEPREFIX".to_string(), "/home/user/.wine".to_string());

            let settings = WineSettings {
                use_global_prefix: true,
                wine_prefix: Some("/custom/prefix".to_string()),
                wine_version: Some("/usr/bin/wine".to_string()),
                wine_type: Some(WineType::ProtonGE),
                use_steam_runtime: true,
                env_vars,
            };

            let json = serde_json::to_string(&settings).unwrap();
            let deserialized: WineSettings = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.use_global_prefix, settings.use_global_prefix);
            assert_eq!(deserialized.wine_prefix, settings.wine_prefix);
            assert_eq!(deserialized.wine_version, settings.wine_version);
            assert_eq!(deserialized.wine_type, settings.wine_type);
            assert_eq!(deserialized.use_steam_runtime, settings.use_steam_runtime);
            assert_eq!(deserialized.env_vars.len(), 1);
        }

        #[test]
        fn test_game_metadata_with_missing_optional_fields() {
            let json = r#"{
                "id": "game-123",
                "title": "Test Game",
                "path": "/path/to/game.exe",
                "vndb_id": null,
                "cover_url": null,
                "play_time": 60,
                "is_finished": false
            }"#;

            let game: GameMetadata = serde_json::from_str(json).unwrap();
            assert_eq!(game.id, "game-123");
            assert_eq!(game.title, "Test Game");
            assert!(game.last_played.is_none());
            assert!(!game.is_hidden);
            assert!(game.game_type.is_none());
            assert!(game.wine_settings.is_none());
        }
    }

    mod wine_type_tests {
        use super::*;

        #[test]
        fn test_all_wine_types_are_distinct() {
            let types = vec![
                WineType::Wine,
                WineType::WineGE,
                WineType::WineStaging,
                WineType::WineTKG,
                WineType::Proton,
                WineType::ProtonGE,
                WineType::ProtonCachyOS,
                WineType::ProtonTKG,
                WineType::Lutris,
                WineType::Bottles,
                WineType::Custom,
            ];

            for (i, t1) in types.iter().enumerate() {
                for (j, t2) in types.iter().enumerate() {
                    if i != j {
                        assert_ne!(t1, t2);
                    }
                }
            }
        }

        #[test]
        fn test_wine_type_clone() {
            let original = WineType::ProtonGE;
            let cloned = original.clone();
            assert_eq!(original, cloned);
        }
    }

    mod wine_source_tests {
        use super::*;

        #[test]
        fn test_all_wine_sources_serialize_correctly() {
            let sources = vec![
                (WineSource::System, "\"System\""),
                (WineSource::Opt, "\"Opt\""),
                (WineSource::Steam, "\"Steam\""),
                (WineSource::SteamFlatpak, "\"SteamFlatpak\""),
                (WineSource::Lutris, "\"Lutris\""),
                (WineSource::Bottles, "\"Bottles\""),
                (WineSource::BottlesFlatpak, "\"BottlesFlatpak\""),
                (WineSource::Custom, "\"Custom\""),
            ];

            for (source, expected) in sources {
                let json = serde_json::to_string(&source).unwrap();
                assert_eq!(json, expected);
            }
        }
    }

    mod game_exited_payload_tests {
        use super::*;

        #[test]
        fn test_payload_serializes() {
            let payload = GameExitedPayload {
                game_id: "game-123".to_string(),
                play_minutes: 45,
            };

            let json = serde_json::to_string(&payload).unwrap();
            assert!(json.contains("game-123"));
            assert!(json.contains("45"));
        }
    }
}
