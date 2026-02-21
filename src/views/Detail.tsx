import { Show } from "solid-js";
import { Play, Clock, Loader2 } from "lucide-solid";
import { Sidebar } from "../components/Sidebar";
import { GameInfoTab } from "../components/detail/GameInfoTab";
import { CharacterList } from "../components/detail/CharacterList";
import type {
  Game,
  VndbVnDetail,
  VndbCharacter,
  VndbUserListItem,
  AppSettings,
  VndbImage,
} from "../types";

interface DetailProps {
  page: "detail" | "detail-chars";
  setPage: (page: "detail" | "detail-chars") => void;
  game: Game;
  vnDetail: VndbVnDetail;
  characters: VndbCharacter[];
  userVn: VndbUserListItem | null;
  runningGame: string | null;
  settings: AppSettings;
  showSpoilers: boolean;
  setShowSpoilers: (show: boolean) => void;
  isRefreshing: boolean;
  onBack: () => void;
  onRefresh: () => void;
  onLaunchGame: (id: string) => void;
  onSetStatus: (labelId: number) => void;
  onSetVote: (vote: number) => void;
  formatPlayTime: (m: number) => string;
  formatLastPlayed: (timestamp: string | null) => string;
  shouldBlur: (img: VndbImage | null) => boolean;
}

export function Detail(props: DetailProps) {
  return (
    <div class="flex h-full bg-[var(--color-bg-primary)] font-['Nunito_Sans'] text-[var(--color-text-primary)] overflow-hidden">
      <Sidebar
        onBack={props.onBack}
        onRefresh={props.onRefresh}
        showSpoilers={props.showSpoilers}
        onToggleSpoilers={() => props.setShowSpoilers(!props.showSpoilers)}
      />

      <div class="flex-1 flex flex-col min-w-0 overflow-hidden relative">
        <Show when={props.isRefreshing}>
          <div class="absolute inset-0 bg-white/90 flex items-center justify-center z-40">
            <div class="flex flex-col items-center gap-4">
              <Loader2 class="w-12 h-12 text-[var(--color-icon)] animate-spin" />
              <span class="text-[var(--color-text-secondary)] text-lg font-medium">
                Refreshing data...
              </span>
            </div>
          </div>
        </Show>

        <div class="flex items-center gap-8 px-8 py-6 z-10">
          <div class="flex items-center gap-1 bg-[var(--color-bg-secondary)] p-1 rounded-full">
            <button
              onClick={() => props.setPage("detail")}
              class={`px-6 py-2 rounded-full text-sm font-bold transition-colors ${
                props.page === "detail"
                  ? "bg-[var(--color-accent)] text-white shadow-lg"
                  : "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
              }`}
            >
              Game Info
            </button>
            <button
              onClick={() => props.setPage("detail-chars")}
              class={`px-6 py-2 rounded-full text-sm font-bold transition-colors ${
                props.page === "detail-chars"
                  ? "bg-[var(--color-accent)] text-white shadow-lg"
                  : "text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]"
              }`}
            >
              Characters
            </button>
          </div>

          <div class="flex-1"></div>

          <Show when={props.game}>
            <Show
              when={props.runningGame !== props.game.id}
              fallback={
                <button class="px-6 py-2.5 bg-[var(--color-success-light)] text-[var(--color-success)] rounded-full font-bold text-sm tracking-wide flex items-center gap-2 cursor-default">
                  <Clock class="w-4 h-4" /> RUNNING
                </button>
              }
            >
              <button
                onClick={() => props.onLaunchGame(props.game.id)}
                class="group relative px-8 py-2.5 bg-[var(--color-accent)] text-white rounded-full font-bold text-sm tracking-wide overflow-hidden shadow-lg hover:shadow-xl transition-shadow"
              >
                <span class="relative z-10 flex items-center gap-2">
                  <Play class="w-4 h-4 fill-current" /> PLAY NOW
                </span>
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent -translate-x-full group-hover:translate-x-full transition-transform duration-500"></div>
              </button>
            </Show>
          </Show>
        </div>

        <div class="flex-1 overflow-y-auto px-8 pb-8 custom-scrollbar z-10">
          <Show when={props.page === "detail"}>
            <GameInfoTab
              game={props.game}
              vnDetail={props.vnDetail}
              userVn={props.userVn}
              isVndbConnected={!!props.settings.vndb_token}
              shouldBlur={props.shouldBlur}
              formatPlayTime={props.formatPlayTime}
              formatLastPlayed={props.formatLastPlayed}
              onSetStatus={props.onSetStatus}
              onSetVote={props.onSetVote}
            />
          </Show>

          <Show when={props.page === "detail-chars"}>
            <CharacterList
              characters={props.characters}
              vnId={props.vnDetail.id}
              showSpoilers={props.showSpoilers}
              shouldBlur={props.shouldBlur}
            />
          </Show>
        </div>
      </div>
    </div>
  );
}
