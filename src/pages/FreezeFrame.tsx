import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Camera, AlertTriangle, Loader2, Download } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { escapeCSV } from "@/lib/utils";
import type { FreezeFrameData } from "@/stores/vehicle";

interface FreezeFrameProps {
  data: FreezeFrameData[];
  isLoading: boolean;
}

export default function FreezeFrame({ data, isLoading }: FreezeFrameProps) {
  const { t, i18n } = useTranslation();
  const locale = i18n.language === "fr" ? "fr-FR" : "en-US";
  const [selectedFrame, setSelectedFrame] = useState(0);

  // Clamp selectedFrame when data array shrinks (prevents out-of-bounds blank screen)
  useEffect(() => {
    if (selectedFrame >= data.length && data.length > 0) {
      setSelectedFrame(0);
    }
  }, [data.length, selectedFrame]);

  const currentFrame = data[selectedFrame];

  const handleExport = useCallback(async () => {
    const frame = data[selectedFrame];
    if (!frame) return;
    const header = [t("freezeFrame.csvPid"), t("freezeFrame.csvName"), t("freezeFrame.csvValue"), t("freezeFrame.csvUnit")].join(",");
    const rows = frame.pids.map(p => `${p.pid},${escapeCSV(p.name)},${p.value},${escapeCSV(p.unit)}`);
    const csv = [header, ...rows].join("\n");
    try {
      await invoke("save_csv_file", { filename: `bricarobd_freezeframe_frame${selectedFrame}_${Date.now()}.csv`, content: csv });
    } catch (e) {
      console.error(`[BricarOBD] ${t("freezeFrame.exportError")}:`, e);
    }
  }, [data, selectedFrame, t]);

  return (
    <div className="space-y-4">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Camera className="w-5 h-5 text-obd-accent" />
          <div>
            <h3 className="text-sm font-semibold text-obd-text">{t("freezeFrame.title")}</h3>
            <p className="text-xs text-obd-text/50">{t("freezeFrame.subtitle")}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {data.length > 0 && currentFrame?.pids.length > 0 && (
            <button
              onClick={handleExport}
              className="p-1.5 rounded-lg bg-obd-success/10 text-obd-success hover:bg-obd-success/20 transition-colors"
              title={t("common.export")}
            >
              <Download className="w-4 h-4" />
            </button>
          )}
          {isLoading && <Loader2 className="w-5 h-5 text-obd-accent animate-spin" />}
        </div>
      </div>

      {data.length === 0 && !isLoading ? (
        <div className="glass-card p-8 text-center">
          <Camera className="w-10 h-10 text-obd-text/20 mx-auto mb-3" />
          <p className="text-sm text-obd-text/50">{t("freezeFrame.noDtcTrigger")}</p>
        </div>
      ) : data.length > 0 && currentFrame ? (
        <div className="space-y-3">
          {/* Frame Selector (tabs) - only show if multiple frames */}
          {data.length > 1 && (
            <div className="flex gap-2 border-b border-obd-border/30">
              {data.map((_, idx) => (
                <button
                  key={idx}
                  onClick={() => setSelectedFrame(idx)}
                  className={`px-3 py-2 text-xs font-medium border-b-2 transition-colors ${
                    selectedFrame === idx
                      ? "border-obd-accent text-obd-accent"
                      : "border-transparent text-obd-text/50 hover:text-obd-text"
                  }`}
                >
                  {t("freezeFrame.frame", { n: idx + 1 })}
                </button>
              ))}
            </div>
          )}

          {/* DTC Badge */}
          <div className="glass-card p-3 flex items-center gap-3">
            <AlertTriangle className="w-4 h-4 text-obd-danger" />
            <span className="text-xs text-obd-text/60">{t("freezeFrame.triggeredBy")}</span>
            <span className="px-2 py-0.5 text-xs font-mono font-bold rounded bg-obd-danger/20 text-obd-danger border border-obd-danger/20">
              {currentFrame.dtcCode}
            </span>
          </div>

          {/* PIDs Table */}
          {currentFrame.pids.length > 0 ? (
            <div className="glass-card overflow-hidden">
              <table className="w-full text-xs">
                <thead>
                  <tr className="border-b border-obd-border/30">
                    <th className="text-left p-2.5 text-obd-text/60 font-medium">{t("freezeFrame.parameter")}</th>
                    <th className="text-right p-2.5 text-obd-text/60 font-medium">{t("freezeFrame.value")}</th>
                  </tr>
                </thead>
                <tbody>
                  {currentFrame.pids.map((pid) => (
                    <tr key={pid.pid} className="border-b border-obd-border/10">
                      <td className="p-2.5 text-obd-text">{pid.name}</td>
                      <td className="p-2.5 text-right font-mono text-obd-text">
                        {pid.value.toLocaleString(locale, { minimumFractionDigits: pid.unit === "RPM" ? 0 : 1, maximumFractionDigits: pid.unit === "RPM" ? 0 : 1 })} <span className="text-obd-text/40">{pid.unit}</span>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          ) : (
            <div className="glass-card p-4 text-center">
              <p className="text-xs text-obd-text/50">{t("freezeFrame.noFrameData")}</p>
            </div>
          )}

          {/* Explanation */}
          <p className="text-[10px] text-obd-text/30 italic px-1">{t("freezeFrame.capturedAt")}</p>
        </div>
      ) : null}
    </div>
  );
}
