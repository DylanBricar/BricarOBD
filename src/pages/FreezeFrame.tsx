import { useTranslation } from "react-i18next";
import { Camera, RefreshCw, AlertTriangle } from "lucide-react";
import type { FreezeFrameData } from "@/stores/vehicle";

interface FreezeFrameProps {
  data: FreezeFrameData | null;
  isLoading: boolean;
  onLoad: () => void;
}

export default function FreezeFrame({ data, isLoading, onLoad }: FreezeFrameProps) {
  const { t } = useTranslation();

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Camera className="w-5 h-5 text-obd-accent" />
          <div>
            <h3 className="text-sm font-semibold text-white">{t("freezeFrame.title")}</h3>
            <p className="text-xs text-white/50">{t("freezeFrame.subtitle")}</p>
          </div>
        </div>
        <button
          onClick={onLoad}
          disabled={isLoading}
          className="flex items-center gap-2 px-3 py-1.5 text-xs font-medium rounded-lg bg-obd-accent/20 text-obd-accent hover:bg-obd-accent/30 disabled:opacity-50 transition-colors"
        >
          <RefreshCw className={`w-3.5 h-3.5 ${isLoading ? "animate-spin" : ""}`} />
          {isLoading ? t("freezeFrame.loading") : t("freezeFrame.load")}
        </button>
      </div>

      {!data && !isLoading ? (
        <div className="glass-card p-8 text-center">
          <Camera className="w-10 h-10 text-white/20 mx-auto mb-3" />
          <p className="text-sm text-white/50">{t("freezeFrame.noDtcTrigger")}</p>
        </div>
      ) : data ? (
        <div className="space-y-3">
          {/* DTC Badge */}
          <div className="glass-card p-3 flex items-center gap-3">
            <AlertTriangle className="w-4 h-4 text-obd-danger" />
            <span className="text-xs text-white/60">{t("freezeFrame.triggeredBy")}</span>
            <span className="px-2 py-0.5 text-xs font-mono font-bold rounded bg-obd-danger/20 text-obd-danger border border-obd-danger/20">
              {data.dtcCode}
            </span>
          </div>

          {/* PIDs Table */}
          <div className="glass-card overflow-hidden">
            <table className="w-full text-xs">
              <thead>
                <tr className="border-b border-white/10">
                  <th className="text-left p-2.5 text-white/60 font-medium">{t("freezeFrame.parameter")}</th>
                  <th className="text-right p-2.5 text-white/60 font-medium">{t("freezeFrame.value")}</th>
                </tr>
              </thead>
              <tbody>
                {data.pids.map((pid) => (
                  <tr key={pid.pid} className="border-b border-white/5">
                    <td className="p-2.5 text-white">{pid.name}</td>
                    <td className="p-2.5 text-right font-mono text-white">
                      {pid.value.toFixed(pid.unit === "RPM" ? 0 : 1)} <span className="text-white/40">{pid.unit}</span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>

          {/* Explanation */}
          <p className="text-[10px] text-white/30 italic px-1">{t("freezeFrame.capturedAt")}</p>
        </div>
      ) : null}
    </div>
  );
}
