import { useState, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import {
  AlertTriangle,
  Trash2,
  Search,
  History,
  Download,
  RefreshCw,
  ExternalLink,
  ChevronRight,
  ShieldAlert,
  Clock,
  Lock,
  Wrench,
  Youtube,
} from "lucide-react";
import type { DtcCode, DtcHistoryEntry, Mode06Result, FreezeFrameData } from "@/stores/vehicle";
import { makeCSVFilename, saveCSVFile } from "@/lib/csv";
import type { VehicleInfo } from "@/stores/connection";
import { cn } from "@/lib/utils";
import { useToast } from "@/hooks/useToast";
import { Toast } from "@/components/Toast";
import Mode06 from "./Mode06";
import FreezeFrame from "./FreezeFrame";

const statusIcon = {
  active: <ShieldAlert size={14} className="text-obd-danger" />,
  pending: <Clock size={14} className="text-obd-warning" />,
  permanent: <Lock size={14} className="text-obd-info" />,
};

const statusBadge = {
  active: "badge-danger",
  pending: "badge-warning",
  permanent: "badge-info",
};

const TAB_OPTIONS = ["dtc", "mode06", "freeze"] as const;

interface DTCProps {
  dtcs: DtcCode[];
  dtcHistory: DtcHistoryEntry[];
  vehicle: VehicleInfo | null;
  onReadAll: () => void;
  onClearAll: () => void;
  isReading?: boolean;
  isClearing?: boolean;
  mode06Results?: Mode06Result[];
  isLoadingMode06?: boolean;
  onLoadMode06?: () => void;
  freezeFrame?: FreezeFrameData[];
  isLoadingFreezeFrame?: boolean;
  onLoadFreezeFrame?: () => void;
}

export default function DTC({
  dtcs,
  dtcHistory,
  vehicle,
  onReadAll,
  onClearAll,
  isReading = false,
  isClearing = false,
  mode06Results = [],
  isLoadingMode06 = false,
  onLoadMode06 = () => {},
  freezeFrame = [],
  isLoadingFreezeFrame = false,
  onLoadFreezeFrame = () => {},
}: DTCProps) {
  const { t, i18n } = useTranslation();
  const [activeTab, setActiveTab] = useState<"dtc" | "mode06" | "freeze">("dtc");
  const [selectedDtc, setSelectedDtc] = useState<string | null>(null);
  const [showConfirm, setShowConfirm] = useState(false);
  const [searchQuery, setSearchQuery] = useState("");
  const { toast, showToast, dismissToast } = useToast();

  const handleExportDtcs = useCallback(async () => {
    if (dtcs.length === 0 && dtcHistory.length === 0) {
      showToast(t("liveData.noExportData"), "error");
      return;
    }
    // Build CSV
    const rows = ["Code,Description,Status,Source"];
    dtcs.forEach((d) => {
      rows.push(`${d.code},"${d.description}",${d.status},${d.source}`);
    });
    if (dtcHistory.length > 0) {
      rows.push("");
      rows.push(`# ${t("dtc.history")}`);
      dtcHistory.forEach((h) => {
        rows.push(`${h.code},"${h.description}",${h.status},${h.source}`);
      });
    }
    const csv = rows.join("\n");
    const filename = makeCSVFilename("bricarobd_dtc");

    try {
      const path = await saveCSVFile(csv, filename);
      showToast(`${t("liveData.exportSuccess")} : ${path}`);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [dtcs, dtcHistory, t, showToast]);

  const filteredDtcs = useMemo(
    () => dtcs.filter((d) =>
      d.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
      d.description.toLowerCase().includes(searchQuery.toLowerCase())
    ),
    [dtcs, searchQuery]
  );

  const filteredHistory = useMemo(() => {
    const activeCodes = new Set(dtcs.map((d) => d.code));
    const uniqueHistory = Array.from(
      new Map(dtcHistory.map((h) => [h.code, h])).values()
    )
      .filter((h) => !activeCodes.has(h.code))
      .sort((a, b) => b.seenAt - a.seenAt);

    return searchQuery
      ? uniqueHistory.filter((h) =>
          h.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
          h.description.toLowerCase().includes(searchQuery.toLowerCase())
        )
      : uniqueHistory;
  }, [dtcs, dtcHistory, searchQuery]);

  const selectedData = useMemo(() => {
    if (!selectedDtc) return null;
    if (selectedDtc.startsWith("hist-")) {
      return dtcHistory.find((h) => h.code === selectedDtc.replace("hist-", ""));
    }
    return filteredDtcs.find((d) => d.code === selectedDtc);
  }, [selectedDtc, dtcHistory, filteredDtcs]);

  const openExternal = useCallback((url: string) => {
    const a = document.createElement("a");
    a.href = url;
    a.target = "_blank";
    a.rel = "noopener noreferrer";
    document.body.appendChild(a);
    a.click();
    // Remove immediately in next microtask to prevent orphaned DOM elements
    requestAnimationFrame(() => {
      if (a.parentNode) a.parentNode.removeChild(a);
    });
  }, []);

  const buildSearchQuery = useCallback((code: string, platform: "google" | "youtube") => {
    const isFr = i18n.language === "fr";
    const parts = [code];

    if (vehicle) {
      if (vehicle.make) parts.push(vehicle.make);
      if (vehicle.model) parts.push(vehicle.model);
    }

    if (isFr) {
      parts.push("diagnostic", "réparation");
    } else {
      parts.push("diagnostic", "repair", "fix");
    }

    const query = encodeURIComponent(parts.join(" "));
    if (platform === "google") {
      return `https://www.google.com/search?q=${query}`;
    }
    return `https://www.youtube.com/results?search_query=${query}`;
  }, [i18n.language, vehicle]);

  return (
    <div className="p-6 space-y-4 animate-slide-in h-full flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-obd-danger/10 border border-obd-danger/20 flex items-center justify-center">
            <AlertTriangle className="text-obd-danger" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("dtc.title")}</h2>
            <p className="text-xs text-obd-text-muted">
              {t("dtc.codeCount", { count: filteredDtcs.length })}
              {filteredHistory.length > 0 && ` · ${filteredHistory.length} ${t("dtc.history").toLowerCase()}`}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          <button onClick={onReadAll} disabled={isReading} className={cn("btn-accent flex items-center gap-1.5 text-xs", isReading && "opacity-60")}>
            <RefreshCw size={14} className={cn(isReading && "animate-spin")} />
            {isReading ? t("dtc.reading") : t("dtc.readAll")}
          </button>
          <button
            onClick={() => setShowConfirm(true)}
            disabled={dtcs.length === 0 || isClearing}
            className={cn("btn-danger flex items-center gap-1.5 text-xs", (dtcs.length === 0 || isClearing) && "opacity-40")}
          >
            <Trash2 size={14} className={cn(isClearing && "animate-spin")} />
            {t("dtc.clearAll")}
          </button>
          <button onClick={handleExportDtcs} className="btn-ghost text-xs flex items-center gap-1.5">
            <Download size={14} />
            {t("dtc.export")}
          </button>
        </div>
      </div>

      {/* Tab bar */}
      <div className="flex gap-1 p-1 rounded-lg bg-white/5 w-fit">
        {TAB_OPTIONS.map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === tab
                ? "bg-obd-accent/20 text-obd-accent"
                : "text-white/50 hover:text-white/80"
            }`}
          >
            {tab === "dtc" ? t("dtc.title") : tab === "mode06" ? t("mode06.tabLabel") : t("freezeFrame.tabLabel")}
          </button>
        ))}
      </div>

      {/* Search — only show for DTC tab */}
      {activeTab === "dtc" && (
        <div className="relative max-w-sm">
          <Search size={14} className="absolute left-3 top-1/2 -translate-y-1/2 text-obd-text-muted" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t("dtc.search")}
            className="input-field pl-9 w-full text-xs"
          />
        </div>
      )}

      {/* Tab content */}
      {activeTab === "dtc" ? (
      <div className="flex flex-col md:flex-row gap-4 flex-1 min-h-0">
        <div className="flex-1 glass-card overflow-hidden flex flex-col">
          {filteredDtcs.length === 0 && filteredHistory.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center text-obd-text-muted">
              <AlertTriangle size={48} strokeWidth={1} className="mb-3 opacity-20" />
              <p className="text-sm">{t("dtc.noCode")}</p>
            </div>
          ) : (
            <div className="flex-1 overflow-y-auto">
              {/* Current DTCs */}
              {filteredDtcs.map((dtc) => (
                <button
                  key={dtc.code}
                  onClick={() => setSelectedDtc(dtc.code === selectedDtc ? null : dtc.code)}
                  className={cn(
                    "data-row w-full text-left group",
                    selectedDtc === dtc.code && "bg-obd-accent/5"
                  )}
                >
                  <div className="flex items-center gap-3 flex-1">
                    {statusIcon[dtc.status]}
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2">
                        <span className="text-sm font-mono font-semibold text-obd-text">{dtc.code}</span>
                        <span className={statusBadge[dtc.status]}>{t(`dtc.${dtc.status}`)}</span>
                      </div>
                      <p className="text-xs text-obd-text-muted truncate mt-0.5">{dtc.description}</p>
                    </div>
                    <span className="text-[10px] text-obd-text-muted">{dtc.source}</span>
                    <ChevronRight size={14} className={cn("text-obd-text-muted transition-transform", selectedDtc === dtc.code && "rotate-90")} />
                  </div>
                </button>
              ))}

              {/* History — always shown, separated by a label */}
              {filteredHistory.length > 0 && (
                <>
                  <div className="px-4 py-2 flex items-center gap-2 border-t border-obd-border/30 bg-obd-surface/30">
                    <History size={12} className="text-obd-warning" />
                    <span className="text-[10px] font-semibold uppercase tracking-wider text-obd-warning">
                      {t("dtc.historyTitle")} ({filteredHistory.length})
                    </span>
                  </div>
                  {filteredHistory.map((h) => (
                    <button
                      key={`hist-${h.code}`}
                      onClick={() => setSelectedDtc(`hist-${h.code}` === selectedDtc ? null : `hist-${h.code}`)}
                      className={cn(
                        "data-row w-full text-left group opacity-70",
                        selectedDtc === `hist-${h.code}` && "bg-obd-warning/5 opacity-100"
                      )}
                    >
                      <div className="flex items-center gap-3 flex-1">
                        <Clock size={14} className="text-obd-warning/60" />
                        <div className="flex-1 min-w-0">
                          <div className="flex items-center gap-2">
                            <span className="text-sm font-mono font-semibold text-obd-text">{h.code}</span>
                            <span className="badge bg-obd-warning/10 text-obd-warning border border-obd-warning/20">
                              {t("dtc.history")}
                            </span>
                          </div>
                          <p className="text-xs text-obd-text-muted truncate mt-0.5">{h.description}</p>
                        </div>
                        <span className="text-[10px] text-obd-text-muted">
                          {new Date(h.seenAt).toLocaleString(i18n.language === "fr" ? "fr-FR" : "en-US", { day: "2-digit", month: "2-digit", year: "numeric", hour: "2-digit", minute: "2-digit" })}
                        </span>
                        <ChevronRight size={14} className="text-obd-text-muted" />
                      </div>
                    </button>
                  ))}
                </>
              )}
            </div>
          )}
        </div>

        {/* Detail panel */}
        {selectedData && (
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
                <h4 className="text-xs font-medium text-white/40 mb-1">{t("dtc.ecu")}</h4>
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

            {!selectedData.causes && !selectedData.quickCheck && !selectedData.repairTips && (
              <p className="text-sm text-obd-text-muted italic">{t("dtc.noTips")}</p>
            )}

            {selectedData.repairTips && !selectedData.quickCheck && (
              <div className="space-y-1">
                <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider flex items-center gap-1.5">
                  <Wrench size={12} />
                  {t("dtc.repairTips")}
                </h4>
                <p className="text-sm text-obd-text leading-relaxed">{selectedData.repairTips}</p>
              </div>
            )}

            <button
              onClick={() => openExternal(buildSearchQuery(selectedData.code, "google"))}
              className="btn-accent w-full flex items-center justify-center gap-1.5 text-xs"
            >
              <ExternalLink size={14} />
              {t("dtc.webSearch")}
            </button>

            <button
              onClick={() => openExternal(buildSearchQuery(selectedData.code, "youtube"))}
              className="w-full px-4 py-2 rounded-lg font-medium text-sm transition-all duration-200 bg-red-600/20 text-red-400 border border-red-500/30 hover:bg-red-600/30 hover:border-red-500/50 active:scale-[0.98] flex items-center justify-center gap-1.5 text-xs"
            >
              <Youtube size={14} />
              {t("dtc.youtubeSearch")}
            </button>
          </div>
        )}
      </div>
      ) : activeTab === "mode06" ? (
        <Mode06 results={mode06Results} isLoading={isLoadingMode06} onLoad={onLoadMode06} />
      ) : (
        <FreezeFrame data={freezeFrame} isLoading={isLoadingFreezeFrame} onLoad={onLoadFreezeFrame} />
      )}

      {/* Confirm Dialog */}
      {showConfirm && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="glass-card p-6 max-w-md w-full mx-4 space-y-4">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-obd-danger/20 flex items-center justify-center">
                <AlertTriangle className="text-obd-danger" size={20} />
              </div>
              <h3 className="text-lg font-semibold">{t("dtc.confirmClear")}</h3>
            </div>
            <p className="text-sm text-obd-text-secondary">{t("dtc.confirmClearMsg")}</p>
            <div className="flex gap-3 justify-end">
              <button onClick={() => setShowConfirm(false)} className="btn-ghost">
                {t("common.cancel")}
              </button>
              <button
                onClick={() => { onClearAll(); setShowConfirm(false); }}
                className="btn-danger"
              >
                {t("common.confirm")}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Toast */}
      {toast && <Toast message={toast.message} type={toast.type} onDismiss={dismissToast} />}
    </div>
  );
}
