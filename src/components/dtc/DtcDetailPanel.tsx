import { AlertTriangle, Wrench, ExternalLink, Youtube } from "lucide-react";
import type { DtcCode, DtcHistoryEntry } from "@/stores/vehicle";
import type { VehicleInfo } from "@/stores/connection";
import { cn } from "@/lib/utils";

interface DtcDetailPanelProps {
  selectedData: DtcCode | DtcHistoryEntry | null;
  vehicle: VehicleInfo | null;
  t: (key: string) => string;
  i18n: { language: string };
  onOpenExternal: (url: string) => void;
  onBuildSearchQuery: (code: string, platform: "google" | "youtube") => string;
}

const statusBadge = {
  active: "badge-danger",
  pending: "badge-warning",
  permanent: "badge-info",
};

export default function DtcDetailPanel({
  selectedData,
  t,
  onOpenExternal,
  onBuildSearchQuery,
}: Omit<DtcDetailPanelProps, 'vehicle' | 'i18n'>) {
  if (!selectedData) return null;

  return (
    <div className="w-full md:w-80 glass-card p-5 space-y-4 overflow-y-auto">
      <div>
        <span className="text-2xl font-mono font-bold text-obd-text">{selectedData.code}</span>
        <span className={cn("ml-2", statusBadge[selectedData.status])}>
          {t(`dtc.${selectedData.status}`)}
        </span>
        {selectedData.difficulty && (
          <span className={`ml-2 px-2 py-0.5 rounded-full text-xs font-medium ${
            selectedData.difficulty === 1 ? "bg-obd-success/20 text-obd-success" :
            selectedData.difficulty === 2 ? "bg-yellow-500/20 text-yellow-400" :
            selectedData.difficulty === 3 ? "bg-orange-500/20 text-orange-400" :
            "bg-obd-danger/20 text-obd-danger"
          }`}>
            {t(`dtc.difficulty${selectedData.difficulty}`)}
          </span>
        )}
      </div>

      <div className="space-y-1">
        <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">
          {t("dtc.description")}
        </h4>
        <p className="text-sm text-obd-text leading-relaxed">{selectedData.description}</p>
      </div>

      <div className="space-y-1">
        <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">
          {t("dtc.source")}
        </h4>
        <p className="text-sm text-obd-text">{selectedData.source}</p>
      </div>

      {selectedData.ecuContext && (
        <div className="space-y-1">
          <h4 className="text-xs font-medium text-obd-text-muted mb-1">{t("dtc.ecu")}</h4>
          <p className="text-sm text-obd-accent">{selectedData.ecuContext}</p>
        </div>
      )}

      {selectedData.causes && selectedData.causes.length > 0 && (
        <div>
          <h4 className="text-xs font-semibold text-obd-text-muted mb-2 flex items-center gap-1.5">
            <AlertTriangle className="w-3.5 h-3.5" />
            {t("dtc.causes")}
          </h4>
          <ul className="space-y-1">
            {selectedData.causes.map((cause, i) => (
              <li key={i} className="text-sm text-obd-text flex items-start gap-2">
                <span className="text-obd-accent mt-1">•</span>
                {cause}
              </li>
            ))}
          </ul>
        </div>
      )}

      {selectedData.quickCheck && (
        <div className="bg-obd-accent/10 border border-obd-accent/20 rounded-lg p-3">
          <h4 className="text-xs font-semibold text-obd-accent mb-1 flex items-center gap-1.5">
            <Wrench className="w-3.5 h-3.5" />
            {t("dtc.quickCheck")}
          </h4>
          <p className="text-sm text-obd-text">{selectedData.quickCheck}</p>
        </div>
      )}

      {selectedData.repairTips && (
        <div className="space-y-1">
          <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider flex items-center gap-1.5">
            <Wrench size={12} />
            {t("dtc.repairTips")}
          </h4>
          <p className="text-sm text-obd-text leading-relaxed">{selectedData.repairTips}</p>
        </div>
      )}

      {!selectedData.causes && !selectedData.quickCheck && !selectedData.repairTips && (
        <p className="text-sm text-obd-text-muted italic">{t("dtc.noTips")}</p>
      )}

      <button
        onClick={() => onOpenExternal(onBuildSearchQuery(selectedData.code, "google"))}
        className="btn-accent w-full flex items-center justify-center gap-1.5 text-xs"
      >
        <ExternalLink size={14} />
        {t("dtc.webSearch")}
      </button>

      <button
        onClick={() => onOpenExternal(onBuildSearchQuery(selectedData.code, "youtube"))}
        className="w-full px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 bg-red-600/20 text-red-400 border border-red-500/30 hover:bg-red-600/30 hover:border-red-500/50 active:scale-[0.98] flex items-center justify-center gap-1.5 text-xs"
      >
        <Youtube size={14} />
        {t("dtc.youtubeSearch")}
      </button>
    </div>
  );
}
