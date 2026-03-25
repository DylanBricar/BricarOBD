import { useTranslation } from "react-i18next";
import { MonitorCheck, CheckCircle2, XCircle, MinusCircle } from "lucide-react";
import type { MonitorStatus } from "@/stores/vehicle";
import { cn } from "@/lib/utils";

interface MonitorsProps {
  monitors: MonitorStatus[];
}

export default function Monitors({ monitors }: MonitorsProps) {
  const { t } = useTranslation();

  const completed = monitors.filter((m) => m.available && m.complete).length;
  const incomplete = monitors.filter((m) => m.available && !m.complete).length;
  const notAvailable = monitors.filter((m) => !m.available).length;

  if (monitors.length === 0) {
    return (
      <div className="p-6 animate-slide-in">
        <div className="flex items-center gap-3 mb-6">
          <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
            <MonitorCheck className="text-obd-accent" size={20} />
          </div>
          <div>
            <h2 className="text-lg font-semibold">{t("monitors.title")}</h2>
            <p className="text-xs text-obd-text-muted">{t("monitors.emissionTestStatus")}</p>
          </div>
        </div>
        <div className="glass-card p-8 flex flex-col items-center justify-center gap-3">
          <MonitorCheck size={32} className="text-obd-text-muted opacity-20" />
          <p className="text-obd-text-muted text-sm">{t("dashboard.noData")}</p>
        </div>
      </div>
    );
  }

  return (
    <div className="p-6 space-y-6 animate-slide-in">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
          <MonitorCheck className="text-obd-accent" size={20} />
        </div>
        <div>
          <h2 className="text-lg font-semibold">{t("monitors.title")}</h2>
          <p className="text-xs text-obd-text-muted">{t("monitors.emissionTestStatus")}</p>
        </div>
      </div>

      {/* Summary */}
      <div className="flex gap-4">
        <div className="glass-card px-5 py-3 flex items-center gap-3">
          <CheckCircle2 size={18} className="text-obd-success" />
          <div>
            <p className="text-xs text-obd-text-muted">{t("monitors.complete")}</p>
            <p className="text-lg font-semibold text-obd-success">{completed}</p>
          </div>
        </div>
        <div className="glass-card px-5 py-3 flex items-center gap-3">
          <XCircle size={18} className="text-obd-warning" />
          <div>
            <p className="text-xs text-obd-text-muted">{t("monitors.incomplete")}</p>
            <p className="text-lg font-semibold text-obd-warning">{incomplete}</p>
          </div>
        </div>
        <div className="glass-card px-5 py-3 flex items-center gap-3">
          <MinusCircle size={18} className="text-obd-text-muted" />
          <div>
            <p className="text-xs text-obd-text-muted">{t("monitors.notAvailable")}</p>
            <p className="text-lg font-semibold text-obd-text-muted">{notAvailable}</p>
          </div>
        </div>
      </div>

      {/* Monitor list */}
      <div className="space-y-2">
        {monitors.map((monitor) => (
          <div
            key={monitor.nameKey}
            className={cn(
              "glass-card p-4 space-y-2 transition-all",
              !monitor.available && "opacity-50"
            )}
          >
            <div className="flex items-start gap-3">
              <div className="flex-shrink-0 mt-0.5">
                {!monitor.available ? (
                  <MinusCircle size={20} className="text-obd-text-muted" />
                ) : monitor.complete ? (
                  <CheckCircle2 size={20} className="text-obd-success" />
                ) : (
                  <XCircle size={20} className="text-obd-warning" />
                )}
              </div>
              <div className="flex-1 min-w-0">
                <p className="text-sm font-medium text-obd-text">{t(monitor.nameKey)}</p>
                {monitor.specificationKey && (
                  <p className="text-[10px] text-obd-text-muted font-mono">{t(monitor.specificationKey)}</p>
                )}
                {monitor.descriptionKey && (
                  <p className="text-xs text-obd-text-muted mt-1 leading-relaxed">{t(monitor.descriptionKey)}</p>
                )}
                <p className="text-[10px] text-obd-text-secondary/60 mt-1.5 font-medium">
                  {!monitor.available
                    ? t("monitors.notAvailable")
                    : monitor.complete
                      ? t("monitors.complete")
                      : t("monitors.incomplete")}
                </p>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
