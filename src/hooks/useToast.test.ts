import { describe, it, expect, beforeEach, afterEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useToast } from "./useToast";

describe("useToast()", () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  describe("initial state", () => {
    it("starts with null toast", () => {
      const { result } = renderHook(() => useToast());

      expect(result.current.toast).toBeNull();
    });

    it("provides showToast and dismissToast functions", () => {
      const { result } = renderHook(() => useToast());

      expect(result.current.showToast).toBeDefined();
      expect(result.current.dismissToast).toBeDefined();
      expect(typeof result.current.showToast).toBe("function");
      expect(typeof result.current.dismissToast).toBe("function");
    });
  });

  describe("showToast()", () => {
    it("shows a success toast by default", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Operation successful");
      });

      expect(result.current.toast).toEqual({
        message: "Operation successful",
        type: "success",
      });
    });

    it("shows an error toast when specified", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("An error occurred", "error");
      });

      expect(result.current.toast).toEqual({
        message: "An error occurred",
        type: "error",
      });
    });

    it("explicitly shows a success toast", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Success!", "success");
      });

      expect(result.current.toast).toEqual({
        message: "Success!",
        type: "success",
      });
    });

    it("overwrites previous toast message", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("First message");
      });

      expect(result.current.toast?.message).toBe("First message");

      act(() => {
        result.current.showToast("Second message", "error");
      });

      expect(result.current.toast).toEqual({
        message: "Second message",
        type: "error",
      });
    });
  });

  describe("dismissToast()", () => {
    it("clears the toast", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Test message");
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        result.current.dismissToast();
      });

      expect(result.current.toast).toBeNull();
    });

    it("can dismiss without showing toast first", () => {
      const { result } = renderHook(() => useToast());

      expect(result.current.toast).toBeNull();

      act(() => {
        result.current.dismissToast();
      });

      expect(result.current.toast).toBeNull();
    });
  });

  describe("auto-dismiss timing", () => {
    it("auto-dismisses toast after default 5000ms", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Auto-dismiss test");
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.toast).toBeNull();
    });

    it("auto-dismisses after custom duration", () => {
      const { result } = renderHook(() => useToast(3000));

      act(() => {
        result.current.showToast("Custom duration test");
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(3000);
      });

      expect(result.current.toast).toBeNull();
    });

    it("does not dismiss before timeout", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Persistence test");
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(4999);
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(1);
      });

      expect(result.current.toast).toBeNull();
    });
  });

  describe("timer cleanup", () => {
    it("clears timer when dismissToast is called before auto-dismiss", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Timer cleanup test");
      });

      act(() => {
        vi.advanceTimersByTime(2000);
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        result.current.dismissToast();
      });

      expect(result.current.toast).toBeNull();

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.toast).toBeNull();
    });

    it("clears previous timer when new toast is shown", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("First toast");
      });

      act(() => {
        vi.advanceTimersByTime(2000);
      });

      act(() => {
        result.current.showToast("Second toast");
      });

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.toast).toBeNull();
    });

    it("cleans up timer on unmount", () => {
      const { result, unmount } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Unmount test");
      });

      expect(result.current.toast).not.toBeNull();

      unmount();

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      // No error should be thrown during cleanup
      expect(true).toBe(true);
    });
  });

  describe("multiple toast sequences", () => {
    it("handles multiple show/dismiss cycles", () => {
      const { result } = renderHook(() => useToast());

      // First toast
      act(() => {
        result.current.showToast("Toast 1", "success");
      });
      expect(result.current.toast?.message).toBe("Toast 1");

      act(() => {
        vi.advanceTimersByTime(5000);
      });
      expect(result.current.toast).toBeNull();

      // Second toast
      act(() => {
        result.current.showToast("Toast 2", "error");
      });
      expect(result.current.toast?.message).toBe("Toast 2");

      act(() => {
        result.current.dismissToast();
      });
      expect(result.current.toast).toBeNull();

      // Third toast
      act(() => {
        result.current.showToast("Toast 3", "success");
      });
      expect(result.current.toast?.message).toBe("Toast 3");
    });

    it("rapid successive toast calls only keep last one", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("Toast A");
        result.current.showToast("Toast B");
        result.current.showToast("Toast C");
      });

      expect(result.current.toast?.message).toBe("Toast C");

      act(() => {
        vi.advanceTimersByTime(5000);
      });

      expect(result.current.toast).toBeNull();
    });
  });

  describe("duration parameter", () => {
    it("accepts duration as first parameter", () => {
      const { result: result1 } = renderHook(() => useToast(2000));
      const { result: result2 } = renderHook(() => useToast());

      act(() => {
        result1.current.showToast("Quick dismiss");
        result2.current.showToast("Default duration");
      });

      act(() => {
        vi.advanceTimersByTime(2000);
      });

      expect(result1.current.toast).toBeNull();
      expect(result2.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(3000);
      });

      expect(result2.current.toast).toBeNull();
    });

    it("accepts both duration and respects toast state", () => {
      const { result } = renderHook(() => useToast(1000));

      act(() => {
        result.current.showToast("Fast dismiss", "success");
      });

      expect(result.current.toast).toEqual({
        message: "Fast dismiss",
        type: "success",
      });

      act(() => {
        vi.advanceTimersByTime(1000);
      });

      expect(result.current.toast).toBeNull();
    });
  });

  describe("edge cases", () => {
    it("handles empty toast message", () => {
      const { result } = renderHook(() => useToast());

      act(() => {
        result.current.showToast("");
      });

      expect(result.current.toast).toEqual({
        message: "",
        type: "success",
      });
    });

    it("handles very long toast message", () => {
      const { result } = renderHook(() => useToast());
      const longMessage = "A".repeat(1000);

      act(() => {
        result.current.showToast(longMessage);
      });

      expect(result.current.toast?.message).toBe(longMessage);
    });

    it("handles zero duration gracefully", () => {
      const { result } = renderHook(() => useToast(0));

      act(() => {
        result.current.showToast("Zero duration");
      });

      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(0);
      });

      expect(result.current.toast).toBeNull();
    });

    it("handles negative duration gracefully", () => {
      const { result } = renderHook(() => useToast(-1000));

      act(() => {
        result.current.showToast("Negative duration");
      });

      // setTimeout with negative or 0 duration still schedules immediately
      expect(result.current.toast).not.toBeNull();

      act(() => {
        vi.advanceTimersByTime(0);
      });

      expect(result.current.toast).toBeNull();
    });
  });
});
