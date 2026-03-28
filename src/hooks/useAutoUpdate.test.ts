import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useAutoUpdate } from "./useAutoUpdate";

vi.mock("@tauri-apps/plugin-updater", () => ({
  check: vi.fn(),
}));

vi.mock("@tauri-apps/plugin-process", () => ({
  relaunch: vi.fn(),
}));

vi.mock("@/lib/devlog", () => ({
  devInfo: vi.fn(),
  devError: vi.fn(),
}));

import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { devInfo, devError } from "@/lib/devlog";

describe("useAutoUpdate", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("initializes with idle status", () => {
    const { result } = renderHook(() => useAutoUpdate());
    expect(result.current.state).toEqual({ status: "idle" });
  });

  it("auto-checks for update on mount after 10s delay", async () => {
    vi.mocked(check).mockResolvedValue(null);

    renderHook(() => useAutoUpdate());

    expect(check).not.toHaveBeenCalled();

    await act(async () => {
      vi.advanceTimersByTime(10000);
      await vi.runAllTimersAsync();
    });

    expect(check).toHaveBeenCalled();
  });

  it("clears auto-check timer on unmount", () => {
    const clearTimeoutSpy = vi.spyOn(globalThis, "clearTimeout");
    vi.mocked(check).mockResolvedValue(null);

    const { unmount } = renderHook(() => useAutoUpdate());
    unmount();

    expect(clearTimeoutSpy).toHaveBeenCalled();
    clearTimeoutSpy.mockRestore();
  });

  describe("checkForUpdate", () => {
    it("transitions to checking status", async () => {
      vi.mocked(check).mockImplementation(
        () =>
          new Promise((resolve) => {
            setTimeout(() => resolve(null), 100);
          })
      );

      const { result } = renderHook(() => useAutoUpdate());

      act(() => {
        result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({ status: "checking" });
    });

    it("transitions to available when update is found", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "New features",
        available: true,
        downloadAndInstall: vi.fn(),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "available",
        version: "2.0.0",
        body: "New features",
      });
      expect(devInfo).toHaveBeenCalledWith(
        "updater",
        "Update available: 2.0.0"
      );
    });

    it("handles undefined body in available update", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: undefined,
        available: true,
        downloadAndInstall: vi.fn(),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "available",
        version: "2.0.0",
        body: undefined,
      });
    });

    it("transitions to upToDate when no update available", async () => {
      vi.mocked(check).mockResolvedValue(null);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({ status: "upToDate" });
      expect(devInfo).toHaveBeenCalledWith("updater", "App is up to date");
    });

    it("resets to idle after 5s when upToDate", async () => {
      vi.mocked(check).mockResolvedValue(null);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({ status: "upToDate" });

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.state).toEqual({ status: "idle" });
    });

    it("handles check error", async () => {
      const error = new Error("Check failed");
      vi.mocked(check).mockRejectedValue(error);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "error",
        message: "Error: Check failed",
      });
      expect(devError).toHaveBeenCalledWith(
        "updater",
        "Check failed: Error: Check failed"
      );
    });

    it("does not update state if unmounted", async () => {
      const deferred = {
        resolve: null as any,
        promise: null as any,
      };

      deferred.promise = new Promise((resolve) => {
        deferred.resolve = resolve;
      });

      vi.mocked(check).mockReturnValue(deferred.promise);

      const { result, unmount } = renderHook(() => useAutoUpdate());

      act(() => {
        result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({ status: "checking" });

      unmount();

      await act(async () => {
        deferred.resolve(null);
        await deferred.promise;
      });

      // State should not update after unmount
      expect(devInfo).not.toHaveBeenCalled();
    });
  });

  describe("downloadAndInstall", () => {
    it("does nothing if no update is available", async () => {
      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(result.current.state).toEqual({ status: "idle" });
    });

    it("transitions through download states with progress", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(async (callback) => {
          callback({ event: "Started", data: { contentLength: 1000 } });
          callback({
            event: "Progress",
            data: { chunkLength: 100 },
          });
          callback({ event: "Finished" });
        }),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "available",
        version: "2.0.0",
        body: "Update",
      });

      vi.mocked(relaunch).mockResolvedValue(undefined);

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(result.current.state).toEqual({ status: "ready" });
    });

    it("updates progress state during download", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(async (callback) => {
          callback({ event: "Started", data: { contentLength: 1000 } });
          callback({
            event: "Progress",
            data: { chunkLength: 250 },
          });
          callback({
            event: "Progress",
            data: { chunkLength: 250 },
          });
          callback({ event: "Finished" });
        }),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      vi.mocked(relaunch).mockResolvedValue(undefined);

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      // Progress updates accumulate
      const state = result.current.state as any;
      expect(state.status).toBe("ready");
    });

    it("logs download events", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(async (callback) => {
          callback({ event: "Started", data: { contentLength: 5000 } });
          callback({ event: "Finished" });
        }),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      vi.mocked(relaunch).mockResolvedValue(undefined);

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(devInfo).toHaveBeenCalledWith(
        "updater",
        "Download started: 5000 bytes"
      );
      expect(devInfo).toHaveBeenCalledWith("updater", "Download finished");
    });

    it("calls relaunch after successful download", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(async (callback) => {
          callback({ event: "Started", data: { contentLength: 1000 } });
          callback({ event: "Finished" });
        }),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);
      vi.mocked(relaunch).mockResolvedValue(undefined);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(relaunch).toHaveBeenCalled();
      expect(devInfo).toHaveBeenCalledWith(
        "updater",
        "Update installed, relaunching..."
      );
    });

    it("handles download error", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn().mockRejectedValue(
          new Error("Download failed")
        ),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(result.current.state).toEqual({
        status: "error",
        message: "Error: Download failed",
      });
      expect(devError).toHaveBeenCalledWith(
        "updater",
        "Download/install failed: Error: Download failed"
      );
    });

    it("does not update state if unmounted during download", async () => {
      const deferred = {
        resolve: null as any,
        promise: null as any,
      };

      deferred.promise = new Promise((resolve) => {
        deferred.resolve = resolve;
      });

      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn().mockReturnValue(deferred.promise),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result, unmount } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      act(() => {
        result.current.downloadAndInstall();
      });

      expect(result.current.state).toEqual({ status: "downloading", progress: 0 });

      unmount();

      await act(async () => {
        deferred.resolve(undefined);
        await deferred.promise;
      });

      // Should not call relaunch after unmount
      expect(relaunch).not.toHaveBeenCalled();
    });
  });

  describe("dismiss", () => {
    it("resets state to idle", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "available",
        version: "2.0.0",
        body: "Update",
      });

      act(() => {
        result.current.dismiss();
      });

      expect(result.current.state).toEqual({ status: "idle" });
    });

    it("clears update reference", async () => {
      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(),
      };

      vi.mocked(check).mockResolvedValue(mockUpdate as any);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      act(() => {
        result.current.dismiss();
      });

      // Verify that downloadAndInstall does nothing after dismiss
      vi.mocked(mockUpdate.downloadAndInstall).mockClear();

      await act(async () => {
        await result.current.downloadAndInstall();
      });

      expect(mockUpdate.downloadAndInstall).not.toHaveBeenCalled();
    });

    it("can dismiss error state", async () => {
      const error = new Error("Check failed");
      vi.mocked(check).mockRejectedValue(error);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "error",
        message: "Error: Check failed",
      });

      act(() => {
        result.current.dismiss();
      });

      expect(result.current.state).toEqual({ status: "idle" });
    });
  });

  describe("state transitions", () => {
    it("allows re-checking after error", async () => {
      vi.mocked(check)
        .mockRejectedValueOnce(new Error("First attempt failed"))
        .mockResolvedValueOnce(null);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "error",
        message: "Error: First attempt failed",
      });

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({ status: "upToDate" });
    });

    it("allows checking after upToDate reset", async () => {
      vi.mocked(check).mockResolvedValue(null);

      const { result } = renderHook(() => useAutoUpdate());

      await act(async () => {
        await result.current.checkForUpdate();
      });

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.state).toEqual({ status: "idle" });

      const mockUpdate = {
        version: "2.0.0",
        body: "Update",
        available: true,
        downloadAndInstall: vi.fn(),
      };

      vi.mocked(check).mockResolvedValueOnce(mockUpdate as any);

      await act(async () => {
        await result.current.checkForUpdate();
      });

      expect(result.current.state).toEqual({
        status: "available",
        version: "2.0.0",
        body: "Update",
      });
    });
  });
});
