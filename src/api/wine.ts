import { commands } from "../bindings";
import type { WineSettings, WineVersion } from "../bindings";

export type {
  WineSettings,
  WineVersion,
  WineType,
  WineSource,
  GameType,
} from "../bindings";

export const getPlatform = () => commands.getPlatform();

export const isLinux = async () => (await commands.getPlatform()) === "linux";

export const getAvailableWineVersions = () =>
  commands.getAvailableWineVersions();

export const refreshWineVersions = () => commands.refreshWineVersions();

export const getDefaultWineSettings = () => commands.getDefaultWineSettings();

export const getDefaultPrefixPath = (gameId: string) =>
  commands.getDefaultPrefixPath(gameId);

export const saveGameWineSettings = (
  gameId: string,
  wineSettings: WineSettings,
) => commands.saveGameWineSettings(gameId, wineSettings);

export const saveGlobalWineDefaults = (
  prefix: string | null,
  binary: string | null,
  useSteamRuntime: boolean,
) => commands.saveGlobalWineDefaults(prefix, binary, useSteamRuntime);

export const isSteamRuntimeAvailable = () => commands.isSteamRuntimeAvailable();

export const validateWineBinary = (binaryPath: string) =>
  commands.validateWineBinary(binaryPath);

export const groupWineVersionsByType = (versions: WineVersion[]) => {
  const groups: Record<string, WineVersion[]> = {};

  for (const version of versions) {
    const type = version.wine_type;
    if (!groups[type]) {
      groups[type] = [];
    }
    groups[type].push(version);
  }

  return groups;
};

export const groupWineVersionsBySource = (versions: WineVersion[]) => {
  const groups: Record<string, WineVersion[]> = {};

  for (const version of versions) {
    const source = version.source || "Unknown";
    if (!groups[source]) {
      groups[source] = [];
    }
    groups[source].push(version);
  }

  return groups;
};

export const getWineTypeDisplayName = (type: string): string => {
  const names: Record<string, string> = {
    Wine: "Wine",
    WineGE: "Wine-GE",
    WineStaging: "Wine Staging",
    WineTKG: "Wine TKG",
    Proton: "Steam Proton",
    ProtonGE: "GE-Proton",
    ProtonCachyOS: "Proton CachyOS",
    ProtonTKG: "Proton TKG",
    Lutris: "Lutris Wine",
    Bottles: "Bottles",
    Custom: "Custom",
  };
  return names[type] || type;
};

export const getWineSourceDisplayName = (source: string): string => {
  const names: Record<string, string> = {
    System: "System",
    Opt: "/opt",
    Steam: "Steam",
    SteamFlatpak: "Steam (Flatpak)",
    Lutris: "Lutris",
    Bottles: "Bottles",
    BottlesFlatpak: "Bottles (Flatpak)",
    Custom: "Custom",
  };
  return names[source] || source;
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
