import { useState, useCallback, useMemo, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { AlertTriangle, Trash2, Search, History, Download, RefreshCw, ChevronRight, ShieldAlert, Clock, Lock } from "lucide-react";
import type { DtcCode, DtcHistoryEntry, Mode06Result, FreezeFrameData } from "@/stores/vehicle";
import { makeCSVFilename, saveCSVFile } from "@/lib/csv";
import type { VehicleInfo } from "@/stores/connection";
import { cn, escapeCSV } from "@/lib/utils";
import { useToast } from "@/hooks/useToast";
import { Toast } from "@/components/Toast";
import Mode06 from "./Mode06";
import FreezeFrame from "./FreezeFrame";
import DtcDetailPanel from "@/components/dtc/DtcDetailPanel";
import DtcClearConfirm from "@/components/dtc/DtcClearConfirm";

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
const STATUS_FILTERS = ["all", "active", "pending", "permanent"] as const;

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
  const [statusFilter, setStatusFilter] = useState<"all" | "active" | "pending" | "permanent">("all");

  const { toast, showToast, dismissToast } = useToast();

  useEffect(() => {
    if (!showConfirm) return;
    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === "Escape") setShowConfirm(false);
    };
    document.addEventListener("keydown", handleEscape);
    return () => document.removeEventListener("keydown", handleEscape);
  }, [showConfirm]);

  const handleExportDtcs = useCallback(async () => {
    if (dtcs.length === 0 && dtcHistory.length === 0) {
      showToast(t("liveData.noExportData"), "error");
      return;
    }
    // Build CSV
    const rows = [[t("dtc.csvCode"), t("dtc.csvDescription"), t("dtc.csvStatus"), t("dtc.csvSource")].join(",")];
    dtcs.forEach((d) => {
      rows.push(`${d.code},${escapeCSV(d.description)},${d.status},${d.source}`);
    });
    if (dtcHistory.length > 0) {
      rows.push("");
      rows.push(`# ${t("dtc.history")}`);
      dtcHistory.forEach((h) => {
        rows.push(`${h.code},${escapeCSV(h.description)},${h.status},${h.source}`);
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
    () => dtcs.filter((d) => {
      const matchesSearch = d.code.toLowerCase().includes(searchQuery.toLowerCase()) ||
        d.description.toLowerCase().includes(searchQuery.toLowerCase());
      const matchesStatus = statusFilter === "all" || d.status === statusFilter;
      return matchesSearch && matchesStatus;
    }),
    [dtcs, searchQuery, statusFilter]
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
      const data = dtcHistory.find((h) => h.code === selectedDtc.replace("hist-", ""));
      return data || null;
    }
    const data = filteredDtcs.find((d) => d.code === selectedDtc);
    return data || null;
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
          <button
            onClick={() => {
              if (activeTab === "dtc") {
                onReadAll();
              } else if (activeTab === "mode06") {
                onLoadMode06?.();
              } else if (activeTab === "freeze") {
                onLoadFreezeFrame?.();
              }
            }}
            disabled={isReading || isLoadingMode06 || isLoadingFreezeFrame}
            className={cn("btn-accent flex items-center gap-1.5 text-xs", (isReading || isLoadingMode06 || isLoadingFreezeFrame) && "opacity-60")}
          >
            <RefreshCw size={14} className={cn((isReading || isLoadingMode06 || isLoadingFreezeFrame) && "animate-spin")} />
            {activeTab === "dtc" ? (isReading ? t("dtc.reading") : t("dtc.readAll")) :
             activeTab === "mode06" ? (isLoadingMode06 ? t("mode06.scanning") : t("mode06.scan")) :
             (isLoadingFreezeFrame ? t("freezeFrame.loading") : t("freezeFrame.load"))}
          </button>
          {activeTab === "dtc" && (
            <>
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
            </>
          )}
        </div>
      </div>

      {/* Tab bar */}
      <div className="flex gap-1 p-1 rounded-lg bg-obd-surface/30 w-fit">
        {TAB_OPTIONS.map(tab => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors ${
              activeTab === tab
                ? "bg-obd-accent/20 text-obd-accent"
                : "text-obd-text-muted hover:text-obd-text"
            }`}
          >
            {tab === "dtc" ? t("dtc.title") : tab === "mode06" ? t("mode06.tabLabel") : t("freezeFrame.tabLabel")}
          </button>
        ))}
      </div>

      {/* Search — only show for DTC tab */}
      {activeTab === "dtc" && (
        <div className="space-y-3">
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

          {/* Status filters */}
          <div className="flex gap-1.5">
            {STATUS_FILTERS.map((filter) => (
              <button
                key={filter}
                onClick={() => setStatusFilter(filter)}
                className={cn(
                  "px-2.5 py-1 text-xs font-medium rounded-md transition-colors",
                  statusFilter === filter
                    ? filter === "all" ? "bg-obd-accent/20 text-obd-accent"
                      : filter === "active" ? "bg-obd-danger/20 text-obd-danger"
                      : filter === "pending" ? "bg-obd-warning/20 text-obd-warning"
                      : "bg-obd-info/20 text-obd-info"
                    : "text-obd-text-muted hover:text-obd-text hover:bg-obd-surface/50"
                )}
              >
                {t(`dtc.filter${filter.charAt(0).toUpperCase() + filter.slice(1)}`)}
              </button>
            ))}
          </div>
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
        <DtcDetailPanel
          selectedData={selectedData}
          t={t}
          onOpenExternal={openExternal}
          onBuildSearchQuery={buildSearchQuery}
        />
      </div>
      ) : activeTab === "mode06" ? (
        <Mode06 results={mode06Results} isLoading={isLoadingMode06} />
      ) : (
        <FreezeFrame data={freezeFrame} isLoading={isLoadingFreezeFrame} />
      )}

      {/* Confirm Dialog */}
      {showConfirm && (
        <DtcClearConfirm
          t={t}
          onConfirm={() => { onClearAll(); setShowConfirm(false); }}
          onCancel={() => setShowConfirm(false)}
        />
      )}

      {/* Toast */}
      {toast && <Toast message={toast.message} type={toast.type} onDismiss={dismissToast} />}
    </div>
  );
}
