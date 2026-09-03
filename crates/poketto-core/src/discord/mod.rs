mod worker;

pub use worker::{
    PresenceHandle, PresenceUpdate, DEFAULT_STATE_TEXT, DISCORD_CLIENT_ID,
    spawn_presence_worker, unix_timestamp,
};
