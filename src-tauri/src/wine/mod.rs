mod classify;
mod detect;
mod prefix;
mod scan;
mod steam;

pub use detect::validate_wine_binary;
pub use prefix::{ensure_prefix_exists, get_default_prefix_path, get_global_prefix_path};
pub use scan::{get_all_wine_versions, get_default_wine};
pub use steam::{get_steam_path, get_steam_run_path, is_steam_runtime_available};
