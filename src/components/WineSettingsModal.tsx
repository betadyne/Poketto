import { createSignal, createResource, For, Show, onMount } from "solid-js";
import { X, Wine, Folder, Terminal } from "lucide-solid";
import { open } from "@tauri-apps/plugin-dialog";
import type { WineSettings, GameType, WineType } from "../bindings";
import * as api from "../api";

interface WineSettingsModalProps {
  gameId: string;
  gameTitle: string;
  initialSettings?: WineSettings | null;
  initialGameType?: GameType | null;
  onSave: (gameType: GameType, wineSettings: WineSettings | null) => void;
  onClose: () => void;
}

export function WineSettingsModal(props: WineSettingsModalProps) {
  const [gameType, setGameType] = createSignal<GameType>(
    props.initialGameType || "WindowsExe",
  );

  const [useGlobalPrefix, setUseGlobalPrefix] = createSignal(
    props.initialSettings?.use_global_prefix ?? false,
  );
  const [winePrefix, setWinePrefix] = createSignal(
    props.initialSettings?.wine_prefix ?? "",
  );
  const [selectedWineBinary, setSelectedWineBinary] = createSignal(
    props.initialSettings?.wine_version ?? "",
  );
  const [wineType, setWineType] = createSignal<WineType | null>(
    props.initialSettings?.wine_type ?? null,
  );
  const [useSteamRuntime, setUseSteamRuntime] = createSignal(
    props.initialSettings?.use_steam_runtime ?? false,
  );

  const [wineVersions] = createResource(async () => {
    return await api.getAvailableWineVersions();
  });

  const [steamRuntimeAvailable] = createResource(async () => {
    return await api.isSteamRuntimeAvailable();
  });

  onMount(async () => {
    if (!winePrefix()) {
      const defaultPrefix = await api.getDefaultPrefixPath(props.gameId);
      setWinePrefix(defaultPrefix);
    }
    if (!selectedWineBinary() && wineVersions()?.length) {
      const firstVersion = wineVersions()![0];
      setSelectedWineBinary(firstVersion.binary_path);
      setWineType(firstVersion.wine_type);
    }
  });

  const handleWineVersionChange = (binaryPath: string) => {
    setSelectedWineBinary(binaryPath);
    const version = wineVersions()?.find((v) => v.binary_path === binaryPath);
    if (version) {
      setWineType(version.wine_type);
    }
  };

  const browsePrefix = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Wine Prefix Directory",
    });
    if (selected) {
      setWinePrefix(selected as string);
    }
  };

  const handleSave = () => {
    if (gameType() === "LinuxNative") {
      props.onSave("LinuxNative", null);
    } else {
      const settings: WineSettings = {
        use_global_prefix: useGlobalPrefix(),
        wine_prefix: useGlobalPrefix() ? null : winePrefix(),
        wine_version: selectedWineBinary() || null,
        wine_type: wineType(),
        use_steam_runtime: useSteamRuntime(),
        env_vars: props.initialSettings?.env_vars || {},
      };
      props.onSave("WindowsExe", settings);
    }
  };

  const groupedVersions = () => {
    const versions = wineVersions();
    if (!versions) return {};
    return api.groupWineVersionsByType(versions);
  };

  return (
    <div
      class="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4"
      onClick={props.onClose}
    >
      <div
        class="bg-slate-800 rounded-lg w-full max-w-lg max-h-[80vh] overflow-hidden"
        onClick={(e) => e.stopPropagation()}
      >
        <div class="flex items-center justify-between p-4 border-b border-slate-700">
          <h2 class="text-lg font-bold text-white flex items-center gap-2">
            <Wine class="w-5 h-5 text-purple-400" />
            Wine Settings
          </h2>
          <button
            onClick={props.onClose}
            class="text-gray-400 hover:text-white"
          >
            <X class="w-5 h-5" />
          </button>
        </div>

        <div class="p-4 space-y-5 overflow-y-auto max-h-[60vh]">
          <div class="bg-slate-700/50 rounded-lg p-3">
            <p class="text-sm text-gray-400">Configuring:</p>
            <p class="text-white font-medium truncate">{props.gameTitle}</p>
          </div>

          <div class="space-y-2">
            <label class="text-sm font-medium text-gray-300">Game Type</label>
            <div class="flex gap-3">
              <button
                onClick={() => setGameType("WindowsExe")}
                class={`flex-1 px-4 py-3 rounded-lg border-2 transition-colors ${
                  gameType() === "WindowsExe"
                    ? "border-purple-500 bg-purple-500/20 text-white"
                    : "border-slate-600 bg-slate-700 text-gray-400 hover:border-slate-500"
                }`}
              >
                <div class="text-sm font-medium">Windows Executable</div>
                <div class="text-xs mt-1 opacity-70">Requires Wine/Proton</div>
              </button>
              <button
                onClick={() => setGameType("LinuxNative")}
                class={`flex-1 px-4 py-3 rounded-lg border-2 transition-colors ${
                  gameType() === "LinuxNative"
                    ? "border-green-500 bg-green-500/20 text-white"
                    : "border-slate-600 bg-slate-700 text-gray-400 hover:border-slate-500"
                }`}
              >
                <div class="text-sm font-medium">Linux Native</div>
                <div class="text-xs mt-1 opacity-70">No Wine needed</div>
              </button>
            </div>
          </div>

          <Show when={gameType() === "WindowsExe"}>
            <div class="space-y-2">
              <label class="text-sm font-medium text-gray-300">
                Wine Version
              </label>
              <Show
                when={!wineVersions.loading && wineVersions()?.length}
                fallback={
                  <div class="px-3 py-2 bg-slate-700 rounded text-gray-400 text-sm">
                    {wineVersions.loading
                      ? "Detecting Wine versions..."
                      : "No Wine/Proton found. Please install Wine."}
                  </div>
                }
              >
                <select
                  value={selectedWineBinary()}
                  onChange={(e) =>
                    handleWineVersionChange(e.currentTarget.value)
                  }
                  class="w-full px-3 py-2 bg-slate-700 rounded text-white text-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
                >
                  <For each={Object.entries(groupedVersions())}>
                    {([type, versions]) => (
                      <Show when={versions.length > 0}>
                        <optgroup label={api.getWineTypeDisplayName(type)}>
                          <For each={versions}>
                            {(v) => (
                              <option value={v.binary_path}>
                                {v.name}
                                {v.version ? ` (${v.version})` : ""}
                              </option>
                            )}
                          </For>
                        </optgroup>
                      </Show>
                    )}
                  </For>
                </select>
              </Show>
            </div>

            <div class="space-y-2">
              <div class="flex items-center justify-between">
                <label class="text-sm font-medium text-gray-300">
                  Wine Prefix
                </label>
                <label class="flex items-center gap-2 text-sm text-gray-400 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={useGlobalPrefix()}
                    onChange={(e) =>
                      setUseGlobalPrefix(e.currentTarget.checked)
                    }
                    class="w-4 h-4 rounded bg-slate-600 border-slate-500 text-purple-500 focus:ring-purple-500"
                  />
                  Use global prefix
                </label>
              </div>
              <Show when={!useGlobalPrefix()}>
                <div class="flex gap-2">
                  <input
                    type="text"
                    value={winePrefix()}
                    onInput={(e) => setWinePrefix(e.currentTarget.value)}
                    placeholder="Wine prefix path..."
                    class="flex-1 px-3 py-2 bg-slate-700 rounded text-white text-sm focus:outline-none focus:ring-2 focus:ring-purple-500"
                  />
                  <button
                    onClick={browsePrefix}
                    class="px-3 py-2 bg-slate-600 hover:bg-slate-500 rounded text-white"
                    title="Browse"
                  >
                    <Folder class="w-4 h-4" />
                  </button>
                </div>
                <p class="text-xs text-gray-500">
                  Each game uses its own prefix for isolation
                </p>
              </Show>
              <Show when={useGlobalPrefix()}>
                <p class="text-xs text-gray-500">
                  Using global Wine prefix from Settings
                </p>
              </Show>
            </div>

            <div class="flex items-center justify-between py-2">
              <div>
                <span class="text-sm text-gray-300">Use Steam Runtime</span>
                <p class="text-xs text-gray-500">
                  Better compatibility for some games
                </p>
              </div>
              <button
                onClick={() => setUseSteamRuntime(!useSteamRuntime())}
                disabled={!steamRuntimeAvailable()}
                class={`w-10 h-5 rounded-full transition-colors ${
                  useSteamRuntime() ? "bg-purple-500" : "bg-slate-600"
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
              when={!steamRuntimeAvailable.loading && !steamRuntimeAvailable()}
            >
              <p class="text-xs text-amber-400 flex items-center gap-1">
                <Terminal class="w-3 h-3" />
                steam-run not found. Install steam-native-runtime from AUR.
              </p>
            </Show>
          </Show>
        </div>

        <div class="flex gap-3 p-4 border-t border-slate-700">
          <button
            onClick={handleSave}
            class="flex-1 px-4 py-2 bg-purple-600 hover:bg-purple-700 rounded-lg text-white font-medium"
          >
            Save Settings
          </button>
          <button
            onClick={props.onClose}
            class="px-4 py-2 bg-slate-700 hover:bg-slate-600 rounded-lg text-gray-300"
          >
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}
