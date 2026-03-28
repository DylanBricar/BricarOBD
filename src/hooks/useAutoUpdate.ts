import { useState, useEffect, useCallback } from "react";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { devInfo, devError } from "@/lib/devlog";

export type UpdateState =
  | { status: "idle" }
  | { status: "checking" }
  | { status: "available"; version: string; body: string | undefined }
  | { status: "downloading"; progress: number }
  | { status: "ready" }
  | { status: "error"; message: string }
  | { status: "upToDate" };

export function useAutoUpdate() {
  const [state, setState] = useState<UpdateState>({ status: "idle" });
  const [update, setUpdate] = useState<Update | null>(null);

  const checkForUpdate = useCallback(async () => {
    setState({ status: "checking" });
    try {
      const result = await check();
      if (result?.available) {
        devInfo("updater", `Update available: ${result.version}`);
        setUpdate(result);
        setState({
          status: "available",
          version: result.version,
          body: result.body ?? undefined,
        });
      } else {
        devInfo("updater", "App is up to date");
        setState({ status: "upToDate" });
        // Reset to idle after 5s
        setTimeout(() => setState({ status: "idle" }), 5000);
      }
    } catch (e) {
      devError("updater", `Check failed: ${e}`);
      setState({ status: "error", message: String(e) });
    }
  }, []);

  const downloadAndInstall = useCallback(async () => {
    if (!update) return;
    try {
      setState({ status: "downloading", progress: 0 });

      await update.downloadAndInstall((event) => {
        if (event.event === "Started" && event.data.contentLength) {
          devInfo("updater", `Download started: ${event.data.contentLength} bytes`);
        } else if (event.event === "Progress") {
          setState((prev) => {
            if (prev.status !== "downloading") return prev;
            const newProgress = prev.progress + (event.data.chunkLength ?? 0);
            return { status: "downloading", progress: newProgress };
          });
        } else if (event.event === "Finished") {
          devInfo("updater", "Download finished");
        }
      });

      setState({ status: "ready" });
      devInfo("updater", "Update installed, relaunching...");
      await relaunch();
    } catch (e) {
      devError("updater", `Download/install failed: ${e}`);
      setState({ status: "error", message: String(e) });
    }
  }, [update]);

  const dismiss = useCallback(() => {
    setState({ status: "idle" });
    setUpdate(null);
  }, []);

  // Check on mount (with 10s delay to not block startup)
  useEffect(() => {
    const timer = setTimeout(checkForUpdate, 10000);
    return () => clearTimeout(timer);
  }, [checkForUpdate]);

  return { state, checkForUpdate, downloadAndInstall, dismiss };
}
