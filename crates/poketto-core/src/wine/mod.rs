mod classify;
pub mod detect;
mod error;
mod prefix;
pub mod scan;
mod steam;

pub use classify::{classify_wine_type, format_version_name, format_wine_name};
pub use detect::{detect_proton_in_folder, detect_wine_environments, detect_wine_in_folder, query_wine_version, validate_wine_binary};
pub use error::{WineError, WineResult};
pub use prefix::{default_prefix_path, ensure_prefix_exists, global_prefix_path};
pub use scan::{get_all_wine_versions, get_default_wine, scan_common_prefixes};
pub use steam::{get_steam_path, get_steam_run_path, is_steam_runtime_available, library_paths_from_vdf, steam_library_folders};
