import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import {
  Plug,
  RefreshCw,
  Radio,
  ChevronDown,
  Wifi,
  Car,
  KeyRound,
  Fingerprint,
  X,
  Clock,
} from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";

interface ConnectionProps {
  status: ConnectionStatus;
  port: string;
  baudRate: number;
  vehicle: VehicleInfo | null;
  availablePorts: string[];
  onConnect: () => void;
  onDisconnect: () => void;
  onDemoConnect: () => void;
  onPortChange: (port: string) => void;
  onBaudRateChange: (baud: number) => void;
}

interface VinHistoryEntry {
  vin: string;
  make: string;
  model: string;
  year: number;
  lastSeen: number;
}

const STORAGE_KEY = "bricarobd_vin_history";

function loadVinHistory(): VinHistoryEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    return raw ? JSON.parse(raw) : [];
  } catch {
    return [];
  }
}

function saveVinHistory(entries: VinHistoryEntry[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
}

const baudRates = [9600, 38400, 115200, 230400, 500000];

export default function Connection({
  status,
  port,
  baudRate,
  vehicle,
  availablePorts,
  onConnect,
  onDisconnect,
  onDemoConnect,
  onPortChange,
  onBaudRateChange,
}: ConnectionProps) {
  const { t, i18n } = useTranslation();
  const [manualVin, setManualVin] = useState("");
  const [vinHistory, setVinHistory] = useState<VinHistoryEntry[]>(loadVinHistory);
  const [toast, setToast] = useState<{ message: string; type: "success" | "error" } | null>(null);
  const isConnected = status === "connected" || status === "demo";

  const showToast = (message: string, type: "success" | "error" = "success") => {
    setToast({ message, type });
    setTimeout(() => setToast(null), 5000);
  };

  // Save vehicle to history when connected
  useEffect(() => {
    if (vehicle && vehicle.vin) {
      setVinHistory((prev) => {
        const filtered = prev.filter((e) => e.vin !== vehicle.vin);
        const updated = [
          {
            vin: vehicle.vin,
            make: vehicle.make,
            model: vehicle.model,
            year: vehicle.year,
            lastSeen: Date.now(),
          },
          ...filtered,
        ].slice(0, 10); // Keep max 10
        saveVinHistory(updated);
        return updated;
      });
    }
  }, [vehicle]);

  const removeFromHistory = (vin: string) => {
    setVinHistory((prev) => {
      const updated = prev.filter((e) => e.vin !== vin);
      saveVinHistory(updated);
      return updated;
    });
  };

  return (
    <div className="p-6 space-y-6 animate-slide-in">
      <div className="flex items-center gap-3">
        <div className="w-10 h-10 rounded-xl bg-obd-accent/10 border border-obd-accent/20 flex items-center justify-center">
          <Plug className="text-obd-accent" size={20} />
        </div>
        <div>
          <h2 className="text-lg font-semibold">{t("connection.title")}</h2>
          <p className="text-xs text-obd-text-muted">{t("connection.adapterType")}</p>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-6">
        {/* Connection Settings */}
        <div className="glass-card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider">
            {t("connection.configuration")}
          </h3>

          {/* Port selector */}
          <div className="space-y-1.5">
            <label className="text-xs text-obd-text-muted">{t("connection.port")}</label>
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

          {/* Baud rate */}
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

          {/* Connect/Disconnect buttons */}
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

          {/* Demo mode */}
          <div className="pt-2 border-t border-obd-border/30">
            <button
              onClick={onDemoConnect}
              disabled={isConnected}
              className={cn(
                "w-full flex items-center gap-3 p-3 rounded-lg transition-all",
                "bg-obd-warning/5 border border-obd-warning/20 hover:bg-obd-warning/10",
                isConnected && "opacity-30 cursor-not-allowed"
              )}
            >
              <Radio size={18} className="text-obd-warning" />
              <div className="text-left">
                <p className="text-sm font-medium text-obd-warning">{t("connection.demo")}</p>
                <p className="text-[10px] text-obd-text-muted">{t("connection.demoDesc")}</p>
              </div>
            </button>
          </div>

          {/* Manual VIN */}
          <div className="space-y-1.5 pt-2 border-t border-obd-border/30">
            <label className="text-xs text-obd-text-muted">{t("connection.manualVin")}</label>
            <div className="flex gap-2">
              <input
                type="text"
                value={manualVin}
                onChange={(e) => setManualVin(e.target.value.toUpperCase())}
                placeholder="VF3LCBHZ6JS123456"
                maxLength={17}
                className="input-field font-mono text-xs flex-1"
              />
              <button
                onClick={() => {
                  if (manualVin.length === 17) {
                    showToast(`${t("connection.vin")}: ${manualVin}`);
                  } else if (manualVin.length > 0) {
                    showToast(t("connection.vinInvalid", { count: manualVin.length }), "error");
                  }
                }}
                disabled={manualVin.length !== 17}
                className={cn("btn-ghost text-xs px-3", manualVin.length !== 17 && manualVin.length > 0 && "opacity-50")}
              >OK</button>
            </div>
          </div>
        </div>

        {/* Right column: Vehicle Info + VIN History */}
        <div className="space-y-6">
          {/* Vehicle Info */}
          <div className="glass-card p-5 space-y-4">
            <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider">
              {t("connection.vehicle")}
            </h3>

            {vehicle ? (
              <div className="space-y-3">
                <div className="p-4 rounded-lg bg-obd-accent/5 border border-obd-accent/15">
                  <div className="flex items-center gap-3">
                    <Car size={24} className="text-obd-accent" />
                    <div>
                      <p className="font-semibold text-obd-text">
                        {vehicle.make} {vehicle.model}
                      </p>
                      <p className="text-xs text-obd-text-muted">{vehicle.year}</p>
                    </div>
                  </div>
                </div>
                <div className="space-y-2">
                  <InfoRow icon={<Fingerprint size={14} />} label={t("connection.vin")} value={vehicle.vin} mono />
                  <InfoRow icon={<Radio size={14} />} label={t("connection.protocol")} value={vehicle.protocol} />
                  <InfoRow icon={<KeyRound size={14} />} label={t("connection.elmVersion")} value={vehicle.elmVersion} />
                </div>
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center py-8 text-obd-text-muted">
                <Car size={48} strokeWidth={1} className="mb-3 opacity-20" />
                <p className="text-sm">{t("connection.disconnected")}</p>
              </div>
            )}

            {status === "error" && (
              <div className="p-3 rounded-lg bg-obd-danger/10 border border-obd-danger/20">
                <p className="text-xs text-obd-danger">{t("connection.errorMessage")}</p>
              </div>
            )}
          </div>

          {/* VIN History */}
          {vinHistory.length > 0 && (
            <div className="glass-card p-5 space-y-3">
              <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider flex items-center gap-2">
                <Clock size={14} />
                {t("connection.vinHistory")}
              </h3>
              <div className="space-y-1.5">
                {vinHistory.map((entry) => (
                  <div
                    key={entry.vin}
                    className="flex items-center gap-3 px-3 py-2 rounded-lg bg-white/[0.02] hover:bg-white/[0.04] transition-colors group"
                  >
                    <Car size={14} className="text-obd-accent flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <p className="text-xs font-medium text-obd-text">
                        {entry.make} {entry.model} <span className="text-obd-text-muted">({entry.year})</span>
                      </p>
                      <p className="text-[10px] font-mono text-obd-text-muted truncate">{entry.vin}</p>
                    </div>
                    <span className="text-[9px] text-obd-text-muted">
                      {new Date(entry.lastSeen).toLocaleDateString(i18n.language === "fr" ? "fr-FR" : "en-US")}
                    </span>
                    <button
                      onClick={() => removeFromHistory(entry.vin)}
                      className="p-1 rounded hover:bg-obd-danger/10 text-obd-text-muted hover:text-obd-danger transition-colors opacity-0 group-hover:opacity-100"
                      title={t("common.delete")}
                    >
                      <X size={12} />
                    </button>
                  </div>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Toast */}
      {toast && (
        <div className={cn(
          "fixed bottom-4 right-4 max-w-md px-4 py-3 rounded-lg shadow-lg flex items-start gap-3 animate-slide-in z-50",
          toast.type === "success" ? "bg-obd-success/90 text-white" : "bg-obd-danger/90 text-white"
        )}>
          <p className="text-xs flex-1 leading-relaxed break-all">{toast.message}</p>
          <button onClick={() => setToast(null)} className="flex-shrink-0 hover:opacity-70">
            <X size={14} />
          </button>
        </div>
      )}
    </div>
  );
}

function InfoRow({
  icon,
  label,
  value,
  mono,
}: {
  icon: React.ReactNode;
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 rounded-lg bg-white/[0.02]">
      <span className="text-obd-text-muted">{icon}</span>
      <span className="text-xs text-obd-text-muted flex-1">{label}</span>
      <span className={cn("text-xs text-obd-text", mono && "font-mono")}>{value}</span>
    </div>
  );
}
