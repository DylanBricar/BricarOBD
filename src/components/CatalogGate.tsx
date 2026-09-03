import { useCallback, useEffect, useMemo, useState, type ReactNode } from "react";
import { Channel, invoke } from "@tauri-apps/api/core";
import { Database, Download, RefreshCw, ShieldCheck } from "lucide-react";
import { useTranslation } from "react-i18next";
import { formatCatalogSize } from "@/lib/catalog";

interface CatalogStatus {
  ready: boolean;
  downloading: boolean;
  expectedBytes: number;
}

interface CatalogDownloadEvent {
  stage: "started" | "progress" | "finished";
  downloadedBytes: number;
  totalBytes: number;
}

interface CatalogGateProps {
  children: ReactNode;
}

export default function CatalogGate({ children }: CatalogGateProps) {
  const { t } = useTranslation();
  const [status, setStatus] = useState<CatalogStatus | null>(null);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState("");

  const refreshStatus = useCallback(async () => {
    try {
      const current = await invoke<CatalogStatus>("get_catalog_status");
      setStatus(current);
      setError("");
    } catch (cause) {
      setError(String(cause));
    }
  }, []);

  useEffect(() => {
    void refreshStatus();
  }, [refreshStatus]);

  const download = useCallback(async () => {
    setError("");
    setProgress(0);
    setStatus((current) => (current ? { ...current, downloading: true } : current));
    const onEvent = new Channel<CatalogDownloadEvent>();
    onEvent.onmessage = (event) => {
      if (event.totalBytes > 0) {
        setProgress(Math.min(100, Math.round((event.downloadedBytes / event.totalBytes) * 100)));
      }
    };
    try {
      const ready = await invoke<CatalogStatus>("download_catalog", { onEvent });
      setStatus(ready);
      setProgress(100);
    } catch (cause) {
      setStatus((current) => (current ? { ...current, downloading: false } : current));
      setError(String(cause));
    }
  }, []);

  const expectedSize = useMemo(() => formatCatalogSize(status?.expectedBytes ?? 527_691_776), [status?.expectedBytes]);

  if (status?.ready) return children;

  const downloading = status?.downloading ?? false;
  return (
    <div className="flex h-screen w-screen items-center justify-center bg-obd-bg p-6 text-obd-text">
      <section className="w-full max-w-xl rounded-2xl border border-obd-border bg-obd-surface p-8 shadow-2xl">
        <div className="mb-6 flex items-center gap-4">
          <div className="rounded-xl bg-obd-primary/15 p-3 text-obd-primary">
            <Database size={34} aria-hidden="true" />
          </div>
          <div>
            <h1 className="text-2xl font-semibold">{t("catalog.title")}</h1>
            <p className="mt-1 text-sm text-obd-text-muted">{t("catalog.subtitle")}</p>
          </div>
        </div>

        <p className="mb-5 text-sm leading-6 text-obd-text-muted">{t("catalog.description", { size: expectedSize })}</p>

        <div className="mb-6 flex items-start gap-3 rounded-lg border border-obd-border bg-obd-bg/60 p-4 text-sm">
          <ShieldCheck className="mt-0.5 shrink-0 text-obd-success" size={20} aria-hidden="true" />
          <span>{t("catalog.integrity")}</span>
        </div>

        {downloading && (
          <div className="mb-5" role="status" aria-live="polite">
            <div className="mb-2 flex justify-between text-sm">
              <span>{t("catalog.downloading")}</span>
              <span>{progress}%</span>
            </div>
            <div className="h-2 overflow-hidden rounded-full bg-obd-border">
              <div
                className="h-full rounded-full bg-obd-primary transition-[width] duration-200"
                style={{ width: `${progress}%` }}
              />
            </div>
          </div>
        )}

        {error && (
          <div
            className="mb-5 rounded-lg border border-obd-danger/40 bg-obd-danger/10 p-3 text-sm text-obd-danger"
            role="alert"
          >
            {t("catalog.error")}: {error}
          </div>
        )}

        <button
          type="button"
          onClick={() => void download()}
          disabled={downloading || (status === null && !error)}
          className="flex w-full items-center justify-center gap-2 rounded-lg bg-obd-primary px-5 py-3 font-medium text-white transition-opacity hover:opacity-90 disabled:cursor-not-allowed disabled:opacity-50"
        >
          {error ? <RefreshCw size={19} aria-hidden="true" /> : <Download size={19} aria-hidden="true" />}
          {downloading ? t("catalog.downloading") : t(error ? "catalog.retry" : "catalog.download")}
        </button>
        <p className="mt-3 text-center text-xs text-obd-text-muted">{t("catalog.wifiHint")}</p>
      </section>
    </div>
  );
}
