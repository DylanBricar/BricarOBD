import { useSyncExternalStore } from "react";

export type ThemeMode = "system" | "dark" | "light";
export type ResolvedTheme = "dark" | "light";

function getInitialThemeMode(): ThemeMode {
  try {
    const saved = localStorage.getItem("bricarobd_theme");
    if (saved === "dark" || saved === "light" || saved === "system") return saved;
  } catch {}
  return "system";
}

let themeMode: ThemeMode = getInitialThemeMode();
const listeners = new Set<() => void>();

function emitChange() {
  listeners.forEach((l) => l());
}

function getSystemTheme(): ResolvedTheme {
  return window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "light";
}

function getResolvedTheme(): ResolvedTheme {
  return themeMode === "system" ? getSystemTheme() : themeMode;
}

function applyTheme() {
  const resolved = getResolvedTheme();
  document.documentElement.classList.toggle("dark", resolved === "dark");
  document.documentElement.classList.toggle("light", resolved === "light");
}

// Listen for system theme changes
const systemThemeQuery = window.matchMedia("(prefers-color-scheme: dark)");
const handleSystemThemeChange = () => {
  if (themeMode === "system") {
    applyTheme();
    emitChange();
  }
};
systemThemeQuery.addEventListener("change", handleSystemThemeChange);

// HMR cleanup: remove stale listener on module re-evaluation
if ((import.meta as any).hot) {
  (import.meta as any).hot.dispose(() => {
    systemThemeQuery.removeEventListener("change", handleSystemThemeChange);
  });
}

export function setThemeMode(mode: ThemeMode) {
  themeMode = mode;
  try {
    localStorage.setItem("bricarobd_theme", mode);
  } catch {}
  applyTheme();
  emitChange();
}

export function getThemeMode(): ThemeMode {
  return themeMode;
}

function themeSubscribe(callback: () => void) {
  listeners.add(callback);
  return () => listeners.delete(callback);
}

function getThemeModeSnapshot(): ThemeMode {
  return themeMode;
}

function getResolvedThemeSnapshot(): ResolvedTheme {
  return getResolvedTheme();
}

export function useThemeStore() {
  const mode = useSyncExternalStore(themeSubscribe, getThemeModeSnapshot);
  const resolved = useSyncExternalStore(themeSubscribe, getResolvedThemeSnapshot);

  return { mode, resolved, setThemeMode };
}

// Initialize on load
applyTheme();
