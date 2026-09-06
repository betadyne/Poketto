use parking_lot::Mutex;
use std::collections::HashMap;
use std::time::Instant;

use crate::discord::DiscordRpc;
use crate::models::{AppSettings, RunningGame, VndbCharacter, VndbVnDetail, WineVersion};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Settle {
    Mine,
    Replaced,
    Gone,
}

pub(crate) fn settle_running_slot(
    slot: &mut Option<RunningGame>,
    id: &str,
    start_time: Instant,
) -> Settle {
    match slot {
        Some(running) if running.id == id && running.start_time == start_time => {
            *slot = None;
            Settle::Mine
        }
        Some(_) => Settle::Replaced,
        None => Settle::Gone,
    }
}
pub struct AppState {
    pub running_game: Mutex<Option<RunningGame>>,
    pub settings: Mutex<AppSettings>,
    pub vn_mem_cache: Mutex<HashMap<String, VndbVnDetail>>,
    pub char_mem_cache: Mutex<HashMap<String, Vec<VndbCharacter>>>,
    pub wine_versions: Mutex<Vec<WineVersion>>,
    pub http_client: reqwest::Client,
    pub discord_rpc: DiscordRpc,
}

impl AppState {
    pub fn settle_running(&self, id: &str, start_time: Instant) -> Settle {
        settle_running_slot(&mut self.running_game.lock(), id, start_time)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn running(id: &str) -> RunningGame {
        RunningGame {
            id: id.to_string(),
            start_time: Instant::now(),
            pid: None,
            binary: None,
        }
    }

    #[test]
    fn test_settle_mine_takes_slot() {
        let session = running("game-1");
        let mut slot = Some(session);
        let start = slot.as_ref().expect("session set").start_time;
        assert_eq!(settle_running_slot(&mut slot, "game-1", start), Settle::Mine);
        assert!(slot.is_none());
    }

    #[test]
    fn test_settle_replaced_keeps_newer_session() {
        let session = running("game-1");
        let mut slot = Some(session);
        assert_eq!(
            settle_running_slot(&mut slot, "game-1", Instant::now()),
            Settle::Replaced
        );
        assert!(slot.is_some());
        assert_eq!(
            settle_running_slot(&mut slot, "game-2", Instant::now()),
            Settle::Replaced
        );
        assert!(slot.is_some());
    }

    #[test]
    fn test_settle_gone_skips_missing_session() {
        let mut slot: Option<RunningGame> = None;
        assert_eq!(
            settle_running_slot(&mut slot, "game-1", Instant::now()),
            Settle::Gone
        );
    }
}
