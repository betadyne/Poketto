use poketto_core::db::{self, Connection, DbResult};
use poketto_core::models::Game;
use slint::VecModel;

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

pub fn filter_games<'a>(games: &'a [Game], filter: LibraryFilter, query: &str) -> Vec<&'a Game> {
    let terms: Vec<String> = query.split_whitespace().map(|term| term.to_lowercase()).collect();
    games
        .iter()
        .filter(|game| match filter {
            LibraryFilter::All => !game.is_hidden,
            LibraryFilter::Unfinished => !game.is_hidden && !game.is_finished,
            LibraryFilter::Finished => !game.is_hidden && game.is_finished,
            LibraryFilter::Hidden => game.is_hidden,
        })
        .filter(|game| {
            let title = game.title.to_lowercase();
            terms.iter().all(|term| title.contains(term))
        })
        .collect()
}

pub fn card_data(game: &Game) -> GameCardData {
    GameCardData {
        id: game.id.clone().into(),
        title: game.title.clone().into(),
        rating: game.rating.unwrap_or(0.0) as f32,
        show_cover: false,
        cover: slint::Image::default(),
    }
}

pub fn refresh_library(
    model: &VecModel<GameCardData>,
    conn: &Connection,
    filter: LibraryFilter,
    query: &str,
) -> DbResult<Vec<Game>> {
    let games = db::get_all_games(conn)?;
    let visible: Vec<Game> = filter_games(&games, filter, query)
        .into_iter()
        .cloned()
        .collect();
    let cards: Vec<GameCardData> = visible.iter().map(card_data).collect();
    model.set_vec(cards);
    Ok(visible)
}

#[cfg(test)]
mod tests {
    use super::*;
    use slint::Model;
    use poketto_core::models::GameType;

    fn game(id: &str, title: &str, finished: bool, hidden: bool) -> Game {
        Game {
            id: id.to_string(),
            title: title.to_string(),
            path: format!("/games/{id}"),
            vndb_id: None,
            cover_url: None,
            play_time_minutes: 0,
            is_finished: finished,
            last_played: None,
            is_hidden: hidden,
            show_spoilers: false,
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
        let shown = filter_games(&games, LibraryFilter::All, "");
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].id, "a");
    }

    #[test]
    fn status_filters_split_finished() {
        let games = vec![
            game("a", "Alpha", true, false),
            game("b", "Beta", false, false),
            game("c", "Gamma", true, true),
        ];
        assert_eq!(filter_games(&games, LibraryFilter::Finished, "").len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Unfinished, "").len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::Hidden, "").len(), 1);
    }

    #[test]
    fn query_matches_case_insensitive_terms() {
        let games = vec![
            game("a", "Muv-Luv Alternative", false, false),
            game("b", "Steins;Gate", false, false),
        ];
        assert_eq!(filter_games(&games, LibraryFilter::All, "muv").len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::All, "MUV LUV").len(), 1);
        assert_eq!(filter_games(&games, LibraryFilter::All, "gate muv").len(), 0);
    }

    #[test]
    fn card_maps_rating_and_placeholders() {
        let card = card_data(&game("a", "Alpha", false, false));
        assert_eq!(card.title.as_str(), "Alpha");
        assert_eq!(card.rating, 8.5);
        assert_eq!(card.show_cover, false);
    }

    #[test]
    fn refresh_reads_database_through_filter() {
        let conn = db::open_in_memory().expect("open");
        db::insert_game(&conn, &game("a", "Alpha", false, false)).expect("insert");
        db::insert_game(&conn, &game("b", "Beta", true, false)).expect("insert");
        let model = VecModel::default();
        refresh_library(&model, &conn, LibraryFilter::Finished, "").expect("refresh");
        assert_eq!(model.row_count(), 1);
        assert_eq!(model.row_data(0).expect("row").title.as_str(), "Beta");
    }
}
