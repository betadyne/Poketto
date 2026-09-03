mod error;
mod repository;
mod schema;

pub use error::{DbError, DbResult};
pub use repository::{
    add_tag, daily_playtime, delete_game, get_all_games, get_game, insert_game, load_settings,
    record_play_session, save_settings, tag_game, tags_for_game, untag_game, update_game,
};
pub use schema::{open, open_in_memory, SCHEMA_VERSION};
