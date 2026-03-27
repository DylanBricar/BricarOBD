import { Clock, Car, Trash2, X } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";

interface VinHistoryEntry {
  vin: string;
  make: string;
  model: string;
  year: number;
  lastSeen: number;
}

interface VinHistoryPanelProps {
  vinHistory: VinHistoryEntry[];
  onSelectVin: (vin: string) => void;
  onRemoveFromHistory: (vin: string) => void;
  onCacheCleared?: () => void;
  isClearing: boolean;
  t: (key: string) => string;
  language: string;
}

export default function VinHistoryPanel({
  vinHistory,
  onSelectVin,
  onRemoveFromHistory,
  onCacheCleared,
  isClearing,
  t,
  language,
}: VinHistoryPanelProps) {
  const handleClearCache = async (e: React.MouseEvent, vin: string) => {
    e.stopPropagation();
    try {
      await invoke("clear_vin_cache", { vin });
      onCacheCleared?.();
    } catch (_) {
      // Non-critical
    }
  };

  const handleRemoveEntry = (e: React.MouseEvent, vin: string) => {
    e.stopPropagation();
    onRemoveFromHistory(vin);
  };

  return (
    <div className="glass-card p-5 space-y-3">
      <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider flex items-center gap-2">
        <Clock size={14} />
        {t("connection.vinHistory")}
      </h3>
      <div className="space-y-1.5">
        {vinHistory.map((entry) => (
          <div
            key={entry.vin}
            onClick={() => onSelectVin(entry.vin)}
            className="flex items-center gap-3 px-3 py-2 rounded-lg bg-white/[0.02] hover:bg-white/[0.04] transition-colors group cursor-pointer"
          >
            <Car size={14} className="text-obd-accent flex-shrink-0" />
            <div className="flex-1 min-w-0">
              <p className="text-xs font-medium text-obd-text">
                {entry.make}{entry.model ? ` ${entry.model}` : ""} {entry.year > 0 && <span className="text-obd-text-muted">({entry.year})</span>}
              </p>
              <p className="text-[10px] font-mono text-obd-text-muted truncate">{entry.vin}</p>
            </div>
            <span className="text-[9px] text-obd-text-muted">
              {new Date(entry.lastSeen).toLocaleDateString(language === "fr" ? "fr-FR" : "en-US")}
            </span>
            <button
              onClick={(e) => handleClearCache(e, entry.vin)}
              disabled={isClearing}
              className={cn("p-1 rounded hover:bg-obd-warning/10 text-obd-text-muted hover:text-obd-warning transition-colors opacity-0 group-hover:opacity-100", isClearing && "opacity-50")}
              title={t("connection.clearCache")}
            >
              <Trash2 size={12} />
            </button>
            <button
              onClick={(e) => handleRemoveEntry(e, entry.vin)}
              disabled={isClearing}
              className={cn("p-1 rounded hover:bg-obd-danger/10 text-obd-text-muted hover:text-obd-danger transition-colors opacity-0 group-hover:opacity-100", isClearing && "opacity-50")}
              title={t("connection.removeFromHistory")}
            >
              <X size={12} />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
