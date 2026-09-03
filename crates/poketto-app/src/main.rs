slint::include_modules!();

mod adapters;

use std::cell::RefCell;
use std::rc::Rc;

use slint::{ModelRc, VecModel};

use adapters::LibraryFilter;

fn data_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Poketto")
}

fn refresh(
    app: &AppWindow,
    model: &VecModel<GameCardData>,
    conn: &poketto_core::db::Connection,
    filter: LibraryFilter,
) {
    let query = app.get_query().to_string();
    if let Err(e) = adapters::refresh_library(model, conn, filter, &query) {
        tracing::warn!("library refresh failed: {e}");
    }
}

fn main() -> Result<(), slint::PlatformError> {
    let dir = data_dir();
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!("data dir unavailable: {e}");
    }
    let conn = Rc::new(RefCell::new(
        poketto_core::db::open(&dir.join("library.db")).expect("open library database"),
    ));

    let app = AppWindow::new()?;
    let model: Rc<VecModel<GameCardData>> = Rc::new(VecModel::default());
    app.set_games(ModelRc::from(model.clone()));
    let filter = Rc::new(RefCell::new(LibraryFilter::default()));

    refresh(&app, &model, &conn.borrow(), *filter.borrow());
    {
        let _handle = app.as_weak();
        let _model = model.clone();
        let conn = conn.clone();
        let _filter = filter.clone();
        app.on_game_clicked(move |id| {
            let title = poketto_core::db::get_game(&conn.borrow(), id.as_str())
                .ok()
                .flatten()
                .map(|game| game.title)
                .unwrap_or_else(|| id.to_string());
            tracing::info!(game_id = id.as_str(), title = title.as_str(), "game launch requested");
        });
    }
    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        app.on_search_accepted(move |_| {
            if let Some(app) = handle.upgrade() {
                refresh(&app, &model, &conn.borrow(), *filter.borrow());
            }
        });
    }

    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        app.on_filter_changed(move |index| {
            if let Some(app) = handle.upgrade() {
                *filter.borrow_mut() = LibraryFilter::from_index(index);
                app.set_active_filter(index);
                refresh(&app, &model, &conn.borrow(), *filter.borrow());
            }
        });
    }

    app.run()
}

