import { Search, Download, ListChecks, Play, Pause, Circle } from "lucide-react";
import { cn } from "@/lib/utils";

interface LiveDataToolbarProps {
  search: string;
  onSearchChange: (value: string) => void;
  showPidSelector: boolean;
  onTogglePidSelector: () => void;
  selectedPidsCount: number;
  totalPidsCount: number;
  refreshRate: number;
  onRefreshRateChange: (ms: number) => void;
  isActive: boolean;
  onTogglePolling: () => void;
  isRecording: boolean;
  recordingDuration: number;
  onStartRecording: () => void;
  onStopRecording: () => void;
  pidDataSize: number;
  onExportCSV: () => void;
  t: (key: string) => string;
}

const REFRESH_OPTIONS = [
  { label: "500ms", value: 500 },
  { label: "1s", value: 1000 },
  { label: "2s", value: 2000 },
  { label: "5s", value: 5000 },
];

export default function LiveDataToolbar({
  search,
  onSearchChange,
  showPidSelector,
  onTogglePidSelector,
  selectedPidsCount,
  totalPidsCount,
  refreshRate,
  onRefreshRateChange,
  isActive,
  onTogglePolling,
  isRecording,
  recordingDuration,
  onStartRecording,
  onStopRecording,
  pidDataSize,
  onExportCSV,
  t,
}: LiveDataToolbarProps) {
  return (
    <div className="flex flex-wrap items-center gap-2 h-auto md:h-[34px]">
      {/* Search */}
      <div className="relative">
        <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-obd-text-muted" />
        <input
          type="text"
          value={search}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("liveData.search")}
          className="input-field pl-9 w-48 text-xs h-[34px]"
        />
      </div>

      {/* PID Selector toggle */}
      <button
        onClick={onTogglePidSelector}
        className={cn(
          "h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 border transition-all",
          showPidSelector
            ? "bg-obd-accent/20 text-obd-accent border-obd-accent/30"
            : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40"
        )}
      >
        <ListChecks size={14} />
        {selectedPidsCount}/{totalPidsCount}
      </button>

      {/* Refresh Rate */}
      <select
        value={refreshRate}
        onChange={(e) => onRefreshRateChange(Number(e.target.value))}
        className="input-field text-xs h-[34px] w-20"
      >
        {REFRESH_OPTIONS.map((opt) => (
          <option key={opt.value} value={opt.value}>{opt.label}</option>
        ))}
      </select>

      {/* Start/Pause */}
      <button
        onClick={onTogglePolling}
        className={cn(
          "h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 transition-all",
          isActive
            ? "bg-obd-warning/20 text-obd-warning border border-obd-warning/30"
            : "bg-obd-accent/20 text-obd-accent border border-obd-accent/30"
        )}
      >
        {isActive ? <Pause size={14} /> : <Play size={14} />}
        {isActive ? t("liveData.pause") : t("liveData.start")}
      </button>

      {/* Record */}
      <button
        onClick={isRecording ? onStopRecording : onStartRecording}
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
        onClick={onExportCSV}
        disabled={pidDataSize === 0}
        className={cn("h-[34px] px-3 rounded-lg text-xs font-medium flex items-center gap-1.5 border bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40", pidDataSize === 0 && "opacity-40")}
      >
        <Download size={14} />
        CSV
      </button>
    </div>
  );
}
