export type {
  GameMetadata as Game,
  VndbImage,
  VndbSearchResult,
  VndbTag,
  VndbProducer,
  VndbVnDetail,
  VndbTrait,
  VndbCharacterVn,
  VndbCharacter,
  VndbUserListItem,
  VndbLabel,
  VndbAuthInfo,
  AppSettings,
  DailyPlaytimeData,
  // Wine types
  WineType,
  GameType,
  WineVersion,
  WineSettings,
} from "./bindings";

export interface GameExitedPayload {
  game_id: string;
  play_minutes: number;
}
