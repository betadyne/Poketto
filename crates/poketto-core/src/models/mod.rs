mod game;
mod playtime;
mod settings;
mod tag;
mod vndb;

pub use game::{is_user_status, is_user_vote, Game, GameType, WineSettings, WineSource, WineType, WineVersion, USER_STATUS_DROPPED, USER_STATUS_FINISHED, USER_STATUS_NONE, USER_STATUS_PLAYING, USER_STATUS_STALLED, USER_VOTE_MAX, USER_VOTE_UNRATED};
pub use playtime::{DailyPlaytimeData, PlaySession};
pub use settings::AppSettings;
pub use tag::Tag;
pub use vndb::{
    VndbAuthInfo, VndbCharacter, VndbCharacterVn, VndbImage, VndbLabel, VndbProducer, VndbResponse,
    VndbSearchResult, VndbTag, VndbTrait, VndbUserListItem, VndbVnDetail,
};
