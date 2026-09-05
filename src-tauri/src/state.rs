use parking_lot::Mutex;
use std::collections::HashMap;

use crate::discord::DiscordRpc;
use crate::models::{AppSettings, RunningGame, VndbCharacter, VndbVnDetail, WineVersion};

pub struct AppState {
    pub running_game: Mutex<Option<RunningGame>>,
    pub settings: Mutex<AppSettings>,
    pub vn_mem_cache: Mutex<HashMap<String, VndbVnDetail>>,
    pub char_mem_cache: Mutex<HashMap<String, Vec<VndbCharacter>>>,
    pub wine_versions: Mutex<Vec<WineVersion>>,
    pub http_client: reqwest::Client,
    pub discord_rpc: DiscordRpc,
}
