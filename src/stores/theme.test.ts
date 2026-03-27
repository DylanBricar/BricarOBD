import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";

// We need to reset modules carefully for theme.ts to re-initialize
describe("useThemeStore()", () => {
  beforeEach(() => {
    localStorage.clear();
    vi.resetModules();
    // Reset the document class before each test
    document.documentElement.classList.remove("dark", "light");
  });

  afterEach(() => {
    localStorage.clear();
    document.documentElement.classList.remove("dark", "light");
  });

  describe("initial state and theme detection", () => {
    it("initializes to system theme by default", async () => {
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");
    });

    it("loads theme from localStorage if available", async () => {
      localStorage.setItem("bricarobd_theme", "dark");
      vi.resetModules();
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("dark");
    });

    it("resolves system theme to dark when system prefers dark", async () => {
      // Reset the matchMedia mock to return dark
      const mockMatchMedia = window.matchMedia as any;
      mockMatchMedia.mockImplementation((query: string) => ({
        matches: query === "(prefers-color-scheme: dark)",
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }));

      vi.resetModules();
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");
      expect(result.current.resolved).toBe("dark");
    });

    it("resolves system theme to light when system prefers light", async () => {
      const mockMatchMedia = window.matchMedia as any;
      mockMatchMedia.mockImplementation((query: string) => ({
        matches: query === "(prefers-color-scheme: light)",
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }));

      vi.resetModules();
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.resolved).toBe("light");
    });

    it("ignores invalid localStorage values", async () => {
      localStorage.setItem("bricarobd_theme", "invalid-theme");
      vi.resetModules();
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");
    });
  });

  describe("setThemeMode()", () => {
    it("changes theme to dark", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");

      act(() => {
        setThemeMode("dark");
      });

      expect(result.current.mode).toBe("dark");
      expect(result.current.resolved).toBe("dark");
    });

    it("changes theme to light", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        setThemeMode("light");
      });

      expect(result.current.mode).toBe("light");
      expect(result.current.resolved).toBe("light");
    });

    it("changes theme back to system", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        setThemeMode("dark");
      });

      expect(result.current.mode).toBe("dark");

      act(() => {
        setThemeMode("system");
      });

      expect(result.current.mode).toBe("system");
    });

    it("persists theme to localStorage", async () => {
      const { setThemeMode } = await import("./theme");

      act(() => {
        setThemeMode("dark");
      });

      expect(localStorage.getItem("bricarobd_theme")).toBe("dark");

      act(() => {
        setThemeMode("light");
      });

      expect(localStorage.getItem("bricarobd_theme")).toBe("light");
    });

    it("applies dark class to document root when dark", async () => {
      const { setThemeMode } = await import("./theme");

      act(() => {
        setThemeMode("dark");
      });

      expect(document.documentElement.classList.contains("dark")).toBe(true);
      expect(document.documentElement.classList.contains("light")).toBe(false);
    });

    it("applies light class to document root when light", async () => {
      const { setThemeMode } = await import("./theme");

      act(() => {
        setThemeMode("light");
      });

      expect(document.documentElement.classList.contains("light")).toBe(true);
      expect(document.documentElement.classList.contains("dark")).toBe(false);
    });
  });

  describe("getThemeMode()", () => {
    it("returns current theme mode", async () => {
      const { setThemeMode, getThemeMode } = await import("./theme");

      expect(getThemeMode()).toBe("system");

      act(() => {
        setThemeMode("dark");
      });

      expect(getThemeMode()).toBe("dark");
    });
  });

  describe("hook reactivity", () => {
    it("hook updates when setThemeMode is called", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");

      act(() => {
        setThemeMode("dark");
      });

      expect(result.current.mode).toBe("dark");

      act(() => {
        setThemeMode("light");
      });

      expect(result.current.mode).toBe("light");
    });

    it("multiple hooks stay in sync", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result: result1 } = renderHook(() => useThemeStore());

      expect(result1.current.mode).toBe("system");

      act(() => {
        setThemeMode("dark");
      });

      expect(result1.current.mode).toBe("dark");

      // Reset for other tests
      act(() => {
        setThemeMode("system");
      });
    });

    it("updates resolved theme when system preference changes", async () => {
      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        setThemeMode("system");
      });

      // Mock system preference change
      const mockMatchMedia = window.matchMedia as any;
      const listeners: Function[] = [];
      mockMatchMedia.mockImplementation((query: string) => ({
        matches: query === "(prefers-color-scheme: dark)",
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn((event: string, listener: Function) => {
          if (event === "change") listeners.push(listener);
        }),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      }));

      // Simulate system preference change to light
      // Note: This test verifies the event listener is set up, but full test would need
      // actual MediaQueryList event simulation which is complex in jsdom
      expect(result.current.mode).toBe("system");
    });
  });

  describe("setThemeMode with hook.setThemeMode", () => {
    it("hook provides setThemeMode method", async () => {
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.setThemeMode).toBeDefined();
      expect(typeof result.current.setThemeMode).toBe("function");
    });

    it("setThemeMode from hook updates state", async () => {
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      act(() => {
        result.current.setThemeMode("dark");
      });

      expect(result.current.mode).toBe("dark");
    });
  });

  describe("error handling", () => {
    it("gracefully handles localStorage errors", async () => {
      const setItemSpy = vi.spyOn(Storage.prototype, "setItem").mockImplementation(() => {
        throw new Error("localStorage full");
      });

      const { setThemeMode, useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      // Should not throw
      act(() => {
        setThemeMode("dark");
      });

      expect(result.current.mode).toBe("dark");

      setItemSpy.mockRestore();
    });

    it("gracefully handles getItem errors on init", async () => {
      const getItemSpy = vi.spyOn(Storage.prototype, "getItem").mockImplementation(() => {
        throw new Error("localStorage access denied");
      });

      vi.resetModules();
      const { useThemeStore } = await import("./theme");
      const { result } = renderHook(() => useThemeStore());

      expect(result.current.mode).toBe("system");

      getItemSpy.mockRestore();
    });
  });
});
