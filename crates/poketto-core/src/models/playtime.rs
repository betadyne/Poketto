use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaySession {
    pub id: i64,
    pub game_id: String,
    pub started_at: String,
    pub play_date: String,
    pub minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DailyPlaytimeData {
    pub games: HashMap<String, HashMap<String, u64>>,
}
