import { describe, it, expect } from "vitest";
import { riskColors } from "./constants";

describe("riskColors", () => {
  it("exports an object with risk color mappings", () => {
    expect(riskColors).toBeDefined();
    expect(typeof riskColors).toBe("object");
  });

  it("has exactly 4 risk levels", () => {
    expect(Object.keys(riskColors)).toHaveLength(4);
  });

  it("has low risk color with success styles", () => {
    expect(riskColors.low).toBe(
      "bg-obd-success/10 text-obd-success border-obd-success/20"
    );
  });

  it("has medium risk color with warning styles", () => {
    expect(riskColors.medium).toBe(
      "bg-obd-warning/10 text-obd-warning border-obd-warning/20"
    );
  });

  it("has high risk color with danger styles", () => {
    expect(riskColors.high).toBe(
      "bg-obd-danger/10 text-obd-danger border-obd-danger/30"
    );
  });

  it("has critical risk color with danger styles", () => {
    expect(riskColors.critical).toBe(
      "bg-obd-danger/15 text-obd-danger border-obd-danger/30"
    );
  });

  it("all risk colors are strings", () => {
    Object.values(riskColors).forEach((color) => {
      expect(typeof color).toBe("string");
    });
  });

  it("all risk colors contain Tailwind classes", () => {
    Object.values(riskColors).forEach((color) => {
      expect(color).toMatch(/bg-obd-|text-obd-|border-obd-/);
    });
  });

  it("critical color has more opacity than high color", () => {
    expect(riskColors.critical).toContain("bg-obd-danger/15");
    expect(riskColors.high).toContain("bg-obd-danger/10");
  });

  it("all keys exist and are accessible", () => {
    expect(riskColors.low).toBeDefined();
    expect(riskColors.medium).toBeDefined();
    expect(riskColors.high).toBeDefined();
    expect(riskColors.critical).toBeDefined();
  });
});
