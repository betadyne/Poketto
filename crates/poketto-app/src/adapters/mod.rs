mod detail;
mod editor;
mod games;

pub use detail::{assemble_detail, presence_buttons, DetailPayload};
pub use editor::{apply_form, platform_index, validate_game_form};
pub use games::{refresh_library, LibraryFilter};
