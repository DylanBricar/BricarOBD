import { useState, useMemo, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Activity, Search, Download, TrendingUp, Circle, Play, Pause, Upload, ListChecks, Check, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import LiveChart from "@/components/charts/LiveChart";
import type { PidValue } from "@/stores/vehicle";
import { cn } from "@/lib/utils";

interface LiveDataProps {
  pidData: Map<number, PidValue>;
  isPolling: boolean;
  onStartPolling: (intervalMs: number) => void;
  onStopPolling: () => void;
  onChangeRefreshRate: (intervalMs: number) => void;
}

const REFRESH_OPTIONS = [
  { label: "500ms", value: 500 },
  { label: "1s", value: 1000 },
  { label: "2s", value: 2000 },
  { label: "5s", value: 5000 },
];

function generateCSV(pidData: Map<number, PidValue>, buffer?: Array<{ timestamp: Date; snapshot: Record<number, number> }>): string {
  const rows: string[] = ["Timestamp,PID,Name,Value,Unit,Min,Max"];

  if (buffer && buffer.length > 0) {
    // Export recording buffer
    buffer.forEach((record) => {
      pidData.forEach((pid) => {
        const value = record.snapshot[pid.pid];
        if (value !== undefined) {
          rows.push(`${record.timestamp.toISOString()},0x${pid.pid.toString(16).toUpperCase().padStart(2, "0")},${pid.name},${value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`);
        }
      });
    });
  } else {
    // Export current snapshot
    const now = new Date().toISOString();
    pidData.forEach((pid) => {
      rows.push(`${now},0x${pid.pid.toString(16).toUpperCase().padStart(2, "0")},${pid.name},${pid.value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`);
    });
  }

  return rows.join("\n");
}

async function saveFile(content: string, filename: string): Promise<string> {
  // Use Tauri backend to write the file to Desktop/BricarOBD_Exports/
  const path = await invoke<string>("save_csv_file", { filename, content });
  return path;
}

function makeFilename(prefix: string): string {
  const now = new Date();
  return `${prefix}_${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}_${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}.csv`;
}

export default function LiveData({ pidData, isPolling, onStartPolling, onStopPolling, onChangeRefreshRate }: LiveDataProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [sortBy, setSortBy] = useState<"name" | "value">("name");
  const [isRecording, setIsRecording] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [refreshRate, setRefreshRate] = useState(500);
  const [timeRange, setTimeRange] = useState<"30s" | "1m" | "5m" | "all">("all");
  const [selectedPids, setSelectedPids] = useState<Set<number>>(new Set()); // empty = all
  const [showPidSelector, setShowPidSelector] = useState(false);
  const recordBufferRef = useRef<Array<{ timestamp: Date; snapshot: Record<number, number> }>>([]);
  const recordingStartRef = useRef<Date | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  // Reset selectedPids when data transitions from empty to populated (reconnect)
  const prevSizeRef = useRef(0);
  useEffect(() => {
    if (pidData.size > 0 && prevSizeRef.current === 0) {
      setSelectedPids(new Set(pidData.keys()));
    }
    prevSizeRef.current = pidData.size;
  }, [pidData.size]);

  // Recording timer with cleanup
  useEffect(() => {
    if (!isRecording) return;
    const interval = setInterval(() => {
      const elapsed = Date.now() - (recordingStartRef.current?.getTime() || Date.now());
      setRecordingDuration(Math.floor(elapsed / 1000));
    }, 200);
    return () => clearInterval(interval);
  }, [isRecording]);

  // Record snapshots when polling
  useEffect(() => {
    if (!isRecording || pidData.size === 0) return;
    const snapshot: Record<number, number> = {};
    pidData.forEach((pid) => { snapshot[pid.pid] = pid.value; });
    recordBufferRef.current.push({ timestamp: new Date(), snapshot });
  }, [pidData, isRecording]);

  const handleTogglePolling = () => {
    if (isPolling) {
      onStopPolling();
    } else {
      onStartPolling(refreshRate);
    }
  };

  const handleRefreshRateChange = (ms: number) => {
    setRefreshRate(ms);
    onChangeRefreshRate(ms);
  };

  const handleStartRecording = () => {
    recordBufferRef.current = [];
    recordingStartRef.current = new Date();
    setRecordingDuration(0);
    setIsRecording(true);
  };

  const [toast, setToast] = useState<{ message: string; type: "success" | "error" } | null>(null);

  const showToast = (message: string, type: "success" | "error" = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 5000);
  };

  const handleStopRecording = async () => {
    setIsRecording(false);
    const snapshots = recordBufferRef.current.length;
    if (snapshots > 0) {
      try {
        const csv = generateCSV(pidData, recordBufferRef.current);
        const filename = makeFilename("bricarobd_recording");
        const path = await saveFile(csv, filename);
        showToast(`${t("liveData.exportSuccess")} : ${path}`);
      } catch (e) {
        showToast(`${t("common.error")}: ${e}`, "error");
      }
    } else {
      showToast(t("liveData.noRecordingData"), "error");
    }
  };

  const handleExportCSV = async () => {
    if (pidData.size === 0) {
      showToast(t("liveData.noExportData"), "error");
      return;
    }
    try {
      const csv = generateCSV(pidData);
      const filename = makeFilename("bricarobd_snapshot");
      const path = await saveFile(csv, filename);
      showToast(`${t("liveData.exportSuccess")} : ${path}`);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  };

  const handleImportCSV = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = new FileReader();
    reader.onload = (e) => {
      const content = e.target?.result as string;
      const lines = content.split("\n").filter(Boolean);
      showToast(t("liveData.importSuccess", { count: lines.length - 1, file: file.name }));
    };
    reader.readAsText(file);
    event.target.value = "";
  };


  const getFilteredHistory = (history: number[]) => {
    const ranges = { "30s": 30, "1m": 60, "5m": 300, all: Infinity };
    return history.slice(-ranges[timeRange]);
  };

  const filteredPids = useMemo(() => {
    let all = Array.from(pidData.values());
    // Filter by selected PIDs (if any are selected)
    if (selectedPids.size > 0) {
      all = all.filter((p) => selectedPids.has(p.pid));
    }
    // Filter by search text
    if (search) {
      all = all.filter((p) => p.name.toLowerCase().includes(search.toLowerCase()));
    }
    return all.sort((a, b) =>
      sortBy === "name" ? a.name.localeCompare(b.name) : b.value - a.value
    );
  }, [pidData, search, sortBy, selectedPids]);

  const selectedData = selectedPid !== null ? pidData.get(selectedPid) : null;

  return (
    <div className="p-6 space-y-4 animate-slide-in h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <Activity className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("liveData.title")}</h2>
            <p className="text-xs text-obd-text-muted">{filteredPids.length} {t("liveData.parameters")}</p>
          </div>
        </div>

        <div className="flex items-center gap-2 h-[34px]">
          {/* Search */}
          <div className="relative">
            <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-obd-text-muted" />
            <input
              type="text"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("liveData.search")}
              className="input-field pl-9 w-48 text-xs h-[34px]"
            />
          </div>

          {/* PID Selector toggle */}
          <button
            onClick={() => setShowPidSelector(!showPidSelector)}
            className={cn(
              "h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 border transition-all",
              showPidSelector
                ? "bg-obd-accent/20 text-obd-accent border-obd-accent/30"
                : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40"
            )}
          >
            <ListChecks size={14} />
            {selectedPids.size}/{pidData.size}
          </button>

          {/* Refresh Rate */}
          <select
            value={refreshRate}
            onChange={(e) => handleRefreshRateChange(Number(e.target.value))}
            className="input-field text-xs h-[34px] w-20"
          >
            {REFRESH_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>

          {/* Start/Pause — AFTER the select */}
          <button
            onClick={handleTogglePolling}
            className={cn(
              "h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-all",
              isPolling
                ? "bg-obd-warning/20 text-obd-warning border border-obd-warning/30"
                : "bg-obd-accent/20 text-obd-accent border border-obd-accent/30"
            )}
          >
            {isPolling ? <Pause size={14} /> : <Play size={14} />}
            {isPolling ? t("liveData.pause") : t("liveData.start")}
          </button>

          {/* Record */}
          <button
            onClick={isRecording ? handleStopRecording : handleStartRecording}
            className={cn(
              "h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-all border",
              isRecording
                ? "bg-red-500/20 text-red-400 border-red-500/30"
                : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40"
            )}
          >
            <Circle size={8} className={cn(isRecording && "fill-red-400")} />
            {isRecording ? `${t("liveData.stop")} (${recordingDuration}s)` : t("liveData.record")}
          </button>

          {/* Export CSV */}
          <button
            onClick={handleExportCSV}
            disabled={pidData.size === 0}
            className={cn("h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 border bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40", pidData.size === 0 && "opacity-40")}
          >
            <Download size={14} />
            CSV
          </button>

          {/* Import CSV */}
          <button
            onClick={() => fileInputRef.current?.click()}
            className="h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 border bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40"
            title="Import CSV"
          >
            <Upload size={14} />
          </button>
          <input ref={fileInputRef} type="file" accept=".csv" onChange={handleImportCSV} className="hidden" />
        </div>
      </div>

      {/* PID Selector Panel */}
      {showPidSelector && pidData.size > 0 && (
        <div className="glass-card p-3 space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-xs font-semibold text-obd-text-secondary">
              {t("liveData.selectParameters")} ({selectedPids.size}/{pidData.size})
            </span>
            <div className="flex gap-1.5">
              <button
                onClick={() => setSelectedPids(new Set(pidData.keys()))}
                className="text-[10px] px-2 py-1 rounded bg-obd-accent/10 text-obd-accent hover:bg-obd-accent/20 transition-colors"
              >
                {t("liveData.selectAll")}
              </button>
              <button
                onClick={() => setSelectedPids(new Set())}
                className="text-[10px] px-2 py-1 rounded bg-obd-border/20 text-obd-text-muted hover:bg-obd-border/40 transition-colors"
              >
                {t("liveData.selectNone")}
              </button>
            </div>
          </div>
          <div className="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto">
            {Array.from(pidData.values())
              .sort((a, b) => a.name.localeCompare(b.name))
              .map((pid) => {
                const isChecked = selectedPids.has(pid.pid);
                return (
                  <button
                    key={pid.pid}
                    onClick={() => {
                      const next = new Set(selectedPids);
                      if (isChecked) next.delete(pid.pid); else next.add(pid.pid);
                      setSelectedPids(next);
                    }}
                    className={cn(
                      "flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11px] font-medium transition-all border",
                      isChecked
                        ? "bg-obd-accent/15 text-obd-accent border-obd-accent/30"
                        : "bg-obd-border/10 text-obd-text-muted border-obd-border/20 opacity-50 hover:opacity-80"
                    )}
                  >
                    <div className={cn(
                      "w-3.5 h-3.5 rounded-sm border flex items-center justify-center transition-colors",
                      isChecked ? "bg-obd-accent border-obd-accent" : "border-obd-border-light"
                    )}>
                      {isChecked && <Check size={10} className="text-obd-bg" />}
                    </div>
                    {pid.name}
                  </button>
                );
              })}
          </div>
        </div>
      )}

      <div className="flex gap-4 flex-1 min-h-0">
        {/* PID Table */}
        <div className="flex-1 glass-card overflow-hidden flex flex-col">
          {/* Table header */}
          <div className="flex items-center px-4 py-2.5 border-b border-obd-border/30 text-[10px] font-semibold uppercase tracking-wider text-obd-text-muted">
            <button onClick={() => setSortBy("name")} className={cn("flex-1", sortBy === "name" && "text-obd-accent")}>
              {t("liveData.parameter")}
            </button>
            <button onClick={() => setSortBy("value")} className={cn("w-24 text-right", sortBy === "value" && "text-obd-accent")}>
              {t("liveData.value")}
            </button>
            <span className="w-16 text-right">{t("liveData.unit")}</span>
            <span className="w-16 text-right">{t("liveData.min")}</span>
            <span className="w-16 text-right">{t("liveData.max")}</span>
            <span className="w-12 text-center">{t("liveData.graph")}</span>
          </div>

          {/* Rows */}
          <div className="flex-1 overflow-y-auto">
            {filteredPids.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12 text-obd-text-muted">
                <Activity size={32} strokeWidth={1} className="mb-2 opacity-20" />
                <p className="text-sm">{isPolling ? t("liveData.search") : t("liveData.noDataYet")}</p>
              </div>
            ) : (
              filteredPids.map((pid) => {
                const range = pid.max - pid.min || 1;
                const percent = ((pid.value - pid.min) / range) * 100;
                return (
                  <button
                    key={pid.pid}
                    onClick={() => setSelectedPid(pid.pid === selectedPid ? null : pid.pid)}
                    className={cn(
                      "data-row w-full text-left",
                      selectedPid === pid.pid && "bg-obd-accent/5 border-l-2 border-l-obd-accent"
                    )}
                  >
                    <div className="flex-1 min-w-0">
                      <span className="text-xs text-obd-text truncate block" title={pid.name}>{pid.name}</span>
                      <div className="mt-1 h-1 w-full max-w-[120px] rounded-full bg-obd-border/30">
                        <div
                          className={cn(
                            "h-full rounded-full transition-all duration-300",
                            percent < 60 ? "bg-obd-accent/60" : percent < 80 ? "bg-amber-500/70" : "bg-red-500/70"
                          )}
                          style={{ width: `${Math.min(100, Math.max(0, percent))}%` }}
                        />
                      </div>
                    </div>
                    <span className="w-24 text-right text-xs font-mono text-obd-text">{pid.value.toFixed(1)}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted">{pid.unit}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted font-mono">{pid.min.toFixed(1)}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted font-mono">{pid.max.toFixed(1)}</span>
                    <div className="w-12 flex justify-center">
                      <TrendingUp size={12} className="text-obd-text-muted" />
                    </div>
                  </button>
                );
              })
            )}
          </div>
        </div>

        {/* Side chart panel */}
        {selectedData && (
          <div className="w-80 space-y-4">
            <div className="flex gap-1.5">
              {(["30s", "1m", "5m", "all"] as const).map((range) => (
                <button
                  key={range}
                  onClick={() => setTimeRange(range)}
                  className={cn(
                    "flex-1 px-2 py-1.5 rounded text-xs font-medium transition-colors",
                    timeRange === range
                      ? "bg-obd-accent text-white"
                      : "bg-obd-border/20 text-obd-text-muted hover:bg-obd-border/40"
                  )}
                >
                  {range === "all" ? t("liveData.all") : range}
                </button>
              ))}
            </div>
            <LiveChart
              data={getFilteredHistory(selectedData.history)}
              label={selectedData.name}
              unit={selectedData.unit}
              color="#06B6D4"
              height={200}
            />
            <div className="glass-card p-4 space-y-2">
              <h4 className="text-xs font-semibold text-obd-text-secondary">{t("liveData.statistics")}</h4>
              <StatRow label={t("liveData.current")} value={selectedData.value.toFixed(2)} unit={selectedData.unit} />
              <StatRow label={t("liveData.minimum")} value={selectedData.min.toFixed(2)} unit={selectedData.unit} />
              <StatRow label={t("liveData.maximum")} value={selectedData.max.toFixed(2)} unit={selectedData.unit} />
              <StatRow
                label={t("liveData.average")}
                value={(selectedData.history.reduce((a, b) => a + b, 0) / selectedData.history.length || 0).toFixed(2)}
                unit={selectedData.unit}
              />
            </div>
          </div>
        )}
      </div>

      {/* Toast notification */}
      {toast && (
        <div className={cn(
          "fixed bottom-4 right-4 max-w-md px-4 py-3 rounded-lg shadow-lg flex items-start gap-3 animate-slide-in z-50",
          toast.type === "success"
            ? "bg-obd-success/90 text-white"
            : "bg-obd-danger/90 text-white"
        )}>
          <p className="text-xs flex-1 leading-relaxed break-all">{toast.message}</p>
          <button onClick={() => setToast(null)} className="flex-shrink-0 hover:opacity-70">
            <X size={14} />
          </button>
        </div>
      )}
    </div>
  );
}

function StatRow({ label, value, unit }: { label: string; value: string; unit: string }) {
  return (
    <div className="flex items-center justify-between py-1.5">
      <span className="text-xs text-obd-text-muted">{label}</span>
      <span className="text-xs font-mono text-obd-text">
        {value} <span className="text-obd-text-muted">{unit}</span>
      </span>
    </div>
  );
}
