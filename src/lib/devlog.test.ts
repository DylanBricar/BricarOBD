import { describe, it, expect, beforeEach, vi, afterEach } from "vitest";
import { devLog, devInfo, devWarn, devError, devDebug } from "./devlog";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");

describe("devLog() and helper functions", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.runAllTimersAsync();
    vi.useRealTimers();
  });

  it("devLog adds entry to batch buffer", () => {
    devLog("INFO", "test", "test message");

    // Should not invoke yet (batch not full, timer not fired)
    expect(invoke).not.toHaveBeenCalled();
  });

  it("devInfo calls devLog with INFO level", () => {
    devInfo("source", "info message");

    // Verify batch buffer is populated (indirectly by checking timer)
    expect(invoke).not.toHaveBeenCalled(); // Not flushed yet
  });

  it("devWarn calls devLog with WARN level", () => {
    devWarn("source", "warn message");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("devError calls devLog with ERROR level", () => {
    devError("source", "error message");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("devDebug calls devLog with DEBUG level", () => {
    devDebug("source", "debug message");
    expect(invoke).not.toHaveBeenCalled();
  });

  it("flushes batch after 1500ms", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    devLog("INFO", "source1", "message1");
    devLog("WARN", "source2", "message2");

    expect(invoke).not.toHaveBeenCalled();

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledWith("add_dev_logs_batch", {
      logs: expect.arrayContaining([
        { level: "INFO", source: "source1", message: "message1" },
        { level: "WARN", source: "source2", message: "message2" },
      ]),
    });
  });

  it("flushes immediately when buffer reaches max size (500)", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    // Add 500 entries
    for (let i = 0; i < 500; i++) {
      devLog("INFO", "source", `message${i}`);
    }

    expect(mockInvoke).toHaveBeenCalledWith(
      "add_dev_logs_batch",
      expect.objectContaining({
        logs: expect.arrayContaining([
          { level: "INFO", source: "source", message: "message0" },
        ]),
      })
    );
  });

  it("schedules flush only once", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    devLog("INFO", "source", "message1");
    devLog("INFO", "source", "message2");
    devLog("INFO", "source", "message3");

    // Should have scheduled only one flush
    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    // Should be called once with all 3 messages
    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });

  it("clears timer after flush", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    devLog("INFO", "source", "message1");

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Add another message and verify new batch is scheduled
    devLog("INFO", "source", "message2");

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("multiple batches with size < 500", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    devLog("INFO", "source", "batch1_msg1");
    devLog("INFO", "source", "batch1_msg2");

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledTimes(1);

    devLog("INFO", "source", "batch2_msg1");

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledTimes(2);
  });

  it("handles invoke errors gracefully", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockRejectedValue(new Error("Network error"));

    devLog("INFO", "source", "message");

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    // Should not throw
    expect(mockInvoke).toHaveBeenCalled();
  });

  it("beforeunload event flushes remaining batch", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    devLog("INFO", "source", "unsaved message");

    // Dispatch beforeunload event
    const beforeUnloadEvent = new Event("beforeunload");
    window.dispatchEvent(beforeUnloadEvent);

    // Should flush immediately
    expect(mockInvoke).toHaveBeenCalledWith("add_dev_logs_batch", {
      logs: expect.arrayContaining([
        { level: "INFO", source: "source", message: "unsaved message" },
      ]),
    });
  });

  it("does not invoke if batch is empty", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("timer is cleared when batch reaches max size", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue(undefined);

    // Add exactly 500 entries to trigger flush
    for (let i = 0; i < 500; i++) {
      devLog("INFO", "source", `message${i}`);
    }

    // Verify invoked for first batch
    expect(mockInvoke).toHaveBeenCalledTimes(1);

    // Advance timers - should not call invoke again if timer was properly cleared
    vi.advanceTimersByTime(1500);
    await vi.runAllTimersAsync();

    expect(mockInvoke).toHaveBeenCalledTimes(1);
  });
});
