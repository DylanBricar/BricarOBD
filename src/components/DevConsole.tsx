import { useState, useEffect, useRef } from "react";
import { invoke } from "@tauri-apps/api/core";
import { X, Copy, Trash2, Pause, Play, Terminal, ChevronDown, ChevronUp, Maximize2, Minimize2 } from "lucide-react";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";

interface LogEntry {
  timestamp: string;
  level: string;
  source: string;
  message: string;
}

const levelColors: Record<string, string> = {
  TX: "text-cyan-400", RX: "text-green-400", INFO: "text-blue-400",
  WARN: "text-amber-400", ERROR: "text-red-400", DEBUG: "text-gray-500",
};

const sourceColors: Record<string, string> = {
  obd: "text-cyan-300", connection: "text-yellow-300", dtc: "text-orange-300",
  ecu: "text-purple-300", db: "text-emerald-300", dashboard: "text-teal-300",
  ui: "text-pink-300", safety: "text-red-300", settings: "text-gray-300",
};

export default function DevConsole({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [paused, setPaused] = useState(false);
  const [filter, setFilter] = useState("");
  const [autoScroll, setAutoScroll] = useState(true);
  const [expanded, setExpanded] = useState(false);
  const scrollRef = useRef<HTMLDivElement>(null);
  const pausedRef = useRef(false);
  const indexRef = useRef(0);

  pausedRef.current = paused;

  // Backend polling — stable
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
    return () => clearInterval(interval);
  }, []);

  // Auto-scroll
  useEffect(() => {
    if (autoScroll && scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [logs.length, autoScroll]);

  const filteredLogs = filter
    ? logs.filter(l =>
        l.message.toLowerCase().includes(filter.toLowerCase()) ||
        l.level.toLowerCase().includes(filter.toLowerCase()) ||
        l.source.toLowerCase().includes(filter.toLowerCase())
      )
    : logs;

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

  const height = expanded ? "h-[80vh]" : "h-[300px]";

  return (
    <div className={cn("fixed bottom-0 left-0 right-0 z-50 flex flex-col border-t border-[#30363D] shadow-2xl", height)}
         style={{ background: "#0D1117ee", backdropFilter: "blur(8px)" }}>
      {/* Toolbar */}
      <div className="flex items-center justify-between px-3 py-1.5 bg-[#161B22] border-b border-[#30363D] flex-shrink-0">
        <div className="flex items-center gap-2">
          <Terminal size={14} className="text-[#58A6FF]" />
          <span className="text-xs font-semibold text-[#58A6FF] font-mono">Dev Console</span>
          <span className="text-[10px] text-[#8B949E] font-mono">{filteredLogs.length}</span>
        </div>
        <div className="flex items-center gap-1.5">
          <input
            type="text"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            placeholder="Filter..."
            className="h-6 px-2 rounded text-[10px] font-mono bg-[#0D1117] border border-[#30363D] text-[#C9D1D9] placeholder-[#484F58] focus:border-[#58A6FF] focus:outline-none w-36"
          />
          <button onClick={() => setPaused(!paused)}
            className={cn("h-6 px-2 rounded text-[10px] font-mono flex items-center gap-1",
              paused ? "bg-[#1F6FEB] text-white" : "bg-[#21262D] text-[#C9D1D9] hover:bg-[#30363D]"
            )}
            aria-label={paused ? "Resume" : "Pause"}>
            {paused ? <Play size={10} /> : <Pause size={10} />}
          </button>
          <button onClick={() => setAutoScroll(!autoScroll)}
            className={cn("h-6 px-2 rounded text-[10px] font-mono flex items-center gap-1",
              autoScroll ? "bg-[#238636] text-white" : "bg-[#21262D] text-[#C9D1D9]"
            )}
            aria-label="Auto-scroll">
            <ChevronDown size={10} />
          </button>
          <button onClick={handleCopy} className="h-6 px-2 rounded text-[10px] font-mono bg-[#238636] text-white hover:bg-[#2EA043] flex items-center gap-1" aria-label="Copy logs">
            <Copy size={10} />
          </button>
          <button onClick={handleClear} className="h-6 px-2 rounded text-[10px] font-mono bg-[#DA3633] text-white hover:bg-[#F85149] flex items-center gap-1" aria-label="Clear logs">
            <Trash2 size={10} />
          </button>
          <button onClick={() => setExpanded(!expanded)} className="h-6 px-2 rounded text-[10px] font-mono bg-[#21262D] text-[#8B949E] hover:bg-[#30363D] flex items-center" aria-label={t("common.toggle")}>
            {expanded ? <Minimize2 size={10} /> : <Maximize2 size={10} />}
          </button>
          <button onClick={onClose} className="h-6 w-6 rounded flex items-center justify-center bg-[#21262D] text-[#8B949E] hover:bg-[#30363D] hover:text-white" aria-label={t("common.close")}>
            <X size={10} />
          </button>
        </div>
      </div>

      {/* Logs */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto px-3 py-1 font-mono text-[11px] leading-[18px]">
        {filteredLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-[#484F58] text-xs">
            Waiting for logs...
          </div>
        ) : (
          filteredLogs.map((log, i) => (
            <div key={i} className="flex gap-1.5 hover:bg-[#161B22] px-1 rounded">
              <span className="text-[#484F58] flex-shrink-0 select-all w-20">{log.timestamp}</span>
              <span className={cn("w-12 flex-shrink-0 font-semibold", levelColors[log.level] || "text-gray-400")}>
                [{log.level}]
              </span>
              <span className={cn("w-20 flex-shrink-0", sourceColors[log.source] || "text-gray-400")}>
                {log.source}
              </span>
              <span className="text-[#C9D1D9] break-all select-all">{log.message}</span>
            </div>
          ))
        )}
      </div>

      {/* Status */}
      <div className="flex items-center justify-between px-3 py-0.5 bg-[#161B22] border-t border-[#30363D] text-[9px] text-[#484F58] font-mono flex-shrink-0">
        <span>{logs.length} total / {filteredLogs.length} shown</span>
        <span>{paused ? "⏸ PAUSED" : "● LIVE"}</span>
      </div>
    </div>
  );
}
