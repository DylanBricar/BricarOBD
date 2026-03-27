import { describe, it, expect } from "vitest";
import { cn, clamp, escapeCSV } from "./utils";

describe("cn() - Tailwind merge with clsx", () => {
  it("combines multiple classes", () => {
    expect(cn("p-4", "text-lg")).toBe("p-4 text-lg");
  });

  it("merges conflicting Tailwind classes", () => {
    const result = cn("p-4", "p-8");
    expect(result).toBe("p-8");
  });

  it("handles conditional classes via clsx", () => {
    expect(cn("p-4", { "text-red-500": true })).toContain("text-red-500");
    expect(cn("p-4", { "text-blue-500": false })).not.toContain("text-blue-500");
  });

  it("removes duplicates and merges responsive classes", () => {
    const result = cn("md:p-4", "md:p-8");
    expect(result).toBe("md:p-8");
  });

  it("handles empty inputs", () => {
    expect(cn("")).toBe("");
    expect(cn("", "p-4")).toBe("p-4");
  });
});

describe("clamp()", () => {
  it("returns value when within range", () => {
    expect(clamp(5, 0, 10)).toBe(5);
  });

  it("returns min when value below range", () => {
    expect(clamp(-5, 0, 10)).toBe(0);
  });

  it("returns max when value above range", () => {
    expect(clamp(15, 0, 10)).toBe(10);
  });

  it("handles edge cases at min/max", () => {
    expect(clamp(0, 0, 10)).toBe(0);
    expect(clamp(10, 0, 10)).toBe(10);
  });

  it("handles negative ranges", () => {
    expect(clamp(-5, -10, -1)).toBe(-5);
    expect(clamp(-15, -10, -1)).toBe(-10);
    expect(clamp(0, -10, -1)).toBe(-1);
  });

  it("handles floating point values", () => {
    expect(clamp(2.5, 0, 5)).toBe(2.5);
    expect(clamp(5.5, 0, 5)).toBe(5);
    expect(clamp(-0.5, 0, 5)).toBe(0);
  });
});

describe("escapeCSV()", () => {
  it("returns plain string when no special chars", () => {
    expect(escapeCSV("hello")).toBe("hello");
  });

  it("escapes double quotes by doubling them and wraps in quotes", () => {
    expect(escapeCSV('say "hello"')).toBe('"say ""hello"""');
  });

  it("wraps strings with commas in quotes", () => {
    expect(escapeCSV("hello,world")).toBe('"hello,world"');
  });

  it("wraps strings with newlines in quotes", () => {
    expect(escapeCSV("hello\nworld")).toBe('"hello\nworld"');
  });

  it("wraps strings with carriage returns in quotes", () => {
    expect(escapeCSV("hello\rworld")).toBe('"hello\rworld"');
  });

  it("wraps strings starting with formula injection chars", () => {
    expect(escapeCSV("=SUM(A1:A10)")).toBe('"=SUM(A1:A10)"');
    expect(escapeCSV("+malicious")).toBe('"+malicious"');
    expect(escapeCSV("-malicious")).toBe('"-malicious"');
    expect(escapeCSV("@malicious")).toBe('"@malicious"');
  });

  it("wraps strings starting with tab character in quotes", () => {
    expect(escapeCSV("\thello")).toBe('"\thello"');
  });

  it("handles numbers converted to string", () => {
    expect(escapeCSV(123)).toBe("123");
    expect(escapeCSV(45.67)).toBe("45.67");
  });

  it("handles empty string", () => {
    expect(escapeCSV("")).toBe("");
  });

  it("combines multiple special chars with proper escaping", () => {
    expect(escapeCSV('he "said" hello, world')).toBe('"he ""said"" hello, world"');
  });

  it("handles carriage return + newline correctly", () => {
    expect(escapeCSV("hello\r\nworld")).toBe('"hello\r\nworld"');
  });

  it("wraps strings with quotes first, then checks other chars", () => {
    expect(escapeCSV('data with "quotes"')).toBe('"data with ""quotes"""');
  });
});
