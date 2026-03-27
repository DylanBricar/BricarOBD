import { useState, useEffect, useCallback, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Clock, Download, Trash2, FileText, Calendar, AlertTriangle, Search } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn, escapeCSV } from "@/lib/utils";
import { useToast } from "@/hooks/useToast";
import { Toast } from "@/components/Toast";
import { makeCSVFilename, saveCSVFile } from "@/lib/csv";

interface Session {
  id: string;
  date: string;
  vehicle: string;
  dtcCount: number;
  notes: string;
  dtcCodes?: string[];
}

interface SessionData {
  id: number;
  timestamp: string;
  make: string;
  model: string;
  dtc_count: number;
  notes: string;
}

export default function History() {
  const { t, i18n } = useTranslation();
  const [sessions, setSessions] = useState<Session[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const { toast, showToast, dismissToast } = useToast();
  const [isLoading, setIsLoading] = useState(true);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState("");

  // Load sessions from database on mount
  useEffect(() => {
    let cancelled = false;
    setIsLoading(true);
    invoke<SessionData[]>("get_sessions_cmd")
      .then((data) => {
        if (cancelled) return;
        const mapped = data.map((s) => {
          const matches = s.notes?.match(/\b[PCBU]\d{4,5}\b/gi);
          return {
            id: s.id.toString(),
            date: s.timestamp,
            vehicle: `${s.make} ${s.model}`,
            dtcCount: s.dtc_count,
            notes: s.notes,
            dtcCodes: Array.from(new Set((matches || []).map(c => c.toUpperCase()))),
          };
        });
        setSessions(mapped);
      })
      .catch(() => { if (!cancelled) setSessions([]); })
      .finally(() => { if (!cancelled) setIsLoading(false); });
    return () => { cancelled = true; };
  }, [i18n.language]);

  const selected = sessions.find((s) => s.id === selectedId);

  const filteredSessions = useMemo(() => sessions.filter((s) => {
    const query = searchQuery.toLowerCase();
    return (
      s.vehicle.toLowerCase().includes(query) ||
      s.notes.toLowerCase().includes(query) ||
      (s.dtcCodes || []).some(code => code.toLowerCase().includes(query))
    );
  }), [sessions, searchQuery]);

  const handleExportAll = useCallback(async () => {
    if (sessions.length === 0) {
      showToast(t("liveData.noExportData"), "error");
      return;
    }
    const rows = [[t("history.csvDate"), t("history.csvVehicle"), t("history.csvDtcCount"), t("history.csvDtcCodes"), t("history.csvNotes")].join(",")];
    sessions.forEach((s) => {
      rows.push(`${escapeCSV(s.date)},${escapeCSV(s.vehicle)},${escapeCSV(s.dtcCount)},${escapeCSV((s.dtcCodes || []).join("; "))},${escapeCSV(s.notes)}`);
    });
    const csv = rows.join("\n");
    const filename = makeCSVFilename("bricarobd_sessions");
    try {
      const path = await saveCSVFile(csv, filename);
      showToast(`${t("liveData.exportSuccess")} : ${path}`);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [sessions, t, showToast]);

  const handleExportOne = useCallback(async (session: Session) => {
    const rows = [[t("history.csvDate"), t("history.csvVehicle"), t("history.csvDtcCount"), t("history.csvDtcCodes"), t("history.csvNotes")].join(",")];
    rows.push(`${escapeCSV(session.date)},${escapeCSV(session.vehicle)},${escapeCSV(session.dtcCount)},${escapeCSV((session.dtcCodes || []).join("; "))},${escapeCSV(session.notes)}`);
    const csv = rows.join("\n");
    const filename = makeCSVFilename(`bricarobd_session_${session.id}`);
    try {
      const path = await saveCSVFile(csv, filename);
      showToast(`${t("liveData.exportSuccess")} : ${path}`);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [t, showToast]);

  const handleDelete = async (id: string) => {
    try {
      await invoke("delete_session_cmd", { id: parseInt(id) });
      setSessions((prev) => prev.filter((s) => s.id !== id));
      if (selectedId === id) setSelectedId(null);
      showToast(t("history.deleted"));
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  };

  return (
    <div className="p-6 space-y-4 animate-slide-in h-full flex flex-col">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <Clock className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("history.title")}</h2>
            <p className="text-xs text-obd-text-muted">{t("history.sessionCount", { count: sessions.length })}</p>
          </div>
        </div>
        <button
          onClick={handleExportAll}
          disabled={sessions.length === 0}
          className={cn("btn-accent flex items-center gap-1.5 text-xs", sessions.length === 0 && "opacity-40")}
        >
          <Download size={14} />
          {t("history.export")}
        </button>
      </div>

      <div className="flex gap-4 flex-1 min-h-0">
        {/* Session list */}
        <div className="flex-1 glass-card overflow-hidden flex flex-col">
          {isLoading ? (
            <div className="flex-1 flex flex-col items-center justify-center text-obd-text-muted">
              <Clock size={48} strokeWidth={1} className="mb-3 opacity-20" />
              <p className="text-sm">{t("common.loading")}</p>
            </div>
          ) : sessions.length === 0 ? (
            <div className="flex-1 flex flex-col items-center justify-center text-obd-text-muted">
              <Clock size={48} strokeWidth={1} className="mb-3 opacity-20" />
              <p className="text-sm">{t("history.noSession")}</p>
            </div>
          ) : (
            <>
              <div className="p-3 border-b border-obd-border/20">
                <div className="relative">
                  <Search size={14} className="absolute left-3 top-1/2 transform -translate-y-1/2 text-obd-text-muted" />
                  <input
                    type="text"
                    placeholder={t("common.search")}
                    value={searchQuery}
                    onChange={(e) => setSearchQuery(e.target.value)}
                    className="w-full pl-8 pr-3 py-2 bg-white/[0.05] border border-obd-border/20 rounded-lg text-sm text-obd-text placeholder-obd-text-muted focus:outline-none focus:border-obd-accent/50"
                  />
                </div>
              </div>
              <div className="flex-1 overflow-y-auto">
                {filteredSessions.length === 0 ? (
                  <div className="flex-1 flex flex-col items-center justify-center text-obd-text-muted py-8">
                    <FileText size={32} strokeWidth={1} className="mb-3 opacity-20" />
                    <p className="text-xs">{t("common.noResults")}</p>
                  </div>
                ) : (
                  filteredSessions.map((session) => (
                    <button
                      key={session.id}
                      onClick={() => setSelectedId(session.id === selectedId ? null : session.id)}
                      className={cn(
                        "data-row w-full text-left",
                        selectedId === session.id && "bg-obd-accent/5 border-l-2 border-l-obd-accent"
                      )}
                    >
                      <div className="w-10 h-10 rounded-lg bg-obd-accent/5 flex items-center justify-center mr-3 flex-shrink-0">
                        <FileText size={18} className="text-obd-accent" />
                      </div>
                      <div className="flex-1 min-w-0">
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-medium text-obd-text">{session.vehicle}</span>
                          {session.dtcCount > 0 && (
                            <span className="badge-warning">{session.dtcCount} DTC</span>
                          )}
                        </div>
                        <p className="text-xs text-obd-text-muted truncate mt-0.5">{session.notes}</p>
                      </div>
                      <div className="flex items-center gap-1.5 text-xs text-obd-text-muted">
                        <Calendar size={12} />
                        {new Date(session.date).toLocaleDateString(i18n.language === "fr" ? "fr-FR" : "en-US", { year: "numeric", month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
                      </div>
                    </button>
                  ))
                )}
              </div>
            </>
          )}
        </div>

        {/* Detail panel */}
        {selected && (
          <div className="w-80 glass-card p-5 space-y-4 overflow-y-auto">
            <div>
              <h3 className="font-semibold text-obd-text">{selected.vehicle}</h3>
              <p className="text-xs text-obd-text-muted flex items-center gap-1.5 mt-1">
                <Calendar size={12} /> {new Date(selected.date).toLocaleDateString(i18n.language === "fr" ? "fr-FR" : "en-US", { year: "numeric", month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}
              </p>
            </div>

            {/* DTC Codes */}
            <div className="space-y-1.5">
              <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">
                {t("dtc.title")} ({selected.dtcCount})
              </h4>
              {selected.dtcCodes && selected.dtcCodes.length > 0 ? (
                <div className="space-y-1">
                  {selected.dtcCodes.map((code) => (
                    <div key={code} className="px-3 py-2 rounded-lg bg-white/[0.02] text-xs font-mono text-obd-warning">
                      {code}
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-xs text-obd-success">{t("dtc.noCode")}</p>
              )}
            </div>

            {/* Notes */}
            <div className="space-y-1.5">
              <h4 className="text-xs font-semibold text-obd-text-secondary uppercase tracking-wider">
                {t("history.notes")}
              </h4>
              <p className="text-sm text-obd-text leading-relaxed">{selected.notes}</p>
            </div>

            {/* Actions */}
            <div className="space-y-2 pt-2 border-t border-obd-border/30">
              <button
                onClick={() => handleExportOne(selected)}
                className="btn-accent w-full flex items-center justify-center gap-1.5 text-xs"
              >
                <Download size={14} />
                {t("history.export")}
              </button>
              <button
                onClick={() => setShowDeleteConfirm(selected.id)}
                className="btn-danger w-full flex items-center justify-center gap-1.5 text-xs"
              >
                <Trash2 size={14} />
                {t("history.delete")}
              </button>
            </div>
          </div>
        )}
      </div>

      {/* Delete Confirm Dialog */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/60 backdrop-blur-sm flex items-center justify-center z-50">
          <div className="glass-card p-6 max-w-md w-full mx-4 space-y-4">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-obd-danger/20 flex items-center justify-center">
                <AlertTriangle className="text-obd-danger" size={20} />
              </div>
              <h3 className="text-lg font-semibold">{t("history.confirmDelete")}</h3>
            </div>
            <p className="text-sm text-obd-text-secondary">{t("history.confirmDeleteMsg")}</p>
            <div className="flex gap-3 justify-end">
              <button onClick={() => setShowDeleteConfirm(null)} className="btn-ghost">
                {t("common.cancel")}
              </button>
              <button
                onClick={() => { handleDelete(showDeleteConfirm); setShowDeleteConfirm(null); }}
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
