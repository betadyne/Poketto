import {
  createSignal,
  createResource,
  createMemo,
  For,
  Show,
  onMount,
  onCleanup,
  createEffect,
} from "solid-js";
import {
  IconX,
  IconSearch,
  IconFolder,
  IconPhoto,
  IconGlassFull,
  IconDeviceDesktop,
  IconCheck,
  IconLoader2,
} from "@tabler/icons-solidjs";
import { open } from "@tauri-apps/plugin-dialog";
import type {
  GameMetadata,
  VndbSearchResult,
  VndbImage,
  WineSettings,
  WineType,
  GameType,
} from "../bindings";
import * as api from "../api";
import { isVndbId, shouldBlur } from "../utils";

export interface GameSettingsOverlayProps {
  mode: "add" | "edit";
  existingGame?: GameMetadata;
  blurNsfw: boolean;
  onSave: (data: GameSettingsData) => Promise<void>;
  onClose: () => void;
}

export interface GameSettingsData {
  id?: string;
  title: string;
  vndbId: string | null;
  steamAppId: string | null;
  coverUrl: string | null;
  executablePath: string;
  gameType: GameType;
  wineSettings: WineSettings | null;
}

export function GameSettingsOverlay(props: GameSettingsOverlayProps) {
  const [title, setTitle] = createSignal(props.existingGame?.title ?? "");
  const [vndbId, setVndbId] = createSignal<string | null>(
    props.existingGame?.vndb_id ?? null,
  );
  const [steamAppId, setSteamAppId] = createSignal(
    props.existingGame?.steam_app_id ?? "",
  );
  const [vndbSearchQuery, setVndbSearchQuery] = createSignal("");
  const [vndbResults, setVndbResults] = createSignal<VndbSearchResult[]>([]);
  const [isSearching, setIsSearching] = createSignal(false);
  const [selectedVndb, setSelectedVndb] = createSignal<VndbSearchResult | null>(
    null,
  );

  const [coverUrl, setCoverUrl] = createSignal(
    props.existingGame?.cover_url ?? "",
  );
  const [coverUrlInput, setCoverUrlInput] = createSignal(
    props.existingGame?.cover_url ?? "",
  );

  const [platformVersion, setPlatformVersion] = createSignal<
    "windows" | "linux"
  >(
    props.existingGame?.game_type === "LinuxNative" ||
      (props.existingGame?.path?.endsWith(".AppImage") ?? false) ||
      (props.existingGame?.path?.endsWith(".sh") ?? false)
      ? "linux"
      : "windows",
  );

  const [useGlobalPrefix, setUseGlobalPrefix] = createSignal(
    props.existingGame?.wine_settings?.use_global_prefix ?? false,
  );
  const [winePrefix, setWinePrefix] = createSignal(
    props.existingGame?.wine_settings?.wine_prefix ?? "",
  );
  const [wineVersion, setWineVersion] = createSignal(
    props.existingGame?.wine_settings?.wine_version ?? "",
  );
  const [wineType, setWineType] = createSignal<WineType | null>(
    props.existingGame?.wine_settings?.wine_type ?? null,
  );
  const [useSteamRuntime, setUseSteamRuntime] = createSignal(
    props.existingGame?.wine_settings?.use_steam_runtime ?? false,
  );

  const [executablePath, setExecutablePath] = createSignal(
    props.existingGame?.path ?? "",
  );

  const [hostPlatform, setHostPlatform] = createSignal<string>("windows");
  const [isSaving, setIsSaving] = createSignal(false);

  const [wineVersions] = createResource(async () => {
    const platform = await api.getPlatform();
    if (platform === "linux") {
      return await api.getAvailableWineVersions();
    }
    return [];
  });

  const [steamRuntimeAvailable] = createResource(async () => {
    const platform = await api.getPlatform();
    if (platform === "linux") {
      return await api.isSteamRuntimeAvailable();
    }
    return false;
  });

  onMount(async () => {
    const platform = await api.getPlatform();
    setHostPlatform(platform);

    if (props.mode === "add" && platform === "linux" && !winePrefix()) {
      const gameId = crypto.randomUUID();
      const defaultPrefix = await api.getDefaultPrefixPath(gameId);
      setWinePrefix(defaultPrefix);
    }

    if (!wineVersion() && wineVersions()?.length) {
      const firstVersion = wineVersions()![0];
      setWineVersion(firstVersion.binary_path);
      setWineType(firstVersion.wine_type);
    }
  });

  createEffect(() => {
    const versions = wineVersions();
    if (versions?.length && !wineVersion()) {
      const firstVersion = versions[0];
      setWineVersion(firstVersion.binary_path);
      setWineType(firstVersion.wine_type);
    }
  });

  let searchDebounceTimer: ReturnType<typeof setTimeout> | undefined;

  const searchVndb = async () => {
    const query = vndbSearchQuery().trim();
    if (!query) {
      setVndbResults([]);
      return;
    }

    setIsSearching(true);
    try {
      if (isVndbId(query)) {
        const detail = await api.fetchVndbDetail(query.toLowerCase());
        if (detail.status === "ok") {
          setVndbResults([
            {
              id: detail.data.id,
              title: detail.data.title,
              image: detail.data.image,
              released: detail.data.released,
              rating: detail.data.rating,
            },
          ]);
        }
      } else {
        const result = await api.searchVndb(query);
        if (result.status === "ok") {
          setVndbResults(result.data);
        }
      }
    } finally {
      setIsSearching(false);
    }
  };

  const handleSearchInput = (value: string) => {
    setVndbSearchQuery(value);
    clearTimeout(searchDebounceTimer);
    if (value.trim()) {
      searchDebounceTimer = setTimeout(searchVndb, 300);
    } else {
      setVndbResults([]);
    }
  };

  onCleanup(() => clearTimeout(searchDebounceTimer));

  const selectVndbResult = (result: VndbSearchResult) => {
    setSelectedVndb(result);
    setVndbId(result.id);
    setTitle(result.title);
    if (result.image?.url) {
      setCoverUrl(result.image.url);
      setCoverUrlInput(result.image.url);
    }
  };

  const handleCoverUrlChange = (value: string) => {
    setCoverUrlInput(value);
    setCoverUrl(value);
  };

  const browseCoverImage = async () => {
    const selected = await open({
      multiple: false,
      filters: [
        { name: "Images", extensions: ["jpg", "jpeg", "png", "webp", "gif"] },
      ],
      title: "Select Cover Image",
    });
    if (selected) {
      const fileUrl = `file://${selected}`;
      setCoverUrl(fileUrl);
      setCoverUrlInput(fileUrl);
    }
  };

  const browseWinePrefix = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Wine Prefix Directory",
    });
    if (selected) {
      setWinePrefix(selected as string);
    }
  };

  const browseExecutable = async () => {
    const isLinuxNative = platformVersion() === "linux";

    const filters = isLinuxNative
      ? [
          { name: "AppImage", extensions: ["AppImage"] },
          { name: "Scripts", extensions: ["sh"] },
          { name: "All Files", extensions: ["*"] },
        ]
      : [
          { name: "Windows Executable", extensions: ["exe"] },
          { name: "All Files", extensions: ["*"] },
        ];

    const selected = await open({
      multiple: false,
      filters,
      title: "Select Game Executable",
    });

    if (selected) {
      setExecutablePath(selected as string);

      if (props.mode === "add" && !title()) {
        const pathParts = (selected as string).split(/[/\\]/);
        const folderName = pathParts[pathParts.length - 2] || "Unknown Game";
        setTitle(folderName);
        setVndbSearchQuery(folderName);
        searchVndb();
      }
    }
  };

  const handleWineVersionChange = (binaryPath: string) => {
    setWineVersion(binaryPath);
    const version = wineVersions()?.find((v) => v.binary_path === binaryPath);
    if (version) {
      setWineType(version.wine_type);
    }
  };

  const groupedVersions = createMemo(() => {
    const versions = wineVersions();
    if (!versions) return {};
    return api.groupWineVersionsByType(versions);
  });

  const blurCheck = (img: VndbImage | null): boolean =>
    shouldBlur(img, props.blurNsfw);

  const canFinish = createMemo(() => {
    if (!title().trim()) return false;
    if (!vndbId()) return false;
    if (!executablePath()) return false;
    if (
      platformVersion() === "windows" &&
      hostPlatform() === "linux" &&
      !wineVersion()
    ) {
      return false;
    }
    return true;
  });

  const handleFinish = async () => {
    if (!canFinish() || isSaving()) return;

    setIsSaving(true);
    try {
      const gameType: GameType =
        platformVersion() === "linux" ? "LinuxNative" : "WindowsExe";

      let wineSettings: WineSettings | null = null;
      if (gameType === "WindowsExe" && hostPlatform() === "linux") {
        wineSettings = {
          use_global_prefix: useGlobalPrefix(),
          wine_prefix: useGlobalPrefix() ? null : winePrefix(),
          wine_version: wineVersion() || null,
          wine_type: wineType(),
          use_steam_runtime: useSteamRuntime(),
          env_vars: props.existingGame?.wine_settings?.env_vars || {},
        };
      }

      const data: GameSettingsData = {
        id: props.existingGame?.id,
        title: title(),
        vndbId: vndbId(),
        steamAppId: steamAppId().trim() || null,
        coverUrl: coverUrl() || null,
        executablePath: executablePath(),
        gameType,
        wineSettings,
      };

      await props.onSave(data);
    } finally {
      setIsSaving(false);
    }
  };

  const showWineSettings = () =>
    platformVersion() === "windows" && hostPlatform() === "linux";

  return (
    <div
      class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4"
      onClick={props.onClose}
    >
      <div
        class="bg-[var(--color-bg-primary)] rounded-lg w-full max-w-2xl max-h-[85vh] overflow-hidden flex flex-col shadow-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
          <h2 class="text-lg font-bold text-[var(--color-text-primary)] flex items-center gap-2">
            {props.mode === "add" ? "Add Game" : "Edit Game Settings"}
          </h2>
          <button
            onClick={props.onClose}
            class="text-[var(--color-icon)] hover:text-[var(--color-text-primary)] transition-colors"
          >
            <IconX class="w-5 h-5" strokeWidth={1.5} />
          </button>
        </div>

        <div class="flex-1 overflow-y-auto p-4 space-y-5">
          <div class="flex gap-4">
            <div class="flex-shrink-0 w-32">
              <div class="w-32 h-44 bg-[var(--color-bg-secondary)] rounded-lg overflow-hidden border-2 border-[var(--color-border)]">
                <Show
                  when={coverUrl()}
                  fallback={
                    <div class="w-full h-full flex items-center justify-center text-[var(--color-icon)]">
                      <IconPhoto class="w-8 h-8" strokeWidth={1.5} />
                    </div>
                  }
                >
                  <img
                    src={coverUrl()}
                    alt="Cover"
                    class={`w-full h-full object-cover ${
                      selectedVndb()?.image && blurCheck(selectedVndb()!.image)
                        ? "blur-lg"
                        : ""
                    }`}
                    onError={() => setCoverUrl("")}
                  />
                </Show>
              </div>
            </div>

            <div class="flex-1 space-y-4">
              <div class="space-y-2">
                <label class="text-sm font-medium text-[var(--color-text-primary)]">
                  Game Title
                </label>
                <input
                  type="text"
                  value={title()}
                  onInput={(e) => setTitle(e.currentTarget.value)}
                  placeholder="Enter game title..."
                  class="w-full px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                />
              </div>

              <div class="space-y-2">
                <label class="text-sm font-medium text-[var(--color-text-primary)]">
                  Link to VNDB
                </label>
                <div class="flex gap-2">
                  <input
                    type="text"
                    value={vndbSearchQuery()}
                    onInput={(e) => handleSearchInput(e.currentTarget.value)}
                    onKeyPress={(e) => e.key === "Enter" && searchVndb()}
                    placeholder="Search game or enter VNDB ID..."
                    class="flex-1 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                  />
                  <button
                    onClick={searchVndb}
                    disabled={isSearching()}
                    class="px-3 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] disabled:opacity-50 rounded-lg text-white transition-colors"
                  >
                    <Show
                      when={isSearching()}
                      fallback={<IconSearch class="w-4 h-4" strokeWidth={1.5} />}
                    >
                      <IconLoader2 class="w-4 h-4 animate-spin" strokeWidth={1.5} />
                    </Show>
                  </button>
                </div>

                <Show when={vndbId()}>
                  <div class="flex items-center gap-2 px-3 py-2 bg-[var(--color-success-light)] border border-[var(--color-success)] rounded-lg">
                    <IconCheck class="w-4 h-4 text-[var(--color-success)]" strokeWidth={1.5} />
                    <span class="text-sm text-[var(--color-success)]">
                      Linked to {vndbId()}
                    </span>
                  </div>
                </Show>

                <Show when={vndbResults().length > 0}>
                  <div class="max-h-40 overflow-y-auto space-y-1 bg-[var(--color-bg-secondary)] rounded-lg p-2">
                    <For each={vndbResults()}>
                      {(result) => (
                        <button
                          onClick={() => selectVndbResult(result)}
                          class={`w-full flex items-center gap-2 p-2 rounded-lg text-left transition-colors ${
                            vndbId() === result.id
                              ? "bg-[var(--color-accent)]/10 border border-[var(--color-accent)]"
                              : "bg-[var(--color-bg-primary)] hover:bg-[var(--color-bg-tertiary)]"
                          }`}
                        >
                          <div class="w-8 h-11 bg-[var(--color-bg-secondary)] rounded overflow-hidden flex-shrink-0">
                            <Show when={result.image?.url}>
                              <img
                                src={result.image!.url}
                                alt={result.title}
                                class={`w-full h-full object-cover ${blurCheck(result.image ?? null) ? "blur-lg" : ""}`}
                              />
                            </Show>
                          </div>
                          <div class="flex-1 min-w-0">
                            <h4 class="text-[var(--color-text-primary)] text-sm font-medium truncate">
                              {result.title}
                            </h4>
                            <p class="text-xs text-[var(--color-text-secondary)]">
                              {result.id}
                              {result.released && ` - ${result.released}`}
                              {result.rating &&
                                ` - ${(result.rating / 10).toFixed(1)}`}
                            </p>
                          </div>
                          <Show when={vndbId() === result.id}>
                            <IconCheck class="w-4 h-4 text-[var(--color-accent)]" strokeWidth={1.5} />
                          </Show>
                        </button>
                      )}
                    </For>
                  </div>
                </Show>
              </div>
            </div>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium text-[var(--color-text-primary)] flex items-center gap-2">
              <IconPhoto class="w-4 h-4" strokeWidth={1.5} />
              Cover Art
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                value={coverUrlInput()}
                onInput={(e) => handleCoverUrlChange(e.currentTarget.value)}
                placeholder="Paste image URL or select local file..."
                class="flex-1 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
              />
              <button
                onClick={browseCoverImage}
                class="px-3 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] transition-colors"
                title="Browse local image"
              >
                <IconFolder class="w-4 h-4" strokeWidth={1.5} />
              </button>
            </div>
            <p class="text-xs text-[var(--color-text-tertiary)]">
              Leave empty to use VNDB cover image
            </p>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium text-[var(--color-text-primary)] flex items-center gap-2">
              <IconDeviceDesktop class="w-4 h-4" strokeWidth={1.5} />
              Platform Version
            </label>
            <select
              value={platformVersion()}
              onChange={(e) =>
                setPlatformVersion(e.currentTarget.value as "windows" | "linux")
              }
              class="w-full px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
            >
              <option value="windows">Windows</option>
              <option value="linux">Linux</option>
            </select>
            <p class="text-xs text-[var(--color-text-tertiary)]">
              {platformVersion() === "windows"
                ? "Windows executable (.exe) - requires Wine/Proton on Linux"
                : "Linux native binary (.AppImage, .sh, etc.)"}
            </p>
          </div>

          <Show when={showWineSettings()}>
            <div class="space-y-4 p-4 bg-[var(--color-bg-secondary)] rounded-lg border border-[var(--color-border)]">
              <h3 class="text-sm font-medium text-[var(--color-accent)] flex items-center gap-2">
                <IconGlassFull class="w-4 h-4" strokeWidth={1.5} />
                Wine/Proton Settings
              </h3>

              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  checked={useGlobalPrefix()}
                  onChange={(e) => setUseGlobalPrefix(e.currentTarget.checked)}
                  class="w-4 h-4 rounded bg-[var(--color-bg-primary)] border-[var(--color-border)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <div>
                  <span class="text-sm text-[var(--color-text-primary)]">Use global prefix</span>
                  <p class="text-xs text-[var(--color-text-tertiary)]">
                    Share wine prefix across multiple games
                  </p>
                </div>
              </label>

              <Show when={!useGlobalPrefix()}>
                <div class="space-y-2">
                  <label class="text-sm text-[var(--color-text-primary)]">Wine Prefix</label>
                  <div class="flex gap-2">
                    <input
                      type="text"
                      value={winePrefix()}
                      onInput={(e) => setWinePrefix(e.currentTarget.value)}
                      placeholder="Path to wine prefix..."
                      class="flex-1 px-3 py-2 bg-[var(--color-bg-primary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                    />
                    <button
                      onClick={browseWinePrefix}
                      class="px-3 py-2 bg-[var(--color-bg-primary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] transition-colors"
                      title="Browse"
                    >
                      <IconFolder class="w-4 h-4" strokeWidth={1.5} />
                    </button>
                  </div>
                </div>
              </Show>

              <div class="space-y-2">
                <label class="text-sm text-[var(--color-text-primary)]">Wine/Proton Version</label>
                <Show
                  when={!wineVersions.loading && wineVersions()?.length}
                  fallback={
                    <div class="px-3 py-2 bg-[var(--color-bg-primary)] rounded-lg text-[var(--color-text-secondary)] text-sm">
                      {wineVersions.loading
                        ? "Detecting Wine versions..."
                        : "No Wine/Proton found. Please install Wine."}
                    </div>
                  }
                >
                  <select
                    value={wineVersion()}
                    onChange={(e) =>
                      handleWineVersionChange(e.currentTarget.value)
                    }
                    class="w-full px-3 py-2 bg-[var(--color-bg-primary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                  >
                    <For each={Object.entries(groupedVersions())}>
                      {([type, versions]) => (
                        <Show when={versions.length > 0}>
                          <optgroup label={api.getWineTypeDisplayName(type)}>
                            <For each={versions}>
                              {(v) => (
                                <option value={v.binary_path}>{v.name}</option>
                              )}
                            </For>
                          </optgroup>
                        </Show>
                      )}
                    </For>
                  </select>
                </Show>
              </div>

              <div class="flex items-center justify-between">
                <div>
                  <span class="text-sm text-[var(--color-text-primary)]">Use Steam Runtime</span>
                  <p class="text-xs text-[var(--color-text-tertiary)]">
                    Better compatibility for some games
                  </p>
                </div>
                <button
                  onClick={() => setUseSteamRuntime(!useSteamRuntime())}
                  disabled={!steamRuntimeAvailable()}
                  class={`w-10 h-5 rounded-full transition-colors ${
                    useSteamRuntime() ? "bg-[var(--color-accent)]" : "bg-[var(--color-border)]"
                  } ${!steamRuntimeAvailable() ? "opacity-50 cursor-not-allowed" : ""}`}
                >
                  <div
                    class={`w-4 h-4 bg-white rounded-full transition-transform ${
                      useSteamRuntime() ? "translate-x-5" : "translate-x-0.5"
                    }`}
                  />
                </button>
              </div>
              <Show
                when={
                  !steamRuntimeAvailable.loading && !steamRuntimeAvailable()
                }
              >
                <p class="text-xs text-amber-600">
                  steam-run not found. Install steam-native-runtime.
                </p>
              </Show>
            </div>
          </Show>

          <div class="space-y-2">
            <label class="text-sm font-medium text-[var(--color-text-primary)]">
              Select Executable
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                value={executablePath()}
                onInput={(e) => setExecutablePath(e.currentTarget.value)}
                placeholder={
                  platformVersion() === "linux"
                    ? "Path to AppImage, script, or binary..."
                    : "Path to .exe file..."
                }
                class="flex-1 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
              />
              <button
                onClick={browseExecutable}
                class="px-3 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] transition-colors"
                title="Browse"
              >
              <IconFolder class="w-4 h-4" strokeWidth={1.5} />
              </button>
            </div>
            <p class="text-xs text-[var(--color-text-tertiary)]">
              {platformVersion() === "linux"
                ? "Supports: .AppImage, .sh, binary files"
                : "Supports: .exe files"}
            </p>
          </div>
          <div class="space-y-2">
            <label class="text-sm font-medium text-[var(--color-text-primary)]">
              Steam App ID
            </label>
            <div class="flex gap-2">
              <input
                type="text"
                inputMode="numeric"
                value={steamAppId()}
                onInput={(e) => setSteamAppId(e.currentTarget.value)}
                placeholder="412830"
                class="flex-1 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-lg text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
              />
            </div>
            <p class="text-xs text-[var(--color-text-tertiary)]">
              Optional. When set, the game launches via the Steam client instead of the executable.
            </p>
          </div>
        </div>

        <div class="flex gap-3 p-4 border-t border-[var(--color-border)] bg-[var(--color-bg-primary)]">
          <button
            onClick={props.onClose}
            class="px-4 py-2 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-lg text-[var(--color-text-primary)] transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleFinish}
            disabled={!canFinish() || isSaving()}
            class="flex-1 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] disabled:opacity-50 disabled:cursor-not-allowed rounded-lg text-white font-medium transition-colors flex items-center justify-center gap-2"
          >
            <Show when={isSaving()}>
              <IconLoader2 class="w-4 h-4 animate-spin" strokeWidth={1.5} />
            </Show>
            {isSaving() ? "Saving..." : "Finish"}
          </button>
        </div>
      </div>
    </div>
  );
}
