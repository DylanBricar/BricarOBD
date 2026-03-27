import { useMemo } from "react";
import { useTranslation } from "react-i18next";
import { Wifi, WifiOff, Radio, Gauge } from "lucide-react";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";
import { cn } from "@/lib/utils";

interface StatusBarProps {
  status: ConnectionStatus;
  vehicle: VehicleInfo | null;
  isPolling?: boolean;
}

export default function StatusBar({ status, vehicle, isPolling }: StatusBarProps) {
  const { t, i18n } = useTranslation();

  const statusConfig = useMemo(() => ({
    connected: {
      icon: Wifi,
      label: t("status.connected"),
      dotClass: "status-dot-connected",
      textClass: "text-obd-success",
    },
    demo: {
      icon: Radio,
      label: t("status.demo"),
      dotClass: "status-dot-connected",
      textClass: "text-obd-warning",
    },
    connecting: {
      icon: Gauge,
      label: t("status.connecting"),
      dotClass: "status-dot bg-obd-warning animate-pulse",
      textClass: "text-obd-warning",
    },
    disconnected: {
      icon: WifiOff,
      label: t("status.disconnected"),
      dotClass: "status-dot-disconnected",
      textClass: "text-obd-text-muted",
    },
    error: {
      icon: WifiOff,
      label: t("status.error"),
      dotClass: "status-dot-disconnected",
      textClass: "text-obd-danger",
    },
  }), [t, i18n.language]);

  const config = statusConfig[status];
  const Icon = config.icon;

  return (
    <footer className="h-8 flex items-center px-4 border-t border-obd-border/30 bg-obd-surface/50 backdrop-blur-sm text-xs select-none">
      {/* Connection status */}
      <div className="flex items-center gap-2">
        <div className={config.dotClass} />
        <span className={cn("font-medium", config.textClass)}>
          {config.label}
        </span>
      </div>

      {/* Protocol */}
      {vehicle && (
        <>
          <div className="mx-3 h-3 w-px bg-obd-border/50" />
          <div className="flex items-center gap-1.5 text-obd-text-muted">
            <Icon size={12} />
            <span>{vehicle.protocol}</span>
          </div>

          <div className="mx-3 h-3 w-px bg-obd-border/50" />
          <span className="text-obd-text-muted">
            {vehicle.make} {vehicle.model}
          </span>

          {vehicle.elmVersion && (
            <>
              <div className="mx-3 h-3 w-px bg-obd-border/50" />
              <span className="text-obd-text-muted">{vehicle.elmVersion}</span>
            </>
          )}
        </>
      )}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Polling indicator */}
      {isPolling && (
        <div className="flex items-center gap-1.5 text-xs text-obd-success">
          <div className="w-2 h-2 rounded-full bg-obd-success animate-pulse" />
          {t("nav.polling")}
        </div>
      )}

      {/* Right side */}
      <span className="text-obd-text-muted">
        BricarOBD v2.0.4
      </span>
    </footer>
  );
}
