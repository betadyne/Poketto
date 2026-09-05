import { createSignal, For, Show, onCleanup } from "solid-js";
import {
  IconDeviceGamepad2,
  IconClock,
  IconStar,
  IconStarFilled,
  IconExternalLink,
  IconChevronDown,
  IconAlertCircle,
  IconX,
} from "@tabler/icons-solidjs";
import { open } from "@tauri-apps/plugin-shell";
import type {
  Game,
  VndbVnDetail,
  VndbUserListItem,
  VndbImage,
} from "../../types";
import { STATUS_LABELS, LENGTH_NAMES, VOTE_LABELS } from "../../constants";
import { stripBBCode } from "../../utils";

interface GameInfoTabProps {
  game: Game;
  vnDetail: VndbVnDetail;
  userVn: VndbUserListItem | null;
  isVndbConnected: boolean;
  shouldBlur: (img: VndbImage | null) => boolean;
  formatPlayTime: (m: number) => string;
  formatLastPlayed: (timestamp: string | null) => string;
  onSetStatus: (labelId: number) => void;
  onSetVote: (vote: number) => void;
}

export function GameInfoTab(props: GameInfoTabProps) {
  const [showStatusDropdown, setShowStatusDropdown] = createSignal(false);
  const [showVoteDropdown, setShowVoteDropdown] = createSignal(false);
  const [errorMessage, setErrorMessage] = createSignal<string | null>(null);

  let errorTimeout: ReturnType<typeof setTimeout> | undefined;
  const showError = (message: string) => {
    clearTimeout(errorTimeout);
    setErrorMessage(message);
    errorTimeout = setTimeout(() => setErrorMessage(null), 4000);
  };
  onCleanup(() => clearTimeout(errorTimeout));

  const handleVoteClick = () => {
    if (!props.isVndbConnected) {
      showError("You need to connect to VNDB first");
      return;
    }
    setShowVoteDropdown(!showVoteDropdown());
  };

  const handleStatusClick = () => {
    if (!props.isVndbConnected) {
      showError("You need to connect to VNDB first");
      return;
    }
    setShowStatusDropdown(!showStatusDropdown());
  };

  return (
    <>
      <Show when={errorMessage()}>
        <div class="fixed top-32 left-1/2 z-[100] animate-fade-in-down">
          <div class="flex items-center gap-3 px-5 py-3.5 bg-[#FEF2F2] rounded-2xl shadow-xl border border-[var(--color-danger)]/20">
            <IconAlertCircle class="w-5 h-5 text-[var(--color-danger)] flex-shrink-0" strokeWidth={1.5} />
            <span class="text-[var(--color-danger)] font-medium text-sm">
              {errorMessage()}
            </span>
            <button
              onClick={() => setErrorMessage(null)}
              class="ml-2 p-1 hover:bg-[var(--color-danger)]/20 rounded-lg transition-colors"
            >
              <IconX class="w-4 h-4 text-[var(--color-danger)]" strokeWidth={1.5} />
            </button>
          </div>
        </div>
      </Show>

      <div class="max-w-6xl mx-auto space-y-8">
        <div>
          <h1 class="text-[64px] leading-tight font-extrabold text-[var(--color-text-primary)] tracking-tight">
            {props.game.title}
          </h1>
        </div>

        <Show when={props.vnDetail.developers?.length}>
          <div class="flex flex-wrap items-center gap-2">
            <span class="text-[var(--color-text-secondary)] text-sm font-medium">
              Developers :{" "}
            </span>
            <For each={props.vnDetail.developers}>
              {(dev) => (
                <span class="px-3 py-1.5 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm font-medium hover:bg-[var(--color-border)] transition-colors">
                  {dev.name}
                </span>
              )}
            </For>
          </div>
        </Show>

        <div class="flex gap-10">
          <div class="w-[300px] shrink-0">
            <div class="aspect-[2/3] w-full rounded-[24px] overflow-hidden shadow-xl bg-[var(--color-bg-secondary)] relative group">
              <Show
                when={props.vnDetail.image?.url}
                fallback={
                  <div class="flex items-center justify-center h-full">
                    <IconDeviceGamepad2 class="w-20 h-20 text-[var(--color-icon)]" strokeWidth={1.5} />
                  </div>
                }
              >
                <img
                  src={props.vnDetail.image!.url}
                  alt={props.vnDetail.title}
                  class={`w-full h-full object-cover ${
                    props.shouldBlur(props.vnDetail.image ?? null)
                      ? "blur-xl scale-110"
                      : ""
                  }`}
                />
              </Show>
              <Show when={props.vnDetail.rating}>
                <div class="absolute top-4 right-4 px-3 py-1.5 bg-black/80 rounded-full flex items-center gap-1.5 shadow-xl">
                  <IconStarFilled class="w-4 h-4 text-yellow-400 fill-current" />
                  <span class="text-white font-bold">
                    {(props.vnDetail.rating! / 10).toFixed(2)}
                  </span>
                </div>
              </Show>
            </div>
          </div>

          <div class="flex-1 space-y-8">
            <div class="grid grid-cols-3 gap-4">
              <div class="relative bg-[var(--color-bg-secondary)] p-4 rounded-[20px] flex flex-col gap-1 items-start group hover:bg-[var(--color-border)] transition-colors">
                <div class="flex items-center gap-2 text-[var(--color-text-secondary)] text-sm font-medium">
                  <IconStar class="w-4 h-4" strokeWidth={1.5} /> Your Vote
                </div>
                <button
                  onClick={handleVoteClick}
                  class="flex items-center gap-2 w-full text-left"
                >
                  <span class="text-2xl font-bold text-[var(--color-text-primary)]">
                    {props.userVn?.vote
                      ? (props.userVn.vote / 10).toFixed(1)
                      : "Rate..."}
                  </span>
                  <IconChevronDown
                    class={`w-5 h-5 text-[var(--color-icon)] transition-transform ${showVoteDropdown() ? "rotate-180" : ""}`}
                    strokeWidth={1.5}
                  />
                </button>
                <Show when={showVoteDropdown()}>
                  <div class="absolute top-full left-0 right-0 mt-2 z-50 bg-[var(--color-bg-primary)] rounded-2xl shadow-xl overflow-hidden">
                    <div class="max-h-64 overflow-y-auto custom-scrollbar">
                      <button
                        onClick={() => {
                          props.onSetVote(0);
                          setShowVoteDropdown(false);
                        }}
                        class="w-full px-4 py-3 text-left text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-secondary)] hover:text-[var(--color-text-primary)] transition-colors flex items-center gap-3 border-b border-[var(--color-border)]"
                      >
                        <span class="text-lg">—</span>
                        <span class="text-sm">No Rating</span>
                      </button>
                      <For each={[100, 90, 80, 70, 60, 50, 40, 30, 20, 10]}>
                        {(v) => (
                          <button
                            onClick={() => {
                              props.onSetVote(v);
                              setShowVoteDropdown(false);
                            }}
                            class={`w-full px-4 py-3 text-left hover:bg-[var(--color-bg-secondary)] transition-colors flex items-center gap-3 ${props.userVn?.vote === v ? "bg-[var(--color-accent)]/10 text-[var(--color-accent)]" : "text-[var(--color-text-primary)] hover:text-[var(--color-text-primary)]"}`}
                          >
                            <IconStar
                              class={`w-4 h-4 ${v >= 80 ? "text-yellow-400 fill-yellow-400" : v >= 60 ? "text-yellow-400" : "text-[var(--color-icon)]"}`}
                              strokeWidth={1.5}
                            />
                            <span class="text-lg font-bold">
                              {(v / 10).toFixed(1)}
                            </span>
                            <span class="text-xs text-[var(--color-text-tertiary)] ml-auto">
                              {VOTE_LABELS[v] || ""}
                            </span>
                          </button>
                        )}
                      </For>
                    </div>
                  </div>
                </Show>
              </div>

              <div class="relative bg-[var(--color-bg-secondary)] p-4 rounded-[20px] flex flex-col gap-1 items-start hover:bg-[var(--color-border)] transition-colors">
                <div class="flex items-center gap-2 text-[var(--color-text-secondary)] text-sm font-medium">
                  <IconDeviceGamepad2 class="w-4 h-4" strokeWidth={1.5} /> Status
                </div>
                <button
                  onClick={handleStatusClick}
                  class="flex items-center gap-2 w-full text-left"
                >
                  <span class="text-2xl font-bold text-[var(--color-text-primary)]">
                    {props.userVn?.labels?.[0]?.label || "Set Status"}
                  </span>
                  <IconChevronDown
                    class={`w-5 h-5 text-[var(--color-icon)] transition-transform ${showStatusDropdown() ? "rotate-180" : ""}`}
                    strokeWidth={1.5}
                  />
                </button>
                <Show when={showStatusDropdown()}>
                  <div class="absolute top-full left-0 right-0 mt-2 z-50 bg-[var(--color-bg-primary)] rounded-2xl shadow-xl overflow-hidden">
                    <For each={STATUS_LABELS}>
                      {(label) => (
                        <button
                          onClick={() => {
                            props.onSetStatus(label.id);
                            setShowStatusDropdown(false);
                          }}
                          class={`w-full px-4 py-3 text-left hover:bg-[var(--color-bg-secondary)] transition-colors flex items-center gap-3 ${props.userVn?.labels?.some((l) => l.id === label.id) ? "bg-[var(--color-accent)]/10 text-[var(--color-accent)]" : "text-[var(--color-text-primary)] hover:text-[var(--color-text-primary)]"}`}
                        >
                          <div
                            class={`w-2 h-2 rounded-full ${label.id === 1 ? "bg-[var(--color-success)]" : label.id === 2 ? "bg-[var(--color-accent)]" : label.id === 3 ? "bg-yellow-400" : label.id === 4 ? "bg-[var(--color-danger)]" : label.id === 5 ? "bg-[var(--color-accent)]" : "bg-[var(--color-icon)]"}`}
                          />
                          <span class="font-medium">{label.name}</span>
                          <Show
                            when={props.userVn?.labels?.some(
                              (l) => l.id === label.id,
                            )}
                          >
                            <span class="ml-auto text-[var(--color-accent)] text-xs">
                              ✓
                            </span>
                          </Show>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>

              <div class="bg-[var(--color-bg-secondary)] p-4 rounded-[20px] flex flex-col gap-1 items-start hover:bg-[var(--color-border)] transition-colors">
                <div class="flex items-center gap-2 text-[var(--color-text-secondary)] text-sm font-medium">
                  <IconClock class="w-4 h-4" strokeWidth={1.5} /> Total Playtime
                </div>
                <div class="text-2xl font-bold text-[var(--color-text-primary)]">
                  {props.formatPlayTime(props.game.play_time)}
                </div>
              </div>
            </div>

            <div class="bg-[var(--color-bg-secondary)] p-4 rounded-[20px] flex flex-col gap-1 items-start hover:bg-[var(--color-border)] transition-colors">
              <div class="flex items-center gap-2 text-[var(--color-text-secondary)] text-sm font-medium">
                  <IconClock class="w-4 h-4" strokeWidth={1.5} /> Last Played
              </div>
              <div class="text-2xl font-bold text-[var(--color-text-primary)]">
                {props.formatLastPlayed(props.game.last_played ?? null)}
              </div>
            </div>

            <div class="flex flex-wrap gap-x-8 gap-y-2 text-[var(--color-text-secondary)] font-light text-lg">
              <Show when={props.vnDetail.length}>
                <span class="flex items-center gap-2">
                  Length:{" "}
                  <span class="text-[var(--color-text-primary)] font-normal">
                    {LENGTH_NAMES[props.vnDetail.length!]}
                  </span>
                </span>
              </Show>
            </div>

            <div class="font-['Nunito_Sans'] text-lg leading-relaxed text-[var(--color-text-secondary)] whitespace-pre-line max-w-3xl">
              {stripBBCode(props.vnDetail.description ?? "")}
            </div>

            <button
              onClick={() => open(`https://vndb.org/${props.vnDetail.id}`)}
              class="inline-flex items-center gap-2 px-5 py-2.5 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-xl text-[var(--color-text-primary)] font-medium transition-colors shadow-sm hover:shadow group"
            >
              <IconExternalLink class="w-4 h-4 group-hover:scale-110 transition-transform" strokeWidth={1.5} />
              View on VNDB
            </button>

            <Show when={props.vnDetail.tags?.length}>
              <div>
                <h3 class="text-sm uppercase tracking-wider text-[var(--color-text-tertiary)] font-bold mb-3 font-['Nunito']">
                  Tags
                </h3>
                <div class="flex flex-wrap gap-2">
                  <For
                    each={props.vnDetail
                      .tags!.filter((t) => t.spoiler === 0)
                      .slice(0, 15)}
                  >
                    {(tag) => (
                      <span class="px-3 py-1.5 rounded-lg bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)] text-sm font-['Nunito_Sans'] hover:bg-[var(--color-border)] transition-colors cursor-default">
                        {tag.name}
                      </span>
                    )}
                  </For>
                  <Show when={props.vnDetail.tags!.length > 15}>
                    <span class="px-3 py-1.5 text-[var(--color-text-tertiary)] text-sm font-medium">
                      +{props.vnDetail.tags!.length - 15} more
                    </span>
                  </Show>
                </div>
              </div>
            </Show>
          </div>
        </div>
      </div>
    </>
  );
}
