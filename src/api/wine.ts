import { commands } from "../bindings";
import type { WineSettings, WineVersion } from "../bindings";

export type {
  WineSettings,
  WineVersion,
  WineType,
  GameType,
} from "../bindings";

/**
 * Get the current platform
 * @returns "linux" | "windows" | "macos" | "unknown"
 */
export const getPlatform = () => commands.getPlatform();

export const isLinux = async () => (await commands.getPlatform()) === "linux";

export const getAvailableWineVersions = () =>
  commands.getAvailableWineVersions();

export const getDefaultWineSettings = () => commands.getDefaultWineSettings();

/**
 * Get default Wine prefix path for a game
 * @param gameId - The game ID
 * @returns Path like ~/Games/Poketto/Prefixes/{gameId}
 */
export const getDefaultPrefixPath = (gameId: string) =>
  commands.getDefaultPrefixPath(gameId);

/**
 * Save Wine settings for a specific game
 * @param gameId - The game ID
 * @param wineSettings - The Wine settings to save
 */
export const saveGameWineSettings = (
  gameId: string,
  wineSettings: WineSettings,
) => commands.saveGameWineSettings(gameId, wineSettings);

/**
 * Save global Wine defaults
 * @param prefix - Default Wine prefix path (null for per-game prefixes)
 * @param binary - Default Wine binary path
 * @param useSteamRuntime - Whether to use Steam Runtime (steam-run)
 */
export const saveGlobalWineDefaults = (
  prefix: string | null,
  binary: string | null,
  useSteamRuntime: boolean,
) => commands.saveGlobalWineDefaults(prefix, binary, useSteamRuntime);

export const isSteamRuntimeAvailable = () => commands.isSteamRuntimeAvailable();

/**
 * Validate a Wine binary path
 * @param binaryPath - Path to the Wine/Proton binary
 * @returns Wine version string if valid
 */
export const validateWineBinary = (binaryPath: string) =>
  commands.validateWineBinary(binaryPath);

export const groupWineVersionsByType = (versions: WineVersion[]) => {
  const groups: Record<string, WineVersion[]> = {
    Wine: [],
    Proton: [],
    ProtonGE: [],
    ProtonCachyOS: [],
  };

  for (const version of versions) {
    if (groups[version.wine_type]) {
      groups[version.wine_type].push(version);
    }
  }

  return groups;
};

export const getWineTypeDisplayName = (type: string): string => {
  const names: Record<string, string> = {
    Wine: "Wine",
    Proton: "Steam Proton",
    ProtonGE: "GE-Proton",
    ProtonCachyOS: "Proton CachyOS",
  };
  return names[type] || type;
};

export const createDefaultWineSettings = async (
  gameId: string,
): Promise<WineSettings> => {
  const defaults = await getDefaultWineSettings();
  const prefixPath = await getDefaultPrefixPath(gameId);

  return {
    ...defaults,
    use_global_prefix: false,
    wine_prefix: prefixPath,
  };
};
