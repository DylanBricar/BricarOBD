import { useState, useMemo, useRef, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Activity, TrendingUp } from "lucide-react";
import LiveChart from "@/components/charts/LiveChart";
import { makeCSVFilename, saveCSVFile } from "@/lib/csv";
import type { PidValue } from "@/stores/vehicle";
import { cn, escapeCSV } from "@/lib/utils";
import { useToast } from "@/hooks/useToast";
import { Toast } from "@/components/Toast";
import { convertValue, useUnitSystem } from "@/lib/units";
import LiveDataToolbar from "@/components/livedata/LiveDataToolbar";
import PidSelectorPanel from "@/components/livedata/PidSelectorPanel";

interface LiveDataProps {
  pidData: Map<number, PidValue>;
  isPolling: boolean;
  onStartPolling: (intervalMs: number) => void;
  onPausePolling: () => void;
  onStopPolling: () => void;
  onChangeRefreshRate: (intervalMs: number) => void;
}

const TIME_RANGE_OPTIONS = ["30s", "1m", "5m", "all"] as const;

function getTheoreticalMax(unit: string): number {
  const map: Record<string, number> = {
    "RPM": 8000,
    "km/h": 250,
    "°C": 120,
    "%": 100,
    "bar": 5,
    "V": 14,
    "A": 100,
    "kPa": 200,
    "ms": 50,
    "g": 5,
  };
  return map[unit] || 100;
}

function generateCSV(pidData: Map<number, PidValue>, header: string, buffer?: Array<{ timestamp: Date; snapshot: Record<number, number> }>): string {
  const rows: string[] = [header];

  if (buffer && buffer.length > 0) {
    // Export recording buffer
    buffer.forEach((record) => {
      pidData.forEach((pid) => {
        const value = record.snapshot[pid.pid];
        if (value !== undefined) {
          rows.push(`${record.timestamp.toISOString()},0x${pid.pid.toString(16).toUpperCase().padStart(2, "0")},${escapeCSV(pid.name)},${value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`);
        }
      });
    });
  } else {
    // Export current snapshot
    const now = new Date().toISOString();
    pidData.forEach((pid) => {
      rows.push(`${now},0x${pid.pid.toString(16).toUpperCase().padStart(2, "0")},${escapeCSV(pid.name)},${pid.value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`);
    });
  }

  return rows.join("\n");
}


export default function LiveData({ pidData, isPolling, onStartPolling, onPausePolling, onStopPolling, onChangeRefreshRate }: LiveDataProps) {
  const { t } = useTranslation();
  const [search, setSearch] = useState("");
  const [selectedPid, setSelectedPid] = useState<number | null>(null);
  const [sortBy, setSortBy] = useState<"name" | "value">("name");
  const [isRecording, setIsRecording] = useState(false);
  const [recordingDuration, setRecordingDuration] = useState(0);
  const [refreshRate, setRefreshRate] = useState(500);
  const [timeRange, setTimeRange] = useState<"30s" | "1m" | "5m" | "all">("all");
  const [selectedPids, setSelectedPids] = useState<Set<number>>(() => {
    try {
      const saved = localStorage.getItem("bricarobd_selected_pids");
      if (saved) return new Set(JSON.parse(saved) as number[]);
    } catch {}
    return new Set();
  });
  const [showPidSelector, setShowPidSelector] = useState(false);
  const [isActive, setIsActive] = useState(isPolling);
  const { system: unitSystem } = useUnitSystem();
  const recordBufferRef = useRef<Array<{ timestamp: Date; snapshot: Record<number, number> }>>([]);
  const recordingStartRef = useRef<Date | null>(null);

  // Sync isActive with isPolling
  useEffect(() => {
    setIsActive(isPolling);
  }, [isPolling]);

  // Reset selectedPids when data transitions from empty to populated (reconnect)
  // If user had a saved selection, restore it; otherwise select all
  const prevSizeRef = useRef(0);
  useEffect(() => {
    if (pidData.size > 0 && prevSizeRef.current === 0) {
      try {
        const saved = localStorage.getItem("bricarobd_selected_pids");
        if (saved) {
          const savedSet = new Set(JSON.parse(saved) as number[]);
          // Only restore if at least some saved PIDs are in the new data
          const intersection = new Set([...savedSet].filter(p => pidData.has(p)));
          if (intersection.size > 0) {
            setSelectedPids(intersection);
          } else {
            setSelectedPids(new Set(pidData.keys()));
          }
        } else {
          setSelectedPids(new Set(pidData.keys()));
        }
      } catch {
        setSelectedPids(new Set(pidData.keys()));
      }
    }
    prevSizeRef.current = pidData.size;
  }, [pidData.size]);

  // Persist PID selection to localStorage
  useEffect(() => {
    if (selectedPids.size > 0) {
      try {
        localStorage.setItem("bricarobd_selected_pids", JSON.stringify([...selectedPids]));
      } catch {}
    }
  }, [selectedPids]);

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
    // Cap recording buffer at 36000 entries (~10h at 1/s) to prevent memory leak
    if (recordBufferRef.current.length > 36000) {
      recordBufferRef.current = recordBufferRef.current.slice(-36000);
    }
  }, [pidData, isRecording]);

  const handleTogglePolling = useCallback(() => {
    setIsActive((prev) => {
      if (!prev && !isPolling) {
        onStartPolling(refreshRate);
      } else if (prev) {
        onPausePolling();
      }
      return !prev;
    });
  }, [isPolling, onStartPolling, onPausePolling, refreshRate]);

  const handleRefreshRateChange = useCallback((ms: number) => {
    setRefreshRate(ms);
    if (isActive) {
      onStopPolling();
      onChangeRefreshRate(ms);
    } else {
      onChangeRefreshRate(ms);
    }
  }, [isActive, onChangeRefreshRate, onStopPolling]);

  const handleStartRecording = useCallback(() => {
    recordBufferRef.current = [];
    recordingStartRef.current = new Date();
    setRecordingDuration(0);
    setIsRecording(true);
  }, []);

  const { toast, showToast, dismissToast } = useToast();

  const handleStopRecording = useCallback(async () => {
    setIsRecording(false);
    const snapshots = recordBufferRef.current.length;
    if (snapshots > 0) {
      try {
        const csvHeader = [t("liveData.csvTimestamp"), t("liveData.csvPid"), t("liveData.csvName"), t("liveData.csvValue"), t("liveData.csvUnit"), t("liveData.csvMin"), t("liveData.csvMax")].join(",");
        const csv = generateCSV(pidData, csvHeader, recordBufferRef.current);
        const filename = makeCSVFilename("bricarobd_recording");
        const path = await saveCSVFile(csv, filename);
        showToast(`${t("liveData.exportSuccess")} : ${path}`);
      } catch (e) {
        showToast(`${t("common.error")}: ${e}`, "error");
      }
    } else {
      showToast(t("liveData.noRecordingData"), "error");
    }
  }, [pidData, showToast, t]);

  const handleExportCSV = useCallback(async () => {
    if (pidData.size === 0) {
      showToast(t("liveData.noExportData"), "error");
      return;
    }
    try {
      const csvHeader = [t("liveData.csvTimestamp"), t("liveData.csvPid"), t("liveData.csvName"), t("liveData.csvValue"), t("liveData.csvUnit"), t("liveData.csvMin"), t("liveData.csvMax")].join(",");
      const csv = generateCSV(pidData, csvHeader);
      const filename = makeCSVFilename("bricarobd_snapshot");
      const path = await saveCSVFile(csv, filename);
      showToast(`${t("liveData.exportSuccess")} : ${path}`);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [pidData, showToast, t]);

  const historyRanges = useMemo(() => ({ "30s": 30, "1m": 60, "5m": 300, all: Infinity }), []);

  const getFilteredHistory = useCallback((history: number[]) => {
    return history.slice(-historyRanges[timeRange]);
  }, [historyRanges, timeRange]);

  const sortedPidValues = useMemo(() => Array.from(pidData.values()).sort((a, b) => a.name.localeCompare(b.name)), [pidData]);

  const theoreticalMaxCache = useMemo(() => {
    const cache: Record<number, number> = {};
    for (const pid of Array.from(pidData.values())) {
      cache[pid.pid] = getTheoreticalMax(pid.unit);
    }
    return cache;
  }, [pidData]);

  const filteredPids = useMemo(() => {
    // Only show PIDs if polling is active
    if (!isActive) return [];
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
  }, [pidData, search, sortBy, selectedPids, isActive]);

  const selectedData = selectedPid !== null ? pidData.get(selectedPid) : null;

  return (
    <div className="p-6 space-y-4 animate-slide-in h-full flex flex-col">
      {/* Header */}
      <div className="flex flex-col md:flex-row items-start md:items-center justify-between gap-3 md:gap-0">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <Activity className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("liveData.title")}</h2>
            <p className="text-xs text-obd-text-muted">{filteredPids.length} {t("liveData.parameters")}</p>
          </div>
        </div>

        <LiveDataToolbar
          search={search}
          onSearchChange={setSearch}
          showPidSelector={showPidSelector}
          onTogglePidSelector={useCallback(() => setShowPidSelector(!showPidSelector), [showPidSelector])}
          selectedPidsCount={selectedPids.size}
          totalPidsCount={pidData.size}
          refreshRate={refreshRate}
          onRefreshRateChange={handleRefreshRateChange}
          isActive={isActive}
          onTogglePolling={handleTogglePolling}
          isRecording={isRecording}
          recordingDuration={recordingDuration}
          onStartRecording={handleStartRecording}
          onStopRecording={handleStopRecording}
          pidDataSize={pidData.size}
          onExportCSV={handleExportCSV}
          t={t}
        />
      </div>

      {/* PID Selector Panel */}
      {showPidSelector && pidData.size > 0 && (
        <PidSelectorPanel
          pidData={pidData}
          selectedPids={selectedPids}
          onSelectedPidsChange={setSelectedPids}
          sortedPidValues={sortedPidValues}
          t={t}
        />
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
                <p className="text-sm">{isActive ? t("liveData.search") : t("liveData.noDataYet")}</p>
              </div>
            ) : (
              filteredPids.map((pid) => {
                const theoreticalMax = theoreticalMaxCache[pid.pid];
                const range = Math.max(pid.max - pid.min, theoreticalMax) || 1;
                const displayValue = convertValue(pid.value, pid.unit, unitSystem);
                const displayMin = convertValue(pid.min, pid.unit, unitSystem);
                const displayMax = convertValue(pid.max, pid.unit, unitSystem);
                const percent = ((displayValue.value - displayMin.value) / range) * 100;
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
                    <span className="w-24 text-right text-xs font-mono text-obd-text">{displayValue.value.toFixed(1)}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted">{displayValue.unit}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted font-mono">{displayMin.value.toFixed(1)}</span>
                    <span className="w-16 text-right text-[10px] text-obd-text-muted font-mono">{displayMax.value.toFixed(1)}</span>
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
              {TIME_RANGE_OPTIONS.map((range) => (
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
              data={getFilteredHistory(selectedData.history).map(v => convertValue(v, selectedData.unit, unitSystem).value)}
              label={selectedData.name}
              unit={convertValue(selectedData.value, selectedData.unit, unitSystem).unit}
              color="#06B6D4"
              height={200}
            />
            <div className="glass-card p-4 space-y-2">
              <h4 className="text-xs font-semibold text-obd-text-secondary">{t("liveData.statistics")}</h4>
              {(() => {
                const currVal = convertValue(selectedData.value, selectedData.unit, unitSystem);
                const minVal = convertValue(selectedData.min, selectedData.unit, unitSystem);
                const maxVal = convertValue(selectedData.max, selectedData.unit, unitSystem);
                const avgVal = convertValue(
                  selectedData.history.length > 0 ? selectedData.history.reduce((a, b) => a + b, 0) / selectedData.history.length : 0,
                  selectedData.unit,
                  unitSystem
                );
                return (
                  <>
                    <StatRow label={t("liveData.current")} value={currVal.value.toFixed(2)} unit={currVal.unit} />
                    <StatRow label={t("liveData.minimum")} value={minVal.value.toFixed(2)} unit={minVal.unit} />
                    <StatRow label={t("liveData.maximum")} value={maxVal.value.toFixed(2)} unit={maxVal.unit} />
                    <StatRow label={t("liveData.average")} value={avgVal.value.toFixed(2)} unit={avgVal.unit} />
                  </>
                );
              })()}
            </div>
          </div>
        )}
      </div>

      {/* Toast notification */}
      {toast && <Toast message={toast.message} type={toast.type} onDismiss={dismissToast} />}
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
