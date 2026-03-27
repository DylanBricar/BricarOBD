import { useState, useEffect, useRef, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, Copy, Trash2, Pause, Play, Terminal, ChevronDown, FolderOpen } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useVirtualizer } from "@tanstack/react-virtual";
import { cn } from "@/lib/utils";

interface LogEntry {
  timestamp: string;
  level: string;
  source: string;
  message: string;
}

const levelColors: Record<string, string> = {
  TX: "dark:text-cyan-400 text-cyan-700", RX: "dark:text-green-400 text-green-700", INFO: "dark:text-blue-400 text-blue-700",
  WARN: "dark:text-amber-400 text-amber-700", ERROR: "dark:text-red-400 text-red-700", DEBUG: "dark:text-gray-500 text-gray-700",
};

const sourceColors: Record<string, string> = {
  obd: "dark:text-cyan-300 text-cyan-700", connection: "dark:text-yellow-300 text-yellow-700", dtc: "dark:text-orange-300 text-orange-700",
  ecu: "dark:text-purple-300 text-purple-700", db: "dark:text-emerald-300 text-emerald-700", dashboard: "dark:text-teal-300 text-teal-700",
  ui: "dark:text-pink-300 text-pink-700", safety: "dark:text-red-300 text-red-700", settings: "dark:text-gray-300 text-gray-700",
};

export default function DevConsole({ isStandalone = false, onClose }: { isStandalone?: boolean; onClose: () => void }) {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [paused, setPaused] = useState(false);
  const [filter, setFilter] = useState("");
  const [autoScroll, setAutoScroll] = useState(true);
  const parentRef = useRef<HTMLDivElement>(null);
  const pausedRef = useRef(false);
  const indexRef = useRef(0);
  const wasPausedBeforeHideRef = useRef(false);

  pausedRef.current = paused;

  // Backend polling — pauses on page visibility hidden
  useEffect(() => {
    // Load initial logs
    invoke<LogEntry[]>("get_dev_logs").then(all => {
      indexRef.current = all.length;
      setLogs(all.slice(-5000));
    }).catch(() => {});

    const poll = async () => {
      if (pausedRef.current) return;
      try {
        const newLogs = await invoke<LogEntry[]>("get_dev_logs", { sinceIndex: indexRef.current });
        if (newLogs.length > 0) {
          indexRef.current += newLogs.length;
          setLogs(prev => {
            const lastEntries = prev.slice(-20);
            const filtered = newLogs.filter(nl =>
              !lastEntries.some(e => e.timestamp === nl.timestamp && e.message === nl.message)
            );
            if (filtered.length === 0) return prev;
            return [...prev, ...filtered].slice(-5000);
          });
        }
      } catch {}
    };

    const interval = setInterval(poll, 500);

    // Page Visibility API — pause polling when hidden, resume when visible
    const handleVisibilityChange = () => {
      if (document.hidden) {
        wasPausedBeforeHideRef.current = pausedRef.current;
        setPaused(true);
      } else if (!wasPausedBeforeHideRef.current) {
        setPaused(false);
      }
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    return () => {
      clearInterval(interval);
      document.removeEventListener("visibilitychange", handleVisibilityChange);
    };
  }, []);

  // Memoize filtered logs
  const filteredLogs = useMemo(
    () => filter
      ? logs.filter(l =>
          l.message.toLowerCase().includes(filter.toLowerCase()) ||
          l.level.toLowerCase().includes(filter.toLowerCase()) ||
          l.source.toLowerCase().includes(filter.toLowerCase())
        )
      : logs,
    [logs, filter]
  );

  // Virtualizer setup
  const virtualizer = useVirtualizer({
    count: filteredLogs.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 18,
    overscan: 10,
  });

  // Auto-scroll with virtualizer
  useEffect(() => {
    if (autoScroll && filteredLogs.length > 0) {
      virtualizer.scrollToIndex(filteredLogs.length - 1, { align: "end" });
    }
  }, [filteredLogs.length, autoScroll, virtualizer]);

  const handleCopy = () => {
    navigator.clipboard.writeText(
      filteredLogs.map(l => `${l.timestamp} [${l.level}] ${l.source}: ${l.message}`).join("\n")
    ).catch(() => {});
  };

  const handleClear = async () => {
    setLogs([]);
    indexRef.current = 0;
    try { await invoke("clear_dev_logs"); } catch {}
  };

  const height = isStandalone ? "h-screen" : "h-[300px]";

  return (
    <div className={cn("flex flex-col border-t border-obd-border shadow-2xl", isStandalone ? "w-screen h-screen" : "fixed bottom-0 left-0 right-0 z-50", height)}
         style={{ background: "var(--obd-bg)", backdropFilter: "blur(8px)" }}>
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-obd-surface border-b border-obd-border flex-shrink-0">
        <div className="flex items-center gap-2">
          <Terminal size={14} className="text-obd-accent" />
          <span className="text-xs font-semibold text-obd-accent font-mono">{t("devConsole.title")}</span>
          <span className="text-[10px] text-obd-text-muted font-mono">{filteredLogs.length}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <input
            type="text"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder={t("devConsole.filter")}
            className="h-6 px-2 rounded text-[10px] font-mono bg-obd-bg border border-obd-border text-obd-text placeholder-obd-text-muted focus:border-obd-accent focus:outline-none w-36"
          />
          <button onClick={() => setPaused(!paused)}
            className={cn("h-6 px-2 rounded text-[10px] font-mono flex items-center gap-1",
              paused ? "bg-obd-accent text-white" : "bg-obd-border text-obd-text-muted hover:bg-obd-border/80"
            )}
            aria-label={paused ? t("devConsole.resume") : t("devConsole.pause")}>
            {paused ? <Play size={10} /> : <Pause size={10} />}
          </button>
          <button onClick={() => setAutoScroll(!autoScroll)}
            className={cn("h-6 px-2 rounded text-[10px] font-mono flex items-center gap-1",
              autoScroll ? "bg-obd-success text-white" : "bg-obd-border text-obd-text-muted"
            )}
            aria-label={t("devConsole.autoScroll")}>
            <ChevronDown size={10} />
          </button>
          <button onClick={() => invoke("open_log_folder").catch(() => {})} className="h-6 px-2 rounded text-[10px] font-mono bg-obd-border text-obd-text-muted hover:bg-obd-border/80 flex items-center gap-1" aria-label={t("devConsole.openFolder")}>
            <FolderOpen size={10} />
          </button>
          <button onClick={handleCopy} className="h-6 px-2 rounded text-[10px] font-mono bg-obd-success text-white hover:opacity-80 flex items-center gap-1" aria-label={t("devConsole.copyLogs")}>
            <Copy size={10} />
          </button>
          <button onClick={handleClear} className="h-6 px-2 rounded text-[10px] font-mono bg-obd-danger text-white hover:opacity-80 flex items-center gap-1" aria-label={t("devConsole.clearLogs")}>
            <Trash2 size={10} />
          </button>
          {!isStandalone && (
            <button onClick={onClose} className="h-6 w-6 rounded flex items-center justify-center bg-obd-border text-obd-text-muted hover:bg-obd-border/80 hover:text-obd-text" aria-label={t("common.close")}>
              <X size={10} />
            </button>
          )}
        </div>
      </div>

      {/* Logs */}
      <div ref={parentRef} className="flex-1 overflow-y-auto bg-obd-bg">
        {filteredLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-obd-text-muted text-xs">
            {t("devConsole.waitingForLogs")}
          </div>
        ) : (
          <div style={{ height: `${virtualizer.getTotalSize()}px`, position: "relative" }}>
            {virtualizer.getVirtualItems().map((virtualItem) => {
              const log = filteredLogs[virtualItem.index];
              return (
                <div
                  key={virtualItem.key}
                  className="flex gap-1.5 hover:bg-obd-surface px-4 py-0 font-mono text-[11px] leading-[18px] absolute w-full"
                  style={{ transform: `translateY(${virtualItem.start}px)` }}
                >
                  <span className="text-obd-text-muted flex-shrink-0 select-all w-20">{log.timestamp}</span>
                  <span className={cn("w-12 flex-shrink-0 font-semibold", levelColors[log.level] || "text-obd-text-muted")}>
                    [{log.level}]
                  </span>
                  <span className={cn("w-20 flex-shrink-0", sourceColors[log.source] || "text-obd-text-muted")}>
                    {log.source}
                  </span>
                  <span className="text-obd-text break-all select-all">{log.message}</span>
                </div>
              );
            })}
          </div>
        )}
      </div>

      {/* Status */}
      <div className="flex items-center justify-between px-3 py-0.5 bg-obd-surface border-t border-obd-border text-[9px] text-obd-text-muted font-mono flex-shrink-0">
        <span>{t("devConsole.statusTotal", { total: logs.length, shown: filteredLogs.length })}</span>
        <span>{paused ? t("devConsole.paused") : t("devConsole.live")}</span>
      </div>
    </div>
  );
}
