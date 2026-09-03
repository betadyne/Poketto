slint::include_modules!();

mod adapters;
mod image_loader;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use slint::{Model, ModelRc, VecModel};

use adapters::LibraryFilter;
use image_loader::{ImageLoader, LoadedCover};

fn data_dir() -> std::path::PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("Poketto")
}

fn cover_dir() -> std::path::PathBuf {
    dirs::cache_dir()
        .unwrap_or_else(data_dir)
        .join("poketto")
        .join("covers")
}

fn refresh(
    app: &AppWindow,
    model: &VecModel<GameCardData>,
    conn: &poketto_core::db::Connection,
    filter: LibraryFilter,
    loader: &ImageLoader,
) {
    let query = app.get_query().to_string();
    match adapters::refresh_library(model, conn, filter, &query) {
        Ok(games) => {
            loader.next_generation();
            for game in &games {
                if let Some(url) = game.cover_url.as_deref() {
                    loader.request(&game.id, url);
                }
            }
        }
        Err(e) => tracing::warn!("library refresh failed: {e}"),
    }
}

fn apply_cover(model: &VecModel<GameCardData>, loaded: &LoadedCover) {
    let Some(image) = &loaded.image else {
        return;
    };
    for row in 0..model.row_count() {
        let Some(entry) = model.row_data(row) else {
            continue;
        };
        if entry.id.as_str() == loaded.game_id {
            let buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
                &image.pixels,
                image.width,
                image.height,
            );
            model.set_row_data(
                row,
                GameCardData {
                    cover: slint::Image::from_rgba8(buffer),
                    show_cover: true,
                    ..entry
                },
            );
            return;
        }
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

    let rt = tokio::runtime::Runtime::new().expect("start background runtime");
    let loader = Rc::new(ImageLoader::new(rt.handle(), cover_dir()).expect("start image loader"));

    let app = AppWindow::new()?;
    let model: Rc<VecModel<GameCardData>> = Rc::new(VecModel::default());
    app.set_games(ModelRc::from(model.clone()));
    let filter = Rc::new(RefCell::new(LibraryFilter::default()));

    refresh(&app, &model, &conn.borrow(), *filter.borrow(), &loader);
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
        let loader = loader.clone();
        app.on_search_accepted(move |_| {
            if let Some(app) = handle.upgrade() {
                refresh(&app, &model, &conn.borrow(), *filter.borrow(), &loader);
            }
        });
    }

    {
        let handle = app.as_weak();
        let model = model.clone();
        let conn = conn.clone();
        let filter = filter.clone();
        let loader = loader.clone();
        app.on_filter_changed(move |index| {
            if let Some(app) = handle.upgrade() {
                *filter.borrow_mut() = LibraryFilter::from_index(index);
                app.set_active_filter(index);
                refresh(&app, &model, &conn.borrow(), *filter.borrow(), &loader);
            }
        });
    }

    let drain_model = model.clone();
    let drain_loader = loader.clone();
    let cover_timer = slint::Timer::default();
    cover_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(120),
        move || {
            for loaded in drain_loader.poll() {
                apply_cover(&drain_model, &loaded);
            }
        },
    );
    app.run()
}
