import { describe, it, expect } from "vitest";
import {
  groupWineVersionsByType,
  groupWineVersionsBySource,
  getWineTypeDisplayName,
  getWineSourceDisplayName,
} from "./wine";
import type { WineVersion } from "../bindings";

const createVersion = (
  name: string,
  wineType: string,
  source?: string
): WineVersion => ({
  name,
  binary_path: `/path/to/${name}`,
  lib_path: null,
  wine_type: wineType as WineVersion["wine_type"],
  version: "1.0",
  source: source as WineVersion["source"],
});

describe("groupWineVersionsByType", () => {
  it("returns empty object for empty array", () => {
    expect(groupWineVersionsByType([])).toEqual({});
  });

  it("groups versions by wine_type", () => {
    const versions = [
      createVersion("Wine 9.0", "Wine"),
      createVersion("Wine 8.0", "Wine"),
      createVersion("GE-Proton9", "ProtonGE"),
      createVersion("Proton 8.0", "Proton"),
    ];

    const result = groupWineVersionsByType(versions);

    expect(Object.keys(result)).toHaveLength(3);
    expect(result["Wine"]).toHaveLength(2);
    expect(result["ProtonGE"]).toHaveLength(1);
    expect(result["Proton"]).toHaveLength(1);
  });

  it("preserves version order within groups", () => {
    const versions = [
      createVersion("Wine 9.0", "Wine"),
      createVersion("Wine 8.0", "Wine"),
    ];

    const result = groupWineVersionsByType(versions);

    expect(result["Wine"][0].name).toBe("Wine 9.0");
    expect(result["Wine"][1].name).toBe("Wine 8.0");
  });
});

describe("groupWineVersionsBySource", () => {
  it("returns empty object for empty array", () => {
    expect(groupWineVersionsBySource([])).toEqual({});
  });

  it("groups versions by source", () => {
    const versions = [
      createVersion("Wine 9.0", "Wine", "System"),
      createVersion("Proton", "Proton", "Steam"),
      createVersion("Wine 8.0", "Wine", "System"),
    ];

    const result = groupWineVersionsBySource(versions);

    expect(Object.keys(result)).toHaveLength(2);
    expect(result["System"]).toHaveLength(2);
    expect(result["Steam"]).toHaveLength(1);
  });

  it('uses "Unknown" for versions without source', () => {
    const versions = [createVersion("Wine 9.0", "Wine")];

    const result = groupWineVersionsBySource(versions);

    expect(result["Unknown"]).toHaveLength(1);
  });
});

describe("getWineTypeDisplayName", () => {
  it("returns display name for known types", () => {
    expect(getWineTypeDisplayName("Wine")).toBe("Wine");
    expect(getWineTypeDisplayName("WineGE")).toBe("Wine-GE");
    expect(getWineTypeDisplayName("WineStaging")).toBe("Wine Staging");
    expect(getWineTypeDisplayName("WineTKG")).toBe("Wine TKG");
    expect(getWineTypeDisplayName("Proton")).toBe("Steam Proton");
    expect(getWineTypeDisplayName("ProtonGE")).toBe("GE-Proton");
    expect(getWineTypeDisplayName("ProtonCachyOS")).toBe("Proton CachyOS");
    expect(getWineTypeDisplayName("ProtonTKG")).toBe("Proton TKG");
    expect(getWineTypeDisplayName("Lutris")).toBe("Lutris Wine");
    expect(getWineTypeDisplayName("Bottles")).toBe("Bottles");
    expect(getWineTypeDisplayName("Custom")).toBe("Custom");
  });

  it("returns input unchanged for unknown types", () => {
    expect(getWineTypeDisplayName("UnknownType")).toBe("UnknownType");
    expect(getWineTypeDisplayName("")).toBe("");
    expect(getWineTypeDisplayName("SomeNewWine")).toBe("SomeNewWine");
  });
});

describe("getWineSourceDisplayName", () => {
  it("returns display name for known sources", () => {
    expect(getWineSourceDisplayName("System")).toBe("System");
    expect(getWineSourceDisplayName("Opt")).toBe("/opt");
    expect(getWineSourceDisplayName("Steam")).toBe("Steam");
    expect(getWineSourceDisplayName("SteamFlatpak")).toBe("Steam (Flatpak)");
    expect(getWineSourceDisplayName("Lutris")).toBe("Lutris");
    expect(getWineSourceDisplayName("Bottles")).toBe("Bottles");
    expect(getWineSourceDisplayName("BottlesFlatpak")).toBe("Bottles (Flatpak)");
    expect(getWineSourceDisplayName("Custom")).toBe("Custom");
  });

  it("returns input unchanged for unknown sources", () => {
    expect(getWineSourceDisplayName("UnknownSource")).toBe("UnknownSource");
    expect(getWineSourceDisplayName("")).toBe("");
    expect(getWineSourceDisplayName("NewSource")).toBe("NewSource");
  });
});
