import { describe, it, expect } from "vitest";
import { shouldBlur } from "./blur";
import type { VndbImage } from "../bindings";

describe("shouldBlur", () => {
  const createImage = (sexual: number, violence: number): VndbImage => ({
    url: "https://example.com/image.jpg",
    sexual,
    violence,
  });

  describe("when blurNsfw is false", () => {
    it("returns false regardless of image content", () => {
      expect(shouldBlur(createImage(2, 2), false)).toBe(false);
      expect(shouldBlur(createImage(0, 0), false)).toBe(false);
      expect(shouldBlur(null, false)).toBe(false);
    });
  });

  describe("when blurNsfw is true", () => {
    it("returns false for null image", () => {
      expect(shouldBlur(null, true)).toBe(false);
    });

    it("returns false for safe images (sexual=0, violence=0)", () => {
      expect(shouldBlur(createImage(0, 0), true)).toBe(false);
    });

    it("returns true when sexual >= 1", () => {
      expect(shouldBlur(createImage(1, 0), true)).toBe(true);
      expect(shouldBlur(createImage(2, 0), true)).toBe(true);
    });

    it("returns true when violence >= 1", () => {
      expect(shouldBlur(createImage(0, 1), true)).toBe(true);
      expect(shouldBlur(createImage(0, 2), true)).toBe(true);
    });

    it("returns true when both sexual and violence >= 1", () => {
      expect(shouldBlur(createImage(1, 1), true)).toBe(true);
      expect(shouldBlur(createImage(2, 2), true)).toBe(true);
    });
  });

  describe("edge cases", () => {
    it("handles undefined sexual/violence gracefully", () => {
      const imageWithUndefined: VndbImage = {
        url: "https://example.com/image.jpg",
      };

      expect(shouldBlur(imageWithUndefined, true)).toBe(false);
    });

    it("handles image with only url", () => {
      const minimalImage: VndbImage = { url: "https://example.com/image.jpg" };
      expect(shouldBlur(minimalImage, true)).toBe(false);
      expect(shouldBlur(minimalImage, false)).toBe(false);
    });
  });
});
