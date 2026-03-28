import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { FlaskConical, CheckCircle2, XCircle, Loader2, Download } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { escapeCSV } from "@/lib/utils";
import { devError } from "@/lib/devlog";
import type { Mode06Result } from "@/stores/vehicle";

interface Mode06Props {
  results: Mode06Result[];
  isLoading: boolean;
}

export default function Mode06({ results, isLoading }: Mode06Props) {
  const { t } = useTranslation();

  const passCount = results.filter(r => r.passed).length;
  const failCount = results.filter(r => !r.passed).length;

  const handleExport = useCallback(async () => {
    const header = [t("mode06.csvTid"), t("mode06.csvMid"), t("mode06.csvName"), t("mode06.csvValue"), t("mode06.csvMin"), t("mode06.csvMax"), t("mode06.csvStatus")].join(",");
    const rows = results.map(r => `${r.tid},${r.mid},${escapeCSV(r.name ?? "")},${r.testValue},${r.minLimit ?? ""},${r.maxLimit ?? ""},${r.passed ? t("mode06.csvPass") : t("mode06.csvFail")}`);
    const csv = [header, ...rows].join("\n");
    try {
      await invoke("save_csv_file", { filename: `bricarobd_mode06_${Date.now()}.csv`, content: csv });
    } catch (e) {
      devError("ui", `${t("mode06.exportError")}: ${e}`);
    }
  }, [results, t]);

  return (
    <div className="space-y-4">
      {/* Header with scan button */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <FlaskConical className="w-5 h-5 text-obd-accent" />
          <div>
            <h3 className="text-sm font-semibold text-obd-text">{t("mode06.title")}</h3>
            <p className="text-xs text-obd-text/50">{t("mode06.subtitle")}</p>
          </div>
        </div>
        <div className="flex items-center gap-2">
          {results.length > 0 && (
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

      {/* Summary badges */}
      {results.length > 0 && (
        <div className="flex gap-3">
          <span className="px-2.5 py-1 text-xs rounded-full bg-obd-success/20 text-obd-success border border-obd-success/20">
            <CheckCircle2 className="w-3 h-3 inline mr-1" />
            {t("mode06.passCount", { count: passCount })}
          </span>
          {failCount > 0 && (
            <span className="px-2.5 py-1 text-xs rounded-full bg-obd-danger/20 text-obd-danger border border-obd-danger/20">
              <XCircle className="w-3 h-3 inline mr-1" />
              {t("mode06.failCount", { count: failCount })}
            </span>
          )}
        </div>
      )}

      {/* Results table or empty state */}
      {results.length === 0 && !isLoading ? (
        <div className="glass-card p-8 text-center">
          <FlaskConical className="w-10 h-10 text-obd-text/20 mx-auto mb-3" />
          <p className="text-sm text-obd-text/50">{t("mode06.noResults")}</p>
        </div>
      ) : results.length > 0 ? (
        <div className="glass-card overflow-hidden">
          <table className="w-full text-xs">
            <thead>
              <tr className="border-b border-obd-border/30">
                <th className="text-left p-2.5 text-obd-text/60 font-medium">{t("mode06.testName")}</th>
                <th className="text-right p-2.5 text-obd-text/60 font-medium">{t("mode06.testValue")}</th>
                <th className="text-right p-2.5 text-obd-text/60 font-medium">{t("mode06.minLimit")}</th>
                <th className="text-right p-2.5 text-obd-text/60 font-medium">{t("mode06.maxLimit")}</th>
                <th className="text-center p-2.5 text-obd-text/60 font-medium">{t("mode06.status")}</th>
              </tr>
            </thead>
            <tbody>
              {results.map((r, i) => (
                <tr key={`${r.tid}-${r.mid}-${i}`} className={`border-b border-obd-border/10 ${!r.passed ? "bg-obd-danger/10" : ""}`}>
                  <td className="p-2.5 text-obd-text">{r.name}</td>
                  <td className="p-2.5 text-right text-obd-text font-mono">{r.testValue.toFixed(1)} <span className="text-obd-text/40">{r.unit}</span></td>
                  <td className="p-2.5 text-right text-obd-text/60 font-mono">{r.minLimit.toFixed(1)}</td>
                  <td className="p-2.5 text-right text-obd-text/60 font-mono">{r.maxLimit.toFixed(1)}</td>
                  <td className="p-2.5 text-center">
                    {r.passed ? (
                      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-obd-success/20 text-obd-success">
                        <CheckCircle2 className="w-3 h-3" />
                        {t("mode06.passed")}
                      </span>
                    ) : (
                      <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-obd-danger/20 text-obd-danger">
                        <XCircle className="w-3 h-3" />
                        {t("mode06.failed")}
                      </span>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}
    </div>
  );
}
