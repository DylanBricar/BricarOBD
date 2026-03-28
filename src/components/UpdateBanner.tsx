import { useTranslation } from "react-i18next";
import { Download, X, RefreshCw, CheckCircle2 } from "lucide-react";
import type { UpdateState } from "@/hooks/useAutoUpdate";

interface UpdateBannerProps {
  state: UpdateState;
  onDownload: () => void;
  onDismiss: () => void;
}

export default function UpdateBanner({ state, onDownload, onDismiss }: UpdateBannerProps) {
  const { t } = useTranslation();

  if (state.status === "idle" || state.status === "checking") return null;

  if (state.status === "upToDate") {
    return (
      <div className="mx-4 mb-2 px-3 py-2 rounded-lg bg-obd-success/10 border border-obd-success/20 flex items-center gap-2 text-xs text-obd-success animate-slide-in">
        <CheckCircle2 size={14} />
        <span>{t("update.upToDate")}</span>
      </div>
    );
  }

  if (state.status === "available") {
    return (
      <div role="status" aria-live="polite" className="mx-4 mb-2 px-3 py-2 rounded-lg bg-obd-accent/10 border border-obd-accent/20 flex items-center gap-2 text-xs animate-slide-in">
        <Download size={14} className="text-obd-accent flex-shrink-0" />
        <span className="flex-1 text-obd-text">
          {t("update.available", { version: state.version })}
        </span>
        <button
          onClick={onDownload}
          className="px-2 py-1 rounded bg-obd-accent/20 text-obd-accent hover:bg-obd-accent/30 transition-colors font-medium"
        >
          {t("update.install")}
        </button>
        <button onClick={onDismiss} aria-label={t("common.close")} className="p-0.5 text-obd-text-muted hover:text-obd-text transition-colors">
          <X size={12} />
        </button>
      </div>
    );
  }

  if (state.status === "downloading") {
    return (
      <div role="status" aria-live="polite" className="mx-4 mb-2 px-3 py-2 rounded-lg bg-obd-accent/10 border border-obd-accent/20 flex items-center gap-2 text-xs animate-slide-in">
        <RefreshCw size={14} className="text-obd-accent animate-spin flex-shrink-0" />
        <span className="text-obd-text">{t("update.downloading")}</span>
      </div>
    );
  }

  if (state.status === "ready") {
    return (
      <div role="status" aria-live="polite" className="mx-4 mb-2 px-3 py-2 rounded-lg bg-obd-success/10 border border-obd-success/20 flex items-center gap-2 text-xs text-obd-success animate-slide-in">
        <CheckCircle2 size={14} />
        <span>{t("update.restarting")}</span>
      </div>
    );
  }

  if (state.status === "error") {
    return (
      <div role="alert" className="mx-4 mb-2 px-3 py-2 rounded-lg bg-obd-danger/10 border border-obd-danger/20 flex items-center gap-2 text-xs animate-slide-in">
        <span className="text-obd-danger flex-1">{t("update.error")}</span>
        <button onClick={onDismiss} aria-label={t("common.close")} className="p-0.5 text-obd-text-muted hover:text-obd-text transition-colors">
          <X size={12} />
        </button>
      </div>
    );
  }

  return null;
}
