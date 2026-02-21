import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { formatPlayTime, formatLastPlayed } from "./formatters";

describe("formatPlayTime", () => {
  describe("when minutes < 60", () => {
    it("returns 0m for 0 minutes", () => {
      expect(formatPlayTime(0)).toBe("0m");
    });

    it("returns 30m for 30 minutes", () => {
      expect(formatPlayTime(30)).toBe("30m");
    });

    it("returns 59m for 59 minutes", () => {
      expect(formatPlayTime(59)).toBe("59m");
    });
  });

  describe("when minutes >= 60", () => {
    it("returns 1h 0m for 60 minutes", () => {
      expect(formatPlayTime(60)).toBe("1h 0m");
    });

    it("returns 1h 30m for 90 minutes", () => {
      expect(formatPlayTime(90)).toBe("1h 30m");
    });

    it("returns 2h 30m for 150 minutes", () => {
      expect(formatPlayTime(150)).toBe("2h 30m");
    });

    it("returns 10h 0m for 600 minutes", () => {
      expect(formatPlayTime(600)).toBe("10h 0m");
    });

    it("returns 100h 59m for 6059 minutes", () => {
      expect(formatPlayTime(6059)).toBe("100h 59m");
    });
  });
});

describe("formatLastPlayed", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2024-06-15T12:00:00Z"));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "Never" for null', () => {
    expect(formatLastPlayed(null)).toBe("Never");
  });

  it('returns "Today" for same day', () => {
    expect(formatLastPlayed("2024-06-15T08:00:00Z")).toBe("Today");
  });

  it('returns "Yesterday" for previous day', () => {
    expect(formatLastPlayed("2024-06-14T12:00:00Z")).toBe("Yesterday");
  });

  it('returns "X days ago" for 2-6 days', () => {
    expect(formatLastPlayed("2024-06-13T12:00:00Z")).toBe("2 days ago");
    expect(formatLastPlayed("2024-06-10T12:00:00Z")).toBe("5 days ago");
  });

  it('returns "X weeks ago" for 7-29 days', () => {
    expect(formatLastPlayed("2024-06-08T12:00:00Z")).toBe("1 weeks ago");
    expect(formatLastPlayed("2024-06-01T12:00:00Z")).toBe("2 weeks ago");
  });

  it('returns "X months ago" for 30-364 days', () => {
    expect(formatLastPlayed("2024-05-15T12:00:00Z")).toBe("1 months ago");
    expect(formatLastPlayed("2024-03-15T12:00:00Z")).toBe("3 months ago");
  });

  it("returns formatted date for 365+ days", () => {
    const result = formatLastPlayed("2023-01-15T12:00:00Z");
    expect(result).not.toBe("Never");
    expect(result).not.toContain("ago");
  });
});
