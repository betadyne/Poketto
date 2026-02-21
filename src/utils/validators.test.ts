import { describe, it, expect } from "vitest";
import { isVndbId } from "./validators";

describe("isVndbId", () => {
  describe("valid VNDB IDs", () => {
    it("returns true for lowercase v followed by digits", () => {
      expect(isVndbId("v123")).toBe(true);
      expect(isVndbId("v1")).toBe(true);
      expect(isVndbId("v99999")).toBe(true);
    });

    it("returns true for uppercase V followed by digits", () => {
      expect(isVndbId("V123")).toBe(true);
      expect(isVndbId("V1")).toBe(true);
    });

    it("handles whitespace", () => {
      expect(isVndbId("  v123  ")).toBe(true);
      expect(isVndbId("\tv123\n")).toBe(true);
    });
  });

  describe("invalid VNDB IDs", () => {
    it("returns false for empty string", () => {
      expect(isVndbId("")).toBe(false);
    });

    it("returns false for only digits", () => {
      expect(isVndbId("123")).toBe(false);
      expect(isVndbId("99999")).toBe(false);
    });

    it("returns false for v without digits", () => {
      expect(isVndbId("v")).toBe(false);
      expect(isVndbId("V")).toBe(false);
    });

    it("returns false for text without v prefix", () => {
      expect(isVndbId("abc")).toBe(false);
      expect(isVndbId("game123")).toBe(false);
    });

    it("returns false for vn prefix", () => {
      expect(isVndbId("vn123")).toBe(false);
    });

    it("returns false for mixed content after v", () => {
      expect(isVndbId("v123abc")).toBe(false);
      expect(isVndbId("v12.3")).toBe(false);
    });

    it("returns false for multiple v prefixes", () => {
      expect(isVndbId("vv123")).toBe(false);
    });
  });
});
