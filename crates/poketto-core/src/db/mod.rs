mod error;
mod repository;
mod schema;

pub use error::{DbError, DbResult};
pub use rusqlite::Connection;
pub use repository::{
    SortBy, SortOrder, add_tag, cache_clear, cache_get, cache_put, cache_remove, daily_playtime,
    delete_game, get_all_games, get_game, insert_game, load_settings, load_sort_pref,
    record_play_session, save_settings, save_sort_pref, tag_game, tags_for_game, untag_game,
    update_game, update_game_status, CacheEntry,
};
pub use schema::{open, open_in_memory, SCHEMA_VERSION};
