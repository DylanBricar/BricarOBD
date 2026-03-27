import { invoke } from "@tauri-apps/api/core";

interface LogEntry {
  level: "INFO" | "WARN" | "ERROR" | "DEBUG";
  source: string;
  message: string;
}

let batchBuffer: LogEntry[] = [];
let flushTimerId: number | null = null;

const BATCH_FLUSH_INTERVAL = 1500;
const BATCH_MAX_SIZE = 500;

function flushBatch() {
  if (batchBuffer.length === 0) return;

  const logsToSend = [...batchBuffer];
  batchBuffer = [];
  flushTimerId = null;

  invoke("add_dev_logs_batch", { logs: logsToSend }).catch(() => {});
}

function scheduleFlush() {
  if (flushTimerId !== null) return;
  flushTimerId = window.setTimeout(flushBatch, BATCH_FLUSH_INTERVAL);
}

window.addEventListener("beforeunload", () => {
  flushBatch();
});

/**
 * Send a log entry to the Rust dev_log buffer (visible in Dev Console window)
 */
export function devLog(level: "INFO" | "WARN" | "ERROR" | "DEBUG", source: string, message: string) {
  batchBuffer.push({ level, source, message });

  if (batchBuffer.length >= BATCH_MAX_SIZE) {
    if (flushTimerId !== null) {
      clearTimeout(flushTimerId);
    }
    flushBatch();
  } else {
    scheduleFlush();
  }
}

export function devInfo(source: string, message: string) { devLog("INFO", source, message); }
export function devWarn(source: string, message: string) { devLog("WARN", source, message); }
export function devError(source: string, message: string) { devLog("ERROR", source, message); }
export function devDebug(source: string, message: string) { devLog("DEBUG", source, message); }
