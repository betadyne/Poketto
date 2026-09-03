use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum WineType {
    #[default]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameType {
    #[default]
    WindowsExe,
    LinuxNative,
}

impl GameType {
    pub fn as_str(&self) -> &'static str {
        match self {
            GameType::WindowsExe => "WindowsExe",
            GameType::LinuxNative => "LinuxNative",
        }
    }
}

impl FromStr for GameType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "WindowsExe" => Ok(GameType::WindowsExe),
            "LinuxNative" => Ok(GameType::LinuxNative),
            _ => Err(value.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WineVersion {
    pub name: String,
    pub binary_path: String,
    pub lib_path: Option<String>,
    pub wine_type: WineType,
    pub version: Option<String>,
    pub source: Option<WineSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WineSettings {
    pub use_global_prefix: bool,
    pub wine_prefix: Option<String>,
    pub wine_version: Option<String>,
    pub wine_type: Option<WineType>,
    pub use_steam_runtime: bool,
    #[serde(default)]
    pub env_vars: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub id: String,
    pub title: String,
    pub path: String,
    pub vndb_id: Option<String>,
    pub cover_url: Option<String>,
    #[serde(default, alias = "play_time")]
    pub play_time_minutes: u64,
    #[serde(default)]
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
    #[serde(default)]
    pub rating: Option<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_games_json_deserializes() {
        let json = r#"{
            "id": "abc",
            "title": "Legacy Title",
            "path": "/games/legacy",
            "vndb_id": "v17",
            "cover_url": null,
            "play_time": 42,
            "is_finished": false
        }"#;
        let game: Game = serde_json::from_str(json).expect("legacy JSON parses");
        assert_eq!(game.id, "abc");
        assert_eq!(game.play_time_minutes, 42);
        assert_eq!(game.is_hidden, false);
        assert_eq!(game.game_type, None);
        assert_eq!(game.wine_settings, None);
    }

    #[test]
    fn game_type_round_trips_through_str() {
        assert_eq!(
            "LinuxNative".parse::<GameType>(),
            Ok(GameType::LinuxNative)
        );
        assert_eq!("Unknown".parse::<GameType>().is_err(), true);
        assert_eq!(GameType::WindowsExe.as_str(), "WindowsExe");
    }
}
