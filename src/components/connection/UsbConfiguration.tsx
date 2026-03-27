import { RefreshCw, ChevronDown, Wifi } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/stores/connection";

const baudRates = [9600, 38400, 115200, 230400, 500000];

interface UsbConfigurationProps {
  port: string;
  baudRate: number;
  isConnected: boolean;
  status: ConnectionStatus;
  availablePorts: string[];
  onPortChange: (port: string) => void;
  onBaudRateChange: (baud: number) => void;
  onScanPorts: () => void;
  onConnect: () => void;
  onDisconnect: () => void;
  t: (key: string) => string;
}

export default function UsbConfiguration({
  port,
  baudRate,
  isConnected,
  status,
  availablePorts,
  onPortChange,
  onBaudRateChange,
  onScanPorts,
  onConnect,
  onDisconnect,
  t,
}: UsbConfigurationProps) {
  return (
    <>
      <div className="space-y-1.5">
        <div className="flex items-center justify-between">
          <label className="text-xs text-obd-text-muted">{t("connection.port")}</label>
          <button onClick={onScanPorts} className="text-[10px] px-2 py-1 rounded bg-obd-border/20 text-obd-text-muted hover:bg-obd-border/40 flex items-center gap-1">
            <RefreshCw size={12} />
            {t("connection.refreshPorts")}
          </button>
        </div>
        <div className="relative">
          <select
            value={port}
            onChange={(e) => onPortChange(e.target.value)}
            disabled={isConnected}
            className="input-field appearance-none pr-8"
          >
            <option value="">{t("connection.selectPort")}</option>
            {availablePorts.map((p) => (
              <option key={p} value={p}>{p}</option>
            ))}
            {availablePorts.length === 0 && (
              <option disabled>{t("connection.noPort")}</option>
            )}
          </select>
          <ChevronDown
            size={14}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-obd-text-muted pointer-events-none"
          />
        </div>
      </div>

      <div className="space-y-1.5">
        <label className="text-xs text-obd-text-muted">{t("connection.baudRate")}</label>
        <div className="relative">
          <select
            value={baudRate}
            onChange={(e) => onBaudRateChange(Number(e.target.value))}
            disabled={isConnected}
            className="input-field appearance-none pr-8"
          >
            {baudRates.map((br) => (
              <option key={br} value={br}>{br.toLocaleString()}</option>
            ))}
          </select>
          <ChevronDown
            size={14}
            className="absolute right-3 top-1/2 -translate-y-1/2 text-obd-text-muted pointer-events-none"
          />
        </div>
      </div>

      <div className="flex gap-3 pt-2">
        {!isConnected ? (
          <button
            onClick={onConnect}
            disabled={!port || status === "connecting"}
            className={cn(
              "btn-accent-solid flex-1 flex items-center justify-center gap-2",
              (!port || status === "connecting") && "opacity-50 cursor-not-allowed"
            )}
          >
            {status === "connecting" ? (
              <RefreshCw size={16} className="animate-spin" />
            ) : (
              <Wifi size={16} />
            )}
            {status === "connecting"
              ? t("connection.connecting")
              : t("connection.connect")}
          </button>
        ) : (
          <button
            onClick={onDisconnect}
            className="btn-danger flex-1 flex items-center justify-center gap-2"
          >
            <Wifi size={16} />
            {t("connection.disconnect")}
          </button>
        )}
      </div>
    </>
  );
}
