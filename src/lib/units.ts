import { useSyncExternalStore } from "react";

export type UnitSystem = "metric" | "imperial";

interface ConversionRule {
  fromUnit: string;
  toUnit: string;
  convert: (value: number) => number;
}

const conversions: ConversionRule[] = [
  { fromUnit: "km/h", toUnit: "mph", convert: (v) => v * 0.621371 },
  { fromUnit: "°C", toUnit: "°F", convert: (v) => v * 9 / 5 + 32 },
  { fromUnit: "kPa", toUnit: "psi", convert: (v) => v * 0.145038 },
  { fromUnit: "bar", toUnit: "psi", convert: (v) => v * 14.5038 },
  { fromUnit: "L/h", toUnit: "gal/h", convert: (v) => v * 0.264172 },
  { fromUnit: "g/s", toUnit: "lb/min", convert: (v) => v * 0.132277 },
  { fromUnit: "km", toUnit: "mi", convert: (v) => v * 0.621371 },
];

export function convertValue(value: number, unit: string, system: UnitSystem): { value: number; unit: string } {
  if (system === "metric") return { value, unit };
  const rule = conversions.find((c) => c.fromUnit === unit);
  if (!rule) return { value, unit };
  return { value: rule.convert(value), unit: rule.toUnit };
}

// --- useSyncExternalStore-based reactive state ---

let currentSystem: UnitSystem = (localStorage.getItem("bricarobd_unit_system") as UnitSystem) || "metric";
const listeners = new Set<() => void>();

function emitChange() {
  listeners.forEach((l) => l());
}

export function setUnitSystem(system: UnitSystem) {
  currentSystem = system;
  localStorage.setItem("bricarobd_unit_system", system);
  emitChange();
}

function subscribe(callback: () => void): () => void {
  listeners.add(callback);
  return () => listeners.delete(callback);
}

function getSnapshot(): UnitSystem {
  return currentSystem;
}

export function useUnitSystem() {
  const system = useSyncExternalStore(subscribe, getSnapshot);
  return { system, setUnitSystem };
}
