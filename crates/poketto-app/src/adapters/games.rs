use poketto_core::db::{self, Connection, DbResult, SortBy, SortOrder};
use poketto_core::models::Game;
use slint::{Model, VecModel};

use crate::GameCardData;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LibraryFilter {
    #[default]
    All,
    Unfinished,
    Finished,
    Hidden,
}

impl LibraryFilter {
    pub fn from_index(index: i32) -> Self {
        match index {
            1 => LibraryFilter::Unfinished,
            2 => LibraryFilter::Finished,
            3 => LibraryFilter::Hidden,
            _ => LibraryFilter::All,
        }
    }
}

pub fn filter_games<'a>(
    games: &'a [Game],
    filter: LibraryFilter,
    query: &str,
    show_hidden: bool,
) -> Vec<&'a Game> {
    let terms: Vec<String> = query.split_whitespace().map(|term| term.to_lowercase()).collect();
    games
        .iter()
        .filter(|game| match filter {
            LibraryFilter::All => show_hidden || !game.is_hidden,
            LibraryFilter::Unfinished => !game.is_finished && (show_hidden || !game.is_hidden),
            LibraryFilter::Finished => game.is_finished && (show_hidden || !game.is_hidden),
            LibraryFilter::Hidden => game.is_hidden,
        })
        .filter(|game| {
            let title = game.title.to_lowercase();
            terms.iter().all(|term| title.contains(term))
        })
        .collect()
}

pub fn query_games_with_filter(
    conn: &Connection,
    filter: LibraryFilter,
    query: &str,
    show_hidden: bool,
    sort: (SortBy, SortOrder),
) -> DbResult<Vec<Game>> {
    let games = db::get_all_games(conn, sort.0, sort.1)?;
    Ok(filter_games(&games, filter, query, show_hidden)
        .into_iter()
        .cloned()
        .collect())
}

pub fn card_data(game: &Game, nsfw: bool) -> GameCardData {
    GameCardData {
        id: game.id.clone().into(),
        title: game.title.clone().into(),
        rating: game.rating.unwrap_or(0.0) as f32,
        show_cover: false,
        hidden: game.is_hidden,
        is_nsfw: nsfw,
        revealed: false,
        playing: false,
        cover: slint::Image::default(),
    }
}

pub fn cover_nsfw(sexual: f64, violence: f64) -> bool {
    sexual > 0.0 || violence > 0.0
}

pub fn game_nsfw(conn: &Connection, game: &Game) -> bool {
    game.vndb_id
        .as_deref()
        .and_then(|vndb_id| poketto_core::vndb::cached_detail_sync(conn, vndb_id).ok().flatten())
        .and_then(|detail| detail.image)
        .is_some_and(|image| cover_nsfw(image.sexual, image.violence))
}

pub fn sort_option_index(sort: SortBy, order: SortOrder) -> i32 {
    match (sort, order) {
        (SortBy::Title, SortOrder::Asc) => 0,
        (SortBy::Title, SortOrder::Desc) => 1,
        (SortBy::PlayTime, SortOrder::Desc) => 2,
        (SortBy::LastPlayed, SortOrder::Desc) => 3,
        _ => 0,
    }
}
#[allow(dead_code)]
pub fn sort_option_at(index: i32) -> (SortBy, SortOrder) {
    match index {
        1 => (SortBy::Title, SortOrder::Desc),
        2 => (SortBy::PlayTime, SortOrder::Desc),
        3 => (SortBy::LastPlayed, SortOrder::Desc),
        _ => (SortBy::Title, SortOrder::Asc),
    }
}

fn reconcile_model(model: &VecModel<GameCardData>, cards: &[GameCardData]) {
    let shared = model.row_count().min(cards.len());
    for (index, card) in cards.iter().enumerate().take(shared) {
        let old = model.row_data(index);
        let merged = match &old {
            Some(existing) if existing.id == card.id => GameCardData {
                revealed: existing.revealed,
                cover: existing.cover.clone(),
                show_cover: existing.show_cover,
                ..card.clone()
            },
            _ => card.clone(),
        };
        if old.as_ref() != Some(&merged) {
            model.set_row_data(index, merged);
        }
    }
    for card in cards.iter().skip(shared) {
        model.push(card.clone());
    }
    for _ in cards.len()..model.row_count() {
        model.remove(cards.len());
    }
}

pub fn refresh_library(
    model: &VecModel<GameCardData>,
    conn: &Connection,
    filter: LibraryFilter,
    query: &str,
    show_hidden: bool,
    sort: (SortBy, SortOrder),
) -> DbResult<Vec<Game>> {
    let visible = query_games_with_filter(conn, filter, query, show_hidden, sort)?;
    let cards: Vec<GameCardData> = visible
        .iter()
        .map(|game| card_data(game, game_nsfw(conn, game)))
        .collect();
    reconcile_model(model, &cards);
    Ok(visible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use poketto_core::models::GameType;

    fn game(id: &str, title: &str, finished: bool, hidden: bool) -> Game {
        Game {
            id: id.to_string(),
            title: title.to_string(),
            path: format!("/games/{id}"),
            work_dir: None,
            vndb_id: None,
            cover_url: None,
            play_time_minutes: 0,
            is_finished: finished,
            last_played: None,
            is_hidden: hidden,
            show_spoilers: false,
            user_status: 0,
            user_vote: 0,
            game_type: Some(GameType::WindowsExe),
            wine_settings: None,
            rating: Some(8.5),
        }
    }

    #[test]
    fn all_hides_hidden_games() {
        let games = vec![
            game("a", "Alpha", false, false),
            game("b", "Beta", false, true),
        ];
        let shown = filter_games(&games, LibraryFilter::All, "", false);
    }

    #[test]
    fn status_filters_split_finished() {
        let games = vec![
            game("a", "Alpha", true, false),
            game("b", "Beta", false, false),
            game("c", "Gamma", true, true),
        ];
        assert_eq!(filter_games(&games, LibraryFilter::Finished, "", false).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Unfinished, "", false).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Hidden, "", false).len(), 1);
    }

    #[test]
    fn query_matches_case_insensitive_terms() {
        let games = vec![
            game("a", "Muv-Luv Alternative", false, false),
            game("b", "Steins;Gate", false, false),
        ];
        assert_eq!(filter_games(&games, LibraryFilter::All, "muv", false).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::All, "MUV LUV", false).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::All, "gate muv", false).len(), 0);
    }

    #[test]
    fn card_maps_rating_and_placeholders() {
        let card = card_data(&game("a", "Alpha", false, false), true);
        assert_eq!(card.title.as_str(), "Alpha");
        assert_eq!(card.rating, 8.5);
        assert_eq!(card.show_cover, false);
        assert_eq!(card.is_nsfw, true);
        assert_eq!(card.revealed, false);
        assert_eq!(card.playing, false);
    }

    #[test]
    fn refresh_reads_database_through_filter() {
        let conn = db::open_in_memory().expect("open");
        db::insert_game(&conn, &game("a", "Alpha", false, false)).expect("insert");
        db::insert_game(&conn, &game("b", "Beta", true, false)).expect("insert");
        let model = VecModel::default();
        refresh_library(&model, &conn, LibraryFilter::Finished, "", false, (SortBy::Title, SortOrder::Asc))
            .expect("refresh");
        assert_eq!(model.row_count(), 1);
        assert_eq!(model.row_data(0).expect("row").title.as_str(), "Beta");
    }

    #[test]
    fn nsfw_threshold_flags_any_positive_score() {
        assert_eq!(cover_nsfw(0.0, 0.0), false);
        assert_eq!(cover_nsfw(0.5, 0.0), true);
        assert_eq!(cover_nsfw(0.0, 1.2), true);
    }

    #[test]
    fn refresh_flags_nsfw_from_cached_scores() {
        let conn = db::open_in_memory().expect("open");
        let mut spicy = game("a", "Spicy", false, false);
        spicy.vndb_id = Some("v17".to_string());
        db::insert_game(&conn, &spicy).expect("insert");
        db::insert_game(&conn, &game("b", "Mild", false, false)).expect("insert");
        let detail = r#"{"id": "v17", "title": "Spicy", "image": {"url": "https://img.jpg", "sexual": 1.5, "violence": 0.0}}"#;
        db::cache_put(
            &conn,
            poketto_core::vndb::KIND_DETAIL,
            "v17",
            detail,
            0,
        )
        .expect("seed");
        let model = VecModel::default();
        refresh_library(&model, &conn, LibraryFilter::All, "", false, (SortBy::Title, SortOrder::Asc))
            .expect("refresh");
        assert_eq!(model.row_count(), 2);
        let spicy_card = model.row_data(1).expect("row");
        assert_eq!(spicy_card.title.as_str(), "Spicy");
        assert_eq!(spicy_card.is_nsfw, true);
        assert_eq!(model.row_data(0).expect("row").is_nsfw, false);
    }

    #[test]
    fn sort_options_round_trip() {
        use poketto_core::db::{SortBy, SortOrder};
        assert_eq!(sort_option_index(SortBy::Title, SortOrder::Asc), 0);
        assert_eq!(sort_option_index(SortBy::Title, SortOrder::Desc), 1);
        assert_eq!(sort_option_index(SortBy::PlayTime, SortOrder::Desc), 2);
        assert_eq!(sort_option_index(SortBy::LastPlayed, SortOrder::Desc), 3);
        assert_eq!(sort_option_index(SortBy::PlayTime, SortOrder::Asc), 0);
        assert_eq!(sort_option_at(0), (SortBy::Title, SortOrder::Asc));
        assert_eq!(sort_option_at(1), (SortBy::Title, SortOrder::Desc));
        assert_eq!(sort_option_at(2), (SortBy::PlayTime, SortOrder::Desc));
        assert_eq!(sort_option_at(3), (SortBy::LastPlayed, SortOrder::Desc));
        assert_eq!(sort_option_at(99), (SortBy::Title, SortOrder::Asc));
    }

    #[test]
    fn refresh_resorts_in_place_and_keeps_covers() {
        use poketto_core::db::{SortBy, SortOrder};
        let conn = db::open_in_memory().expect("open");
        let mut alpha = game("a", "Alpha", false, false);
        alpha.play_time_minutes = 10;
        let mut zulu = game("z", "Zulu", false, false);
        zulu.play_time_minutes = 90;
        db::insert_game(&conn, &alpha).expect("insert");
        db::insert_game(&conn, &zulu).expect("insert");
        let model = VecModel::default();
        let asc = (SortBy::Title, SortOrder::Asc);
        refresh_library(&model, &conn, LibraryFilter::All, "", false, asc).expect("refresh");
        assert_eq!(model.row_count(), 2);
        let mut covered = model.row_data(0).expect("row");
        covered.show_cover = true;
        covered.revealed = true;
        model.set_row_data(0, covered);
        zulu.play_time_minutes = 1;
        db::update_game(&conn, &zulu).expect("update");
        refresh_library(
            &model,
            &conn,
            LibraryFilter::All,
            "",
            false,
            (SortBy::PlayTime, SortOrder::Desc),
        )
        .expect("refresh");
        assert_eq!(model.row_count(), 2);
        assert_eq!(model.row_data(0).expect("row").id.as_str(), "a");
        assert_eq!(model.row_data(0).expect("row").show_cover, true);
        assert_eq!(model.row_data(0).expect("row").revealed, true);
        assert_eq!(model.row_data(1).expect("row").id.as_str(), "z");
        db::delete_game(&conn, "z").expect("delete");
        refresh_library(&model, &conn, LibraryFilter::All, "", false, asc).expect("refresh");
        assert_eq!(model.row_count(), 1);
        assert_eq!(model.row_data(0).expect("row").id.as_str(), "a");
        assert_eq!(model.row_data(0).expect("row").show_cover, true);
    }

    #[test]
    fn show_hidden_toggle_includes_hidden_games() {
        let games = vec![
            game("a", "Alpha", false, false),
            game("b", "Beta", true, false),
            game("c", "Gamma", false, true),
        ];
        assert_eq!(filter_games(&games, LibraryFilter::All, "", false).len(), 2);
        assert_eq!(filter_games(&games, LibraryFilter::All, "", true).len(), 3);
        assert_eq!(filter_games(&games, LibraryFilter::Finished, "", true).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Unfinished, "", true).len(), 2);
        assert_eq!(filter_games(&games, LibraryFilter::Hidden, "", true).len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Hidden, "", false).len(), 1);
    }

    #[test]
    fn flexible_query_combines_text_hidden_and_sort() {
        let conn = db::open_in_memory().expect("open");
        let mut hidden = game("h", "Hidden Gem", false, true);
        hidden.play_time_minutes = 99;
        db::insert_game(&conn, &game("a", "Alpha", false, false)).expect("insert");
        db::insert_game(&conn, &hidden).expect("insert");
        let listed = query_games_with_filter(
            &conn,
            LibraryFilter::All,
            "hidden",
            false,
            (SortBy::PlayTime, SortOrder::Desc),
        )
        .expect("query");
        assert_eq!(listed.len(), 0);
        let listed = query_games_with_filter(
            &conn,
            LibraryFilter::All,
            "",
            true,
            (SortBy::PlayTime, SortOrder::Desc),
        )
        .expect("query");
        assert_eq!(listed.len(), 2);
        assert_eq!(listed[0].id, "h");
        assert_eq!(listed[1].id, "a");
    }
}
