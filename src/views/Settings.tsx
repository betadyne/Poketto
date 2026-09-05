import { Show, For, createSignal, onMount } from "solid-js";
import {
  IconLibrary as LibraryIcon,
  IconSettings as SettingsIcon,
  IconNotes as ScrollText,
  IconLogout as LogOut,
  IconUser,
  IconRefresh,
} from "@tabler/icons-solidjs";
import { useNavigate } from "@solidjs/router";

import { useSettings } from "../context";
import { useUpdater } from "../hooks/useUpdater";
import * as api from "../api";

interface DiscordButtonConfig {
  key: "vndb_game" | "vndb_profile" | "github";
  label: string;
  sublabel?: string;
  requiresToken?: boolean;
}

const DISCORD_BUTTONS: DiscordButtonConfig[] = [
  { key: "vndb_game", label: "View on VNDB", sublabel: "(game page)" },
  { key: "vndb_profile", label: "My VNDB Profile", requiresToken: true },
  { key: "github", label: "GitHub Repository" },
];

export function Settings() {
  const navigate = useNavigate();
  const settings = useSettings();
  const updater = useUpdater();

  const [tokenInput, setTokenInput] = createSignal("");
  const [platform, setPlatform] = createSignal<string>("windows");
  const [wineVersions, setWineVersions] = createSignal<api.WineVersion[]>([]);
  const [steamRuntimeAvailable, setSteamRuntimeAvailable] = createSignal(false);
  const [defaultWineBinary, setDefaultWineBinary] = createSignal<string>("");
  const [useSteamRuntime, setUseSteamRuntime] = createSignal(false);

  onMount(async () => {
    const p = await api.getPlatform();
    setPlatform(p);
    if (p === "linux") {
      const versions = await api.getAvailableWineVersions();
      setWineVersions(versions);
      const steamAvail = await api.isSteamRuntimeAvailable();
      setSteamRuntimeAvailable(steamAvail);
      const defaults = await api.getDefaultWineSettings();
      setDefaultWineBinary(defaults.wine_version || "");
      setUseSteamRuntime(defaults.use_steam_runtime);
    }
  });

  const saveWineDefaults = async () => {
    await api.saveGlobalWineDefaults(
      null,
      defaultWineBinary() || null,
      useSteamRuntime(),
    );
  };

  const getButtonValue = (key: DiscordButtonConfig["key"]): boolean => {
    const s = settings.settings();
    if (key === "vndb_game") return s.discord_btn_vndb_game ?? true;
    if (key === "vndb_profile") return s.discord_btn_vndb_profile ?? false;
    return s.discord_btn_github ?? false;
  };

  const handleDiscordButtonChange = (
    changedKey: DiscordButtonConfig["key"],
    newValue: boolean,
    checkbox: HTMLInputElement,
  ) => {
    const values = {
      vndb_game: changedKey === "vndb_game" ? newValue : getButtonValue("vndb_game"),
      vndb_profile: changedKey === "vndb_profile" ? newValue : getButtonValue("vndb_profile"),
      github: changedKey === "github" ? newValue : getButtonValue("github"),
    };

    const count = Object.values(values).filter(Boolean).length;
    if (count <= 2) {
      settings.updateDiscordButtons(values.vndb_game, values.vndb_profile, values.github);
    } else {
      checkbox.checked = !newValue;
    }
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
            <button
              onClick={() => navigate("/")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <LibraryIcon class="w-5 h-5 text-[var(--color-icon)]" strokeWidth={1.5} />
              </div>
              <span class="font-medium">My Games</span>
            </button>
          </nav>

          <div class="h-px bg-[var(--color-border)] w-full" />

          <nav class="space-y-1">
            <button class="w-full flex items-center gap-3 px-3 py-2.5 rounded-xl transition-all">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-accent)] flex items-center justify-center">
                <SettingsIcon class="w-5 h-5 text-white" strokeWidth={1.5} />
              </div>
              <span class="font-medium text-[var(--color-text-primary)]">
                Settings
              </span>
            </button>
            <button
              onClick={() => navigate("/logs")}
              class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group"
            >
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <ScrollText class="w-5 h-5 text-[var(--color-icon)]" strokeWidth={1.5} />
              </div>
              <span class="font-medium">Logs</span>
            </button>
            <button class="w-full flex items-center gap-3 px-3 py-2.5 text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] rounded-xl transition-all group">
              <div class="w-9 h-9 rounded-lg bg-[var(--color-bg-secondary)] group-hover:bg-[var(--color-border)] flex items-center justify-center transition-colors">
                <LogOut class="w-5 h-5 text-[var(--color-icon)]" strokeWidth={1.5} />
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
        <header class="h-20 px-8 flex items-center border-b border-[var(--color-border)]">
          <h2 class="text-2xl font-bold text-[var(--color-text-primary)]">
            Settings
          </h2>
        </header>

        <div class="flex-1 overflow-y-auto px-8 py-8 custom-scrollbar">
          <div class="max-w-2xl space-y-8">

            <section class="space-y-4">
              <h3 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
                Account
              </h3>
              <div class="space-y-3">
                <div>
                  <label class="text-sm font-medium text-[var(--color-text-primary)] mb-2 block">
                    VNDB API Token
                  </label>
                  <Show
                    when={settings.authUser()}
                    fallback={
                      <div class="space-y-2">
                        <div class="flex gap-2">
                          <input
                            type="password"
                            value={tokenInput()}
                            onInput={(e) => setTokenInput(e.currentTarget.value)}
                            placeholder="Enter token..."
                            class="flex-1 px-3 py-2 bg-[var(--color-bg-secondary)] rounded-xl text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                          />
                          <button
                            onClick={async () => {
                              if (tokenInput()) {
                                await settings.saveToken(tokenInput());
                                setTokenInput("");
                              }
                            }}
                            class="px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent-hover)] rounded-xl text-white text-sm font-medium transition-colors"
                          >
                            Save
                          </button>
                        </div>
                        <p class="text-xs text-[var(--color-text-tertiary)]">
                          Get token from vndb.org/u/tokens
                        </p>
                      </div>
                    }
                  >
                    <div class="flex items-center justify-between bg-[var(--color-bg-secondary)] px-4 py-3 rounded-xl">
                      <span class="text-[var(--color-text-primary)] flex items-center gap-2 text-sm font-medium">
                        <IconUser class="w-4 h-4 text-[var(--color-accent)]" strokeWidth={1.5} /> {settings.authUser()}
                      </span>
                      <button
                        onClick={settings.clearToken}
                        class="text-[var(--color-danger)] hover:underline text-sm font-medium"
                      >
                        Logout
                      </button>
                    </div>
                  </Show>
                </div>
              </div>
            </section>

            <div class="h-px bg-[var(--color-border)]" />

            <section class="space-y-4">
              <h3 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
                Content
              </h3>
              <div class="flex items-center justify-between">
                <div>
                  <span class="text-sm font-medium text-[var(--color-text-primary)]">Blur NSFW Content</span>
                  <p class="text-xs text-[var(--color-text-tertiary)] mt-0.5">
                    Blur images with sexual or violent content
                  </p>
                </div>
                <button
                  onClick={settings.toggleBlurNsfw}
                  class={`w-10 h-5 rounded-full transition-colors ${settings.settings().blur_nsfw ? "bg-[var(--color-success)]" : "bg-[var(--color-border)]"}`}
                >
                  <div
                    class={`w-4 h-4 bg-white rounded-full transition-transform shadow ${settings.settings().blur_nsfw ? "translate-x-5" : "translate-x-0.5"}`}
                  />
                </button>
              </div>
            </section>

            <div class="h-px bg-[var(--color-border)]" />

            <section class="space-y-4">
              <h3 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
                Discord
              </h3>
              <div class="space-y-4">
                <div class="flex items-center justify-between">
                  <div>
                    <span class="text-sm font-medium text-[var(--color-text-primary)]">Rich Presence</span>
                    <p class="text-xs text-[var(--color-text-tertiary)] mt-0.5">
                      Show currently playing game in Discord activity status
                    </p>
                  </div>
                  <button
                    onClick={settings.toggleDiscordRpc}
                    class={`w-10 h-5 rounded-full transition-colors ${settings.settings().discord_rpc_enabled ? "bg-[var(--color-success)]" : "bg-[var(--color-border)]"}`}
                  >
                    <div
                      class={`w-4 h-4 bg-white rounded-full transition-transform shadow ${settings.settings().discord_rpc_enabled ? "translate-x-5" : "translate-x-0.5"}`}
                    />
                  </button>
                </div>

                <Show when={settings.settings().discord_rpc_enabled}>
                  <div class="pl-4 border-l-2 border-[var(--color-border)] space-y-3">
                    <p class="text-xs text-[var(--color-text-tertiary)]">
                      Buttons to show (max 2):{" "}
                      {DISCORD_BUTTONS.filter((b) => getButtonValue(b.key)).length}
                      /2
                    </p>

                    <For each={DISCORD_BUTTONS}>
                      {(btn) => {
                        const isDisabled = () => btn.requiresToken && !settings.settings().vndb_token;

                        return (
                          <label
                            class={`flex items-center gap-2.5 ${isDisabled() ? "opacity-50 cursor-not-allowed" : "cursor-pointer"}`}
                          >
                            <input
                              type="checkbox"
                              checked={getButtonValue(btn.key)}
                              disabled={isDisabled()}
                              onChange={(e) =>
                                handleDiscordButtonChange(btn.key, e.currentTarget.checked, e.currentTarget)
                              }
                              class="w-4 h-4 disabled:opacity-50"
                            />
                            <span class="text-sm text-[var(--color-text-primary)]">{btn.label}</span>
                            <Show when={btn.sublabel}>
                              <span class="text-xs text-[var(--color-text-tertiary)]">{btn.sublabel}</span>
                            </Show>
                            <Show when={btn.requiresToken && !settings.settings().vndb_token}>
                              <span class="text-xs text-amber-600">
                                (requires VNDB token)
                              </span>
                            </Show>
                          </label>
                        );
                      }}
                    </For>
                  </div>
                </Show>
              </div>
            </section>

            <Show when={platform() === "linux"}>
              <div class="h-px bg-[var(--color-border)]" />

              <section class="space-y-4">
                <h3 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
                  Wine
                </h3>
                <div class="space-y-4">
                  <div>
                    <label class="text-sm font-medium text-[var(--color-text-primary)] mb-2 block">
                      Default Wine Version
                    </label>
                    <select
                      value={defaultWineBinary()}
                      onChange={(e) => {
                        setDefaultWineBinary(e.currentTarget.value);
                        saveWineDefaults();
                      }}
                      class="w-full px-3 py-2 bg-[var(--color-bg-secondary)] rounded-xl text-[var(--color-text-primary)] text-sm focus:outline-none focus:ring-2 focus:ring-[var(--color-accent)]"
                    >
                      <Show when={wineVersions().length === 0}>
                        <option value="">No Wine/Proton found</option>
                      </Show>
                      <For
                        each={Object.entries(
                          api.groupWineVersionsByType(wineVersions()),
                        )}
                      >
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
                  </div>

                  <div class="flex items-center justify-between">
                    <div>
                      <span class="text-sm font-medium text-[var(--color-text-primary)]">Use Steam Runtime</span>
                      <p class="text-xs text-[var(--color-text-tertiary)] mt-0.5">
                        Better compatibility for some games
                      </p>
                    </div>
                    <button
                      onClick={() => {
                        setUseSteamRuntime(!useSteamRuntime());
                        saveWineDefaults();
                      }}
                      disabled={!steamRuntimeAvailable()}
                      class={`w-10 h-5 rounded-full transition-colors ${
                        useSteamRuntime() ? "bg-[var(--color-accent)]" : "bg-[var(--color-border)]"
                      } ${!steamRuntimeAvailable() ? "opacity-50 cursor-not-allowed" : ""}`}
                    >
                      <div
                        class={`w-4 h-4 bg-white rounded-full transition-transform shadow ${
                          useSteamRuntime() ? "translate-x-5" : "translate-x-0.5"
                        }`}
                      />
                    </button>
                  </div>
                  <Show when={!steamRuntimeAvailable()}>
                    <p class="text-xs text-amber-600">
                      steam-run not found. Install steam-native-runtime.
                    </p>
                  </Show>
                </div>
              </section>
            </Show>

            <div class="h-px bg-[var(--color-border)]" />

            <section class="space-y-4">
              <h3 class="text-xs uppercase tracking-widest text-[var(--color-text-tertiary)] font-bold">
                Maintenance
              </h3>
              <div class="space-y-4">
                <div>
                  <button
                    onClick={async () => {
                      await api.clearAllCache();
                      alert("Cache cleared!");
                    }}
                    class="w-full px-4 py-2.5 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] rounded-xl text-sm text-[var(--color-text-primary)] font-medium transition-colors"
                  >
                    Clear VNDB Cache
                  </button>
                  <p class="text-xs text-[var(--color-text-tertiary)] mt-1.5">
                    Remove all cached VN and character data
                  </p>
                </div>

                <div>
                  <button
                    onClick={() => updater.checkForUpdates(false)}
                    disabled={updater.status() === "checking"}
                    class="w-full px-4 py-2.5 bg-[var(--color-bg-secondary)] hover:bg-[var(--color-border)] disabled:opacity-50 rounded-xl text-sm text-[var(--color-text-primary)] font-medium transition-colors flex items-center justify-center gap-2"
                  >
                    <IconRefresh
                      class={`w-4 h-4 ${updater.status() === "checking" ? "animate-spin" : ""}`}
                      strokeWidth={1.5}
                    />
                    {updater.status() === "checking"
                      ? "Checking..."
                      : "Check for Updates"}
                  </button>
                  <p class="text-xs text-[var(--color-text-tertiary)] mt-1.5">
                    Current version: v{__APP_VERSION__}
                  </p>
                </div>
              </div>
            </section>

          </div>
        </div>
      </main>
    </div>
  );
}
