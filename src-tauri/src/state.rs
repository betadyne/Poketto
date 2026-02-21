use parking_lot::Mutex;
use redb::Database;
use std::collections::HashMap;
use std::sync::Arc;

use crate::discord::DiscordRpc;
use crate::models::{
    AppSettings, GameMetadata, RunningGame, VndbCharacter, VndbVnDetail, WineVersion,
};

pub struct AppState {
    pub games: Mutex<Vec<GameMetadata>>,
    pub running_game: Mutex<Option<RunningGame>>,
    pub settings: Mutex<AppSettings>,
    pub vn_mem_cache: Mutex<HashMap<String, VndbVnDetail>>,
    pub char_mem_cache: Mutex<HashMap<String, Vec<VndbCharacter>>>,
    pub wine_versions: Mutex<Vec<WineVersion>>,
    pub http_client: reqwest::Client,
    pub db: Option<Arc<Database>>,
    pub discord_rpc: DiscordRpc,
}
