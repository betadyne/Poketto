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
  CustomPresence,
  PresenceActivityType,
  PresenceTimestampMode,
} from "./bindings";

export interface GameExitedPayload {
  game_id: string;
  play_minutes: number;
}

export interface PlaytimeUpdatedPayload {
  game_id: string;
  duration_seconds: number;
}
