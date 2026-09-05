import { createSignal, createMemo, Show, onMount, ErrorBoundary } from "solid-js";
import { Router, Route, useNavigate, useParams } from "@solidjs/router";

import type { VndbImage } from "./bindings";
import { useLibraryFilters } from "./hooks/useLibraryFilters";
import { useUpdater } from "./hooks/useUpdater";
import { formatPlayTime, formatLastPlayed, shouldBlur } from "./utils";

import {
  GameProvider,
  SettingsProvider,
  VndbProvider,
  useGame,
  useSettings,
  useVndb,
} from "./context";
import { Library } from "./views/Library";
import { Detail } from "./views/Detail";
import { Settings } from "./views/Settings";
import { Logs } from "./views/Logs";
import { TitleBar } from "./components/TitleBar";
import { UpdateDialog } from "./components/UpdateDialog";
import {
  GameSettingsOverlay,
  type GameSettingsData,
} from "./components/GameSettingsOverlay";

function LibraryPage() {
  const game = useGame();
  const settings = useSettings();
  const navigate = useNavigate();

  const [gameSettingsOverlay, setGameSettingsOverlay] = createSignal<{
    mode: "add" | "edit";
    existingGame?: typeof game.games extends () => (infer T)[] ? T : never;
  } | null>(null);

  const libraryFilters = useLibraryFilters(() => game.games());
  const updater = useUpdater();

  const addGame = () => {
    setGameSettingsOverlay({ mode: "add" });
  };

  const editGame = (gameData: Parameters<typeof game.updateGame>[0]) => {
    setGameSettingsOverlay({ mode: "edit", existingGame: gameData });
  };

  const handleGameSettingsSave = async (data: GameSettingsData) => {
    const overlay = gameSettingsOverlay();
    if (!overlay) return;

    if (overlay.mode === "add") {
      const newGame = await game.addGame(data.executablePath);
      if (newGame) {
        await game.updateGame({
          ...newGame,
          title: data.title,
          vndb_id: data.vndbId,
          steam_app_id: data.steamAppId,
          discord_status: data.discordStatus,
          cover_url: data.coverUrl,
          game_type: data.gameType,
          wine_settings: data.wineSettings,
        });
      }
    } else if (overlay.existingGame) {
      await game.updateGame({
        ...overlay.existingGame,
        title: data.title,
        vndb_id: data.vndbId,
        steam_app_id: data.steamAppId,
        discord_status: data.discordStatus,
        cover_url: data.coverUrl,
        path: data.executablePath,
        game_type: data.gameType,
        wine_settings: data.wineSettings,
      });
    }

    setGameSettingsOverlay(null);
  };

  const openDetail = (gameData: { id: string; vndb_id?: string | null }) => {
    if (!gameData.vndb_id) return;
    navigate(`/game/${gameData.id}`);
  };

  return (
    <>
      <Library
        games={game.games()}
        filteredGames={libraryFilters.filteredGames()}
        runningGame={game.runningGame()}
        loading={game.loading()}
        authUser={settings.authUser()}
        searchQuery={libraryFilters.searchQuery()}
        onSearchChange={libraryFilters.setSearchQuery}
        sortBy={libraryFilters.sortBy()}
        onSortByChange={libraryFilters.setSortBy}
        sortOrder={libraryFilters.sortOrder()}
        onSortOrderChange={libraryFilters.setSortOrder}
        showHidden={libraryFilters.showHidden()}
        onShowHiddenChange={libraryFilters.setShowHidden}
        formatPlayTime={formatPlayTime}
        onAddGame={addGame}
        onStopTracking={game.stopTracking}
        onLaunchGame={game.launchGame}
        onRemoveGame={game.removeGame}
        onEditSettings={editGame}
        onOpenDetail={openDetail}
        onHideGame={game.hideGame}
      />

      <Show when={gameSettingsOverlay()}>
        <GameSettingsOverlay
          mode={gameSettingsOverlay()!.mode}
          existingGame={gameSettingsOverlay()!.existingGame}
          blurNsfw={settings.settings().blur_nsfw}
          onSave={handleGameSettingsSave}
          onClose={() => setGameSettingsOverlay(null)}
        />
      </Show>

      <UpdateDialog
        status={updater.status()}
        updateInfo={updater.updateInfo()}
        downloadProgress={updater.downloadProgress()}
        error={updater.error()}
        onDownload={updater.downloadAndInstall}
        onRestart={updater.restartApp}
        onDismiss={updater.dismissUpdate}
      />
    </>
  );
}

function DetailPage() {
  const params = useParams<{ id: string }>();
  const navigate = useNavigate();
  const game = useGame();
  const settings = useSettings();
  const vndb = useVndb();

  const [showSpoilers, setShowSpoilers] = createSignal(false);
  const [isRefreshing, setIsRefreshing] = createSignal(false);
  const [currentTab, setCurrentTab] = createSignal<"detail" | "detail-chars">(
    "detail",
  );

  const updater = useUpdater();

  const currentGame = createMemo(() =>
    game.games().find((g) => g.id === params.id) || null,
  );

  const loadDetail = async (forceRefresh = false) => {
    const g = currentGame();
    if (!g?.vndb_id) return;
    await Promise.all([
      vndb.fetchDetail(g.vndb_id, forceRefresh),
      vndb.fetchCharacters(g.vndb_id, forceRefresh),
    ]);
    if (settings.settings().vndb_token) {
      await vndb.fetchUserVn(g.vndb_id);
    }
  };

  onMount(() => {
    loadDetail();
  });

  const refreshDetail = async () => {
    setIsRefreshing(true);
    try {
      await loadDetail(true);
    } finally {
      setIsRefreshing(false);
    }
  };

  const goBack = () => {
    vndb.clearDetail();
    navigate("/");
  };

  const setStatus = async (labelId: number) => {
    const g = currentGame();
    if (!g?.vndb_id) return;
    await vndb.setStatus(g.vndb_id, labelId);
  };

  const setVote = async (vote: number) => {
    const g = currentGame();
    if (!g?.vndb_id) return;
    await vndb.setVote(g.vndb_id, vote);
  };

  const blurCheck = (img: VndbImage | null): boolean =>
    shouldBlur(img, settings.settings().blur_nsfw);

  return (
    <>
      <Show
        when={vndb.vnDetail() && currentGame()}
        fallback={
          <div class="flex-1 flex items-center justify-center text-slate-400">
            Loading...
          </div>
        }
      >
        <ErrorBoundary
          fallback={(err) => (
            <div class="flex-1 flex items-center justify-center p-8">
              <div class="text-center space-y-3">
                <p class="text-[var(--color-danger)] font-medium">Failed to load game details</p>
                <p class="text-sm text-[var(--color-text-tertiary)]">{err.toString()}</p>
                <button onClick={goBack} class="px-4 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)]">
                  Go Back
                </button>
              </div>
            </div>
          )}
        >
          <Detail
          page={currentTab()}
          setPage={setCurrentTab}
          game={currentGame()!}
          vnDetail={vndb.vnDetail()!}
          characters={vndb.characters()}
          userVn={vndb.userVn()}
          runningGame={game.runningGame()}
          settings={settings.settings()}
          showSpoilers={showSpoilers()}
          setShowSpoilers={setShowSpoilers}
          isRefreshing={isRefreshing()}
          onBack={goBack}
          onRefresh={refreshDetail}
          onLaunchGame={game.launchGame}
          onSetStatus={setStatus}
          onSetVote={setVote}
          formatPlayTime={formatPlayTime}
          formatLastPlayed={formatLastPlayed}
          shouldBlur={blurCheck}
        />
        </ErrorBoundary>
      </Show>

      <UpdateDialog
        status={updater.status()}
        updateInfo={updater.updateInfo()}
        downloadProgress={updater.downloadProgress()}
        error={updater.error()}
        onDownload={updater.downloadAndInstall}
        onRestart={updater.restartApp}
        onDismiss={updater.dismissUpdate}
      />
    </>
  );
}

function AppLayout(props: { children?: any }) {
  return (
    <ErrorBoundary
      fallback={(err) => (
        <div class="h-screen flex items-center justify-center bg-[var(--color-bg-primary)] text-[var(--color-text-primary)] p-8">
          <div class="max-w-md text-center space-y-4">
            <h1 class="text-2xl font-bold text-[var(--color-danger)]">Something went wrong</h1>
            <p class="text-[var(--color-text-secondary)]">{err.toString()}</p>
            <button
              onClick={() => window.location.reload()}
              class="px-4 py-2 bg-[var(--color-accent)] text-white rounded-lg"
            >
              Reload App
            </button>
          </div>
        </div>
      )}
    >
      <GameProvider>
        <SettingsProvider>
          <VndbProvider>
            <div class="h-screen flex flex-col bg-[var(--color-bg-primary)]">
              <TitleBar />
              <div class="flex-1 overflow-hidden relative">{props.children}</div>
            </div>
          </VndbProvider>
        </SettingsProvider>
      </GameProvider>
    </ErrorBoundary>
  );
}

function App() {
  return (
    <Router root={AppLayout}>
      <Route path="/" component={LibraryPage} />
      <Route path="/settings" component={Settings} />
      <Route path="/logs" component={Logs} />
      <Route path="/game/:id" component={DetailPage} />
    </Router>
  );
}

export default App;
