import { Show, For, createSignal, createEffect, onCleanup } from "solid-js";
import {
  Plus,
  Gamepad2,
  Clock,
  Search,
  LayoutGrid,
  List,
  Library as LibraryIcon,
  ChevronDown,
  Filter,
  User,
  Check,
  Settings,
  ScrollText,
  LogOut,
} from "lucide-solid";
import { useNavigate } from "@solidjs/router";
import { GameCard } from "../components/GameCard";
import type { Game } from "../types";
import type { SortBy, SortOrder } from "../hooks/useLibraryFilters";

interface LibraryProps {
  games: Game[];
  filteredGames: Game[];
  runningGame: string | null;
  loading: boolean;
  authUser: string | null;
  searchQuery: string;
  onSearchChange: (query: string) => void;
  sortBy: SortBy;
  onSortByChange: (s: SortBy) => void;
  sortOrder: SortOrder;
  onSortOrderChange: (o: SortOrder) => void;
  showHidden: boolean;
  onShowHiddenChange: (v: boolean) => void;
  formatPlayTime: (m: number) => string;
  onAddGame: () => void;
  onStopTracking: () => void;
  onLaunchGame: (id: string) => void;
  onRemoveGame: (id: string) => void;
  onEditSettings: (game: Game) => void;
  onOpenDetail: (game: Game) => void;
  onHideGame: (id: string, hidden: boolean) => void;
}

export function Library(props: LibraryProps) {
  const navigate = useNavigate();
  const [viewMode, setViewMode] = createSignal<"grid" | "list">("grid");
  const [showSortDropdown, setShowSortDropdown] = createSignal(false);
  const [showFiltersDropdown, setShowFiltersDropdown] = createSignal(false);
  const [activeContextMenu, setActiveContextMenu] = createSignal<string | null>(
    null,
  );

  const handleClickOutside = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (!target.closest(".sort-dropdown")) {
      setShowSortDropdown(false);
    }
    if (!target.closest(".filters-dropdown")) {
      setShowFiltersDropdown(false);
    }
  };

  createEffect(() => {
    window.addEventListener("click", handleClickOutside);
    onCleanup(() => window.removeEventListener("click", handleClickOutside));
  });

  const sortOptions = [
    { value: "lastPlayed", label: "Last played" },
    { value: "playTime", label: "Most played" },
    { value: "title", label: "Name" },
  ];

  const getSortLabel = () => {
    return (
      sortOptions.find((o) => o.value === props.sortBy)?.label || "Last played"
    );
  };

  const handleContextMenuOpen = (gameId: string) => {
    setActiveContextMenu(gameId);
  };

  return (
    <div class="flex h-full bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] overflow-hidden font-['Nunito_Sans']">
      <aside class="w-64 bg-[var(--color-bg-primary)] flex flex-col border-r border-[var(--color-border)]">
        <div class="p-6">
          <h1 class="font-bold text-xl text-[var(--color-text-primary)] tracking-tight">
            Poketto
          </h1>
        </div>

        <div class="flex-1 overflow-y-auto px-4 custom-scrollbar flex flex-col gap-4">
          <nav class="space-y-1">
            <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-accent)] flex items-center justify-center">
                <LibraryIcon class="w-5 h-5 text-white" />
              </div>
              <span class="font-medium text-[var(--color-text-primary)]">
                My Games
              </span>
            </button>
          </nav>

          <div class="h-px bg-[var(--color-border)] w-full" />

          <nav class="space-y-1">
            <button
              onClick={() => navigate("/settings")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <Settings class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">Settings</span>
            </button>
            <button
              onClick={() => navigate("/logs")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <ScrollText class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">Logs</span>
            </button>
            <button class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <LogOut class="w-5 h-5 text-[var(--color-icon)]" />
              </div>
              <span class="font-medium">Log out</span>
            </button>
          </nav>
        </div>

        <div class="p-6 text-xs text-[var(--color-text-tertiary)] font-medium text-center">
          Poketto Version: {__APP_VERSION__}
        </div>
      </aside>

      <main class="flex-1 flex flex-col min-w-0 bg-[var(--color-bg-primary)]">
        <header class="h-20 px-8 flex items-center justify-between gap-8 border-b border-[var(--color-border)]">
          <h2 class="text-2xl font-bold text-[var(--color-text-primary)]">
            Overview
          </h2>

          <div class="flex-1 max-w-xl relative group">
            <Search class="absolute left-4 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--color-icon)] group-focus-within:text-[var(--color-accent)] transition-colors" />
            <input
              type="text"
              value={props.searchQuery}
              onInput={(e) => props.onSearchChange(e.currentTarget.value)}
              placeholder="Search game titles..."
              class="w-full pl-12 pr-4 py-3 bg-[var(--color-bg-secondary)] rounded-2xl text-[var(--color-text-primary)] placeholder:text-[var(--color-text-tertiary)] focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)] transition-all font-medium"
            />
          </div>

          <div class="flex items-center gap-4">
            <Show when={props.authUser}>
              <div class="flex items-center gap-2 px-3 py-1.5 bg-[var(--color-bg-secondary)] rounded-xl">
                <User class="w-4 h-4 text-[var(--color-accent)]" />
                <span class="text-sm font-medium text-[var(--color-text-primary)]">
                  {props.authUser}
                </span>
              </div>
            </Show>

            <div class="flex items-center gap-3 pl-4 border-l border-[var(--color-border)]">
              <Show when={props.runningGame}>
                <button
                  onClick={props.onStopTracking}
                  class="flex items-center gap-2 px-4 py-2 bg-[var(--color-danger-light)] text-[var(--color-danger)] hover:bg-[var(--color-danger)] hover:text-white rounded-xl transition-all"
                >
                  <Clock class="w-4 h-4 animate-pulse" />
                  <span class="text-sm font-bold">Stop Game</span>
                </button>
              </Show>

              <button
                onClick={props.onAddGame}
                class="flex items-center gap-2 px-4 py-2.5 bg-[var(--color-accent)] text-white font-bold text-sm rounded-xl hover:bg-[var(--color-accent-hover)] transition-all"
              >
                <Plus class="w-4 h-4" />
                <span>Add Game</span>
              </button>
            </div>
          </div>
        </header>

        <div class="px-8 py-6 flex flex-col gap-6">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="text-[var(--color-text-primary)] font-medium">
                All Games
              </span>
              <span class="text-[var(--color-text-tertiary)]">
                ({props.games.length})
              </span>
            </div>

            <div class="flex items-center gap-4">
              <div class="sort-dropdown relative">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowSortDropdown(!showSortDropdown());
                    setShowFiltersDropdown(false);
                  }}
                  class="flex items-center gap-2 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-xl text-sm hover:bg-[var(--color-border)] transition-colors"
                >
                  <span class="text-[var(--color-text-secondary)]">
                    Sort by
                  </span>
                  <span class="text-[var(--color-text-primary)] font-medium">
                    {getSortLabel()}
                  </span>
                  <ChevronDown
                    class={`w-4 h-4 text-[var(--color-icon)] transition-transform ${showSortDropdown() ? "rotate-180" : ""}`}
                  />
                </button>

                <Show when={showSortDropdown()}>
                  <div class="absolute top-full right-0 mt-2 bg-[var(--color-bg-primary)] rounded-xl shadow-lg overflow-hidden min-w-[160px] z-50">
                    <For each={sortOptions}>
                      {(option) => (
                        <button
                          onClick={() => {
                            props.onSortByChange(option.value as SortBy);
                            setShowSortDropdown(false);
                          }}
                          class={`w-full flex items-center justify-between px-4 py-2.5 text-sm text-left hover:bg-[var(--color-bg-secondary)] transition-colors ${
                            props.sortBy === option.value
                              ? "text-[var(--color-text-primary)]"
                              : "text-[var(--color-text-secondary)]"
                          }`}
                        >
                          <span>{option.label}</span>
                          <Show when={props.sortBy === option.value}>
                            <Check class="w-4 h-4 text-[var(--color-success)]" />
                          </Show>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>

              <div class="filters-dropdown relative">
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setShowFiltersDropdown(!showFiltersDropdown());
                    setShowSortDropdown(false);
                  }}
                  class={`flex items-center gap-2 px-3 py-2 rounded-xl text-sm transition-colors ${
                    props.showHidden
                      ? "bg-[var(--color-accent)] text-white"
                      : "bg-[var(--color-bg-secondary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]"
                  }`}
                >
                  <Filter class="w-4 h-4" />
                  <span>Filters</span>
                  <ChevronDown
                    class={`w-4 h-4 transition-transform ${showFiltersDropdown() ? "rotate-180" : ""}`}
                  />
                </button>

                <Show when={showFiltersDropdown()}>
                  <div class="absolute top-full right-0 mt-2 bg-[var(--color-bg-primary)] rounded-xl shadow-lg overflow-hidden min-w-[200px] z-50">
                    <label class="flex items-center gap-3 px-4 py-3 hover:bg-[var(--color-bg-secondary)] cursor-pointer transition-colors">
                      <input
                        type="checkbox"
                        checked={props.showHidden}
                        onChange={(e) =>
                          props.onShowHiddenChange(e.currentTarget.checked)
                        }
                        class="w-4 h-4 cursor-pointer"
                      />
                      <span class="text-sm text-[var(--color-text-primary)]">
                        Show Hidden Games
                      </span>
                    </label>
                  </div>
                </Show>
              </div>

              <div class="h-4 w-px bg-[var(--color-border)]"></div>

              <div class="flex gap-1 bg-[var(--color-bg-secondary)] rounded-lg p-1">
                <button
                  onClick={() => setViewMode("grid")}
                  class={`p-1.5 rounded transition-colors ${
                    viewMode() === "grid"
                      ? "text-[var(--color-text-primary)] bg-[var(--color-bg-primary)] shadow-sm"
                      : "text-[var(--color-icon)] hover:text-[var(--color-text-primary)]"
                  }`}
                >
                  <LayoutGrid class="w-4 h-4" />
                </button>
                <button
                  onClick={() => setViewMode("list")}
                  class={`p-1.5 rounded transition-colors ${
                    viewMode() === "list"
                      ? "text-[var(--color-text-primary)] bg-[var(--color-bg-primary)] shadow-sm"
                      : "text-[var(--color-icon)] hover:text-[var(--color-text-primary)]"
                  }`}
                >
                  <List class="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>
        </div>

        <div class="flex-1 overflow-y-auto px-8 pt-4 pb-8 custom-scrollbar">
          <Show
            when={props.filteredGames.length > 0}
            fallback={
              <div class="flex flex-col items-center justify-center h-64 text-[var(--color-text-tertiary)] border-2 border-dashed border-[var(--color-border)] rounded-3xl">
                <Gamepad2 class="w-12 h-12 mb-3 opacity-20" />
                <p>No games found in this category.</p>
                <button
                  onClick={props.onAddGame}
                  class="mt-4 text-[var(--color-accent)] hover:underline font-medium"
                >
                  Add your first game
                </button>
              </div>
            }
          >
            <Show
              when={viewMode() === "grid"}
              fallback={
                <div class="flex flex-col gap-2">
                  <For each={props.filteredGames}>
                    {(game) => (
                      <div
                        class={`group flex items-center gap-4 p-3 bg-[var(--color-bg-secondary)] rounded-xl cursor-pointer transition-all hover:bg-[var(--color-border)] ${
                          game.is_hidden && props.showHidden ? "opacity-50" : ""
                        } ${props.runningGame === game.id ? "ring-2 ring-[var(--color-success)]" : ""}`}
                        onClick={() =>
                          game.vndb_id
                            ? props.onOpenDetail(game)
                            : props.onEditSettings(game)
                        }
                        onContextMenu={(e) => {
                          e.preventDefault();
                          handleContextMenuOpen(game.id);
                        }}
                      >
                        <div class="w-16 h-22 rounded-lg overflow-hidden flex-shrink-0 bg-[var(--color-border)]">
                          <Show
                            when={game.cover_url}
                            fallback={
                              <div class="w-full h-full flex items-center justify-center">
                                <Gamepad2 class="w-6 h-6 text-[var(--color-icon)]" />
                              </div>
                            }
                          >
                            <img
                              src={game.cover_url!}
                              alt={game.title}
                              class="w-full h-full object-cover"
                              loading="lazy"
                            />
                          </Show>
                        </div>

                        <div class="flex-1 min-w-0">
                          <h3 class="text-[var(--color-text-primary)] font-medium truncate">
                            {game.title}
                          </h3>
                          <p class="text-sm text-[var(--color-text-tertiary)]">
                            {props.formatPlayTime(game.play_time)}
                          </p>
                        </div>

                        <div class="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                          <Show when={props.runningGame === game.id}>
                            <span class="px-2 py-1 bg-[var(--color-success)] text-white text-xs font-bold rounded animate-pulse">
                              RUNNING
                            </span>
                          </Show>
                          <button
                            onClick={(e) => {
                              e.stopPropagation();
                              props.onLaunchGame(game.id);
                            }}
                            class="px-4 py-2 bg-[var(--color-accent)] text-white rounded-lg font-medium hover:bg-[var(--color-accent-hover)] transition-colors"
                          >
                            Play
                          </button>
                        </div>
                      </div>
                    )}
                  </For>
                </div>
              }
            >
              <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6 gap-6">
                <For each={props.filteredGames}>
                  {(game) => (
                    <GameCard
                      game={game}
                      isRunning={props.runningGame === game.id}
                      showHidden={props.showHidden}
                      formatPlayTime={props.formatPlayTime}
                      onPlay={props.onLaunchGame}
                      onRemove={props.onRemoveGame}
                      onEditSettings={props.onEditSettings}
                      onClick={(g) =>
                        g.vndb_id
                          ? props.onOpenDetail(g)
                          : props.onEditSettings(g)
                      }
                      onHide={props.onHideGame}
                      activeContextMenu={activeContextMenu()}
                      onContextMenuOpen={handleContextMenuOpen}
                    />
                  )}
                </For>
              </div>
            </Show>
          </Show>
        </div>
      </main>
    </div>
  );
}
