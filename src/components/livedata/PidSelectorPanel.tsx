import { useCallback } from "react";
import { Check } from "lucide-react";
import type { PidValue } from "@/stores/vehicle";
import { cn } from "@/lib/utils";

interface PidSelectorPanelProps {
  pidData: Map<number, PidValue>;
  selectedPids: Set<number>;
  onSelectedPidsChange: (selected: Set<number>) => void;
  sortedPidValues: PidValue[];
  t: (key: string) => string;
}

export default function PidSelectorPanel({
  pidData,
  selectedPids,
  onSelectedPidsChange,
  sortedPidValues,
  t,
}: PidSelectorPanelProps) {
  const handleSelectAll = useCallback(() => {
    onSelectedPidsChange(new Set(pidData.keys()));
  }, [pidData, onSelectedPidsChange]);

  const handleSelectNone = useCallback(() => {
    onSelectedPidsChange(new Set());
  }, [onSelectedPidsChange]);

  const handleTogglePid = useCallback((pidId: number) => {
    const next = new Set(selectedPids);
    if (next.has(pidId)) {
      next.delete(pidId);
    } else {
      next.add(pidId);
    }
    onSelectedPidsChange(next);
  }, [selectedPids, onSelectedPidsChange]);

  return (
    <div className="glass-card p-3 space-y-2">
      <div className="flex items-center justify-between">
        <span className="text-xs font-semibold text-obd-text-secondary">
          {t("liveData.selectParameters")} ({selectedPids.size}/{pidData.size})
        </span>
        <div className="flex gap-1.5">
          <button
            onClick={handleSelectAll}
            className="text-[10px] px-2 py-1 rounded bg-obd-accent/10 text-obd-accent hover:bg-obd-accent/20 transition-colors"
          >
            {t("liveData.selectAll")}
          </button>
          <button
            onClick={handleSelectNone}
            className="text-[10px] px-2 py-1 rounded bg-obd-border/20 text-obd-text-muted hover:bg-obd-border/40 transition-colors"
          >
            {t("liveData.selectNone")}
          </button>
        </div>
      </div>
      <div className="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto">
        {sortedPidValues.map((pid) => {
          const isChecked = selectedPids.has(pid.pid);
          return (
            <button
              key={pid.pid}
              onClick={() => handleTogglePid(pid.pid)}
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
  );
}
