import { describe, it, expect, vi } from "vitest";
import { getTheoreticalMax, generateCSV } from "./livedata";
import type { PidValue } from "@/stores/vehicle";

vi.mock("@/lib/utils", () => ({
  escapeCSV: (value: string | number): string => {
    const str = String(value);
    const escaped = str.replace(/"/g, '""');
    if (/[,"\n\r]/.test(str) || /^[=+\-@\t\r]/.test(str)) {
      return `"${escaped}"`;
    }
    return escaped;
  },
}));

describe("getTheoreticalMax()", () => {
  it("returns 8000 for RPM", () => {
    expect(getTheoreticalMax("RPM")).toBe(8000);
  });

  it("returns 250 for km/h", () => {
    expect(getTheoreticalMax("km/h")).toBe(250);
  });

  it("returns 120 for °C", () => {
    expect(getTheoreticalMax("°C")).toBe(120);
  });

  it("returns 100 for % (percentage)", () => {
    expect(getTheoreticalMax("%")).toBe(100);
  });

  it("returns 5 for bar", () => {
    expect(getTheoreticalMax("bar")).toBe(5);
  });

  it("returns 14 for V (voltage)", () => {
    expect(getTheoreticalMax("V")).toBe(14);
  });

  it("returns 100 for A (amperage)", () => {
    expect(getTheoreticalMax("A")).toBe(100);
  });

  it("returns 200 for kPa", () => {
    expect(getTheoreticalMax("kPa")).toBe(200);
  });

  it("returns 50 for ms (milliseconds)", () => {
    expect(getTheoreticalMax("ms")).toBe(50);
  });

  it("returns 5 for g (acceleration)", () => {
    expect(getTheoreticalMax("g")).toBe(5);
  });

  it("returns 100 for unknown units", () => {
    expect(getTheoreticalMax("unknown")).toBe(100);
  });

  it("returns 100 for empty string", () => {
    expect(getTheoreticalMax("")).toBe(100);
  });

  it("returns 100 for arbitrary units", () => {
    expect(getTheoreticalMax("Hz")).toBe(100);
    expect(getTheoreticalMax("Nm")).toBe(100);
  });

  it("is case-sensitive (returns fallback for wrong case)", () => {
    expect(getTheoreticalMax("rpm")).toBe(100);
    expect(getTheoreticalMax("RPM")).toBe(8000);
  });
});

describe("generateCSV()", () => {
  const mockHeader = "Timestamp,PID,Name,Value,Unit,Min,Max";

  it("returns header only when pidData is empty", () => {
    const pidData = new Map<number, PidValue>();
    const result = generateCSV(pidData, mockHeader);
    expect(result).toBe(mockHeader);
  });

  it("exports single PID with correct format", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "Engine RPM",
          value: 1500.5,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    const lines = result.split("\n");

    expect(lines).toHaveLength(2);
    expect(lines[0]).toBe(mockHeader);
    expect(lines[1]).toContain("0x0C");
    expect(lines[1]).toContain("Engine RPM");
    expect(lines[1]).toContain("1500.50");
  });

  it("formats PID as two-digit hex with uppercase", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
      [
        0xf1,
        {
          pid: 0xf1,
          name: "Speed",
          value: 85,
          unit: "km/h",
          min: 0,
          max: 250,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    expect(result).toContain("0x0C");
    expect(result).toContain("0xF1");
  });

  it("formats values with 2 decimal places", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 1234.56789,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    expect(result).toContain("1234.57");
  });

  it("exports multiple PIDs", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
      [
        0x05,
        {
          pid: 0x05,
          name: "Temperature",
          value: 85.5,
          unit: "°C",
          min: -40,
          max: 120,
          history: [],
          timestamp: Date.now(),
        },
      ],
      [
        0x0d,
        {
          pid: 0x0d,
          name: "Speed",
          value: 50.25,
          unit: "km/h",
          min: 0,
          max: 250,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    const lines = result.split("\n");
    expect(lines).toHaveLength(4);
  });

  it("exports buffer data with timestamps when buffer is provided", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 3000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const timestamp1 = new Date("2026-03-28T10:00:00Z");
    const timestamp2 = new Date("2026-03-28T10:00:01Z");

    const buffer = [
      {
        timestamp: timestamp1,
        snapshot: { 0x0c: 1500 },
      },
      {
        timestamp: timestamp2,
        snapshot: { 0x0c: 2000 },
      },
    ];

    const result = generateCSV(pidData, mockHeader, buffer);
    const lines = result.split("\n");

    expect(lines).toHaveLength(3);
    expect(lines[1]).toContain(timestamp1.toISOString());
    expect(lines[2]).toContain(timestamp2.toISOString());
  });

  it("skips undefined values in buffer snapshot", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
      [
        0x0d,
        {
          pid: 0x0d,
          name: "Speed",
          value: 50,
          unit: "km/h",
          min: 0,
          max: 250,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const timestamp = new Date("2026-03-28T10:00:00Z");
    const buffer = [
      {
        timestamp,
        snapshot: { 0x0c: 1500 },
      },
    ];

    const result = generateCSV(pidData, mockHeader, buffer);
    const lines = result.split("\n");

    expect(lines).toHaveLength(2);
    expect(lines[1]).toContain("0x0C");
    expect(lines[1]).not.toContain("0x0D");
  });

  it("includes current snapshot timestamp when buffer is empty", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader, []);
    const lines = result.split("\n");

    expect(lines).toHaveLength(2);
    expect(lines[1]).toMatch(/\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
  });

  it("formats min and max values with 2 decimal places", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x05,
        {
          pid: 0x05,
          name: "Temperature",
          value: 85,
          unit: "°C",
          min: -40.123,
          max: 120.789,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    expect(result).toContain("-40.12");
    expect(result).toContain("120.79");
  });

  it("escapes CSV values properly", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "Engine,RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    expect(result).toContain('"Engine,RPM"');
  });

  it("returns newline-separated rows", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
      [
        0x0d,
        {
          pid: 0x0d,
          name: "Speed",
          value: 50,
          unit: "km/h",
          min: 0,
          max: 250,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    const lines = result.split("\n");

    expect(lines.every((line) => line.length > 0)).toBe(true);
  });

  it("handles zero values correctly", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 0,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    expect(result).toContain("0.00");
  });

  it("includes all required columns in output", () => {
    const pidData = new Map<number, PidValue>([
      [
        0x0c,
        {
          pid: 0x0c,
          name: "RPM",
          value: 2000,
          unit: "RPM",
          min: 0,
          max: 8000,
          history: [],
          timestamp: Date.now(),
        },
      ],
    ]);

    const result = generateCSV(pidData, mockHeader);
    const lines = result.split("\n");
    const dataLine = lines[1];

    const parts = dataLine.split(",");
    expect(parts.length).toBeGreaterThanOrEqual(6);
    expect(dataLine).toMatch(/\d{4}-\d{2}-\d{2}/);
    expect(dataLine).toContain("0x0C");
    expect(dataLine).toContain("RPM");
  });
});
