import { invoke } from "@tauri-apps/api/core";

/**
 * Send a log entry to the Rust dev_log buffer (visible in Dev Console window)
 */
export function devLog(level: "INFO" | "WARN" | "ERROR" | "DEBUG", source: string, message: string) {
  invoke("add_dev_log", { level, source, message }).catch(() => {});
}

export function devInfo(source: string, message: string) { devLog("INFO", source, message); }
export function devWarn(source: string, message: string) { devLog("WARN", source, message); }
export function devError(source: string, message: string) { devLog("ERROR", source, message); }
export function devDebug(source: string, message: string) { devLog("DEBUG", source, message); }
