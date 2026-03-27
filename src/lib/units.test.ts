import { describe, it, expect, beforeEach } from "vitest";
import { renderHook, act } from "@testing-library/react";
import i18n from "./i18n";
import { convertValue, useUnitSystem } from "./units";

describe("convertValue()", () => {
  it("returns metric value and unit unchanged when system is metric", () => {
    const result = convertValue(100, "km/h", "metric");
    expect(result.value).toBe(100);
    expect(result.unit).toBe("km/h");
  });

  it("converts km/h to mph", () => {
    const result = convertValue(100, "km/h", "imperial");
    expect(result.value).toBeCloseTo(62.1371, 4);
    expect(result.unit).toBe("mph");
  });

  it("converts °C to °F", () => {
    const result = convertValue(0, "°C", "imperial");
    expect(result.value).toBeCloseTo(32, 1);
    expect(result.unit).toBe("°F");
  });

  it("converts °C to °F with negative values", () => {
    const result = convertValue(-40, "°C", "imperial");
    expect(result.value).toBeCloseTo(-40, 1);
    expect(result.unit).toBe("°F");
  });

  it("converts kPa to psi", () => {
    const result = convertValue(100, "kPa", "imperial");
    expect(result.value).toBeCloseTo(14.5038, 4);
    expect(result.unit).toBe("psi");
  });

  it("converts bar to psi", () => {
    const result = convertValue(1, "bar", "imperial");
    expect(result.value).toBeCloseTo(14.5038, 4);
    expect(result.unit).toBe("psi");
  });

  it("converts L/h to gal/h", () => {
    const result = convertValue(10, "L/h", "imperial");
    expect(result.value).toBeCloseTo(2.64172, 5);
    expect(result.unit).toBe("gal/h");
  });

  it("converts g/s to lb/min", () => {
    const result = convertValue(1, "g/s", "imperial");
    expect(result.value).toBeCloseTo(0.132277, 5);
    expect(result.unit).toBe("lb/min");
  });

  it("converts km to mi", () => {
    const result = convertValue(100, "km", "imperial");
    expect(result.value).toBeCloseTo(62.1371, 4);
    expect(result.unit).toBe("mi");
  });

  it("returns original value and unit when unit is unknown", () => {
    const result = convertValue(50, "unknown_unit", "imperial");
    expect(result.value).toBe(50);
    expect(result.unit).toBe("unknown_unit");
  });

  it("handles zero values", () => {
    const result = convertValue(0, "km/h", "imperial");
    expect(result.value).toBe(0);
    expect(result.unit).toBe("mph");
  });

  it("handles negative values", () => {
    const result = convertValue(-10, "°C", "imperial");
    expect(result.value).toBeCloseTo(-10 * 9 / 5 + 32, 1);
    expect(result.unit).toBe("°F");
  });

  it("handles floating point values", () => {
    const result = convertValue(45.5, "km/h", "imperial");
    expect(result.value).toBeCloseTo(45.5 * 0.621371, 5);
    expect(result.unit).toBe("mph");
  });
});

describe("useUnitSystem() hook — language-based derivation", () => {
  beforeEach(() => {
    i18n.changeLanguage("fr");
  });

  it("returns metric when language is French", () => {
    const { result } = renderHook(() => useUnitSystem());
    expect(result.current.system).toBe("metric");
  });

  it("returns imperial when language is English", async () => {
    await act(async () => {
      await i18n.changeLanguage("en");
    });
    const { result } = renderHook(() => useUnitSystem());
    expect(result.current.system).toBe("imperial");
  });

  it("updates reactively when language changes", async () => {
    const { result } = renderHook(() => useUnitSystem());
    expect(result.current.system).toBe("metric");

    await act(async () => {
      await i18n.changeLanguage("en");
    });
    expect(result.current.system).toBe("imperial");

    await act(async () => {
      await i18n.changeLanguage("fr");
    });
    expect(result.current.system).toBe("metric");
  });
});
