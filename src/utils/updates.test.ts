import { describe, it, expect } from "vitest";
import { RELEASE_PAGE_URL, releaseTagUrl } from "./updates";

describe("RELEASE_PAGE_URL", () => {
  it("points at the latest release page", () => {
    expect(RELEASE_PAGE_URL).toBe(
      "https://github.com/betadyne/Poketto/releases/latest"
    );
  });
});

describe("releaseTagUrl", () => {
  it("prefixes a bare version with v", () => {
    expect(releaseTagUrl("0.1.1-beta")).toBe(
      "https://github.com/betadyne/Poketto/releases/tag/v0.1.1-beta"
    );
  });

  it("keeps an existing v prefix as-is", () => {
    expect(releaseTagUrl("v0.1.1-beta")).toBe(
      "https://github.com/betadyne/Poketto/releases/tag/v0.1.1-beta"
    );
  });
});
