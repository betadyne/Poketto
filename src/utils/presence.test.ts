import { describe, it, expect } from "vitest";
import { epochToLocalInput, localInputToEpoch, parseOptionalInt } from "./presence";

describe("epochToLocalInput", () => {
  it("returns empty string for missing epochs", () => {
    expect(epochToLocalInput(null)).toBe("");
    expect(epochToLocalInput(undefined)).toBe("");
  });

  it("round-trips through localInputToEpoch", () => {
    const epoch = 1788640842;
    expect(localInputToEpoch(epochToLocalInput(epoch))).toBe(epoch);
  });
});

describe("localInputToEpoch", () => {
  it("returns null for blank or invalid input", () => {
    expect(localInputToEpoch("")).toBe(null);
    expect(localInputToEpoch("   ")).toBe(null);
    expect(localInputToEpoch("not-a-date")).toBe(null);
  });
});

describe("parseOptionalInt", () => {
  it("parses non-negative integers", () => {
    expect(parseOptionalInt("4")).toBe(4);
    expect(parseOptionalInt(" 10 ")).toBe(10);
  });

  it("returns null for blank, negative, or fractional input", () => {
    expect(parseOptionalInt("")).toBe(null);
    expect(parseOptionalInt("-1")).toBe(null);
    expect(parseOptionalInt("2.5")).toBe(null);
    expect(parseOptionalInt("abc")).toBe(null);
  });
});
