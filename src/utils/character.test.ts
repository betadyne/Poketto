import { describe, it, expect } from "vitest";
import { sexLabel } from "./character";

describe("sexLabel", () => {
  it("maps m to Male", () => {
    expect(sexLabel("m")).toBe("Male");
  });

  it("maps f to Female", () => {
    expect(sexLabel("f")).toBe("Female");
  });

  it("passes unknown codes through for display", () => {
    expect(sexLabel("b")).toBe("b");
  });

  it("returns Unknown for missing codes", () => {
    expect(sexLabel(null)).toBe("Unknown");
    expect(sexLabel(undefined)).toBe("Unknown");
  });
});
