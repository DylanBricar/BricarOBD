import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import {
  Plug,
  RefreshCw,
  Radio,
  ChevronDown,
  ChevronUp,
  Wifi,
  Car,
  KeyRound,
  Fingerprint,
  Clock,
  Smartphone,
  X,
  AlertCircle,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useToast } from "@/hooks/useToast";
import { Toast } from "@/components/Toast";
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
  onVehicleUpdate?: (vehicle: VehicleInfo) => void;
}

interface VinHistoryEntry {
  vin: string;
  make: string;
  model: string;
  year: number;
  lastSeen: number;
}

interface WiFiAdapter {
  host: string;
  port: number;
  name: string;
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

// WiFi validation functions
const isValidIpAddress = (ip: string): boolean => {
  const ipRegex = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
  return ipRegex.test(ip);
};

const isValidPort = (port: string): boolean => {
  const portNum = parseInt(port);
  return !isNaN(portNum) && portNum >= 1 && portNum <= 65535;
};

const isValidVin = (vin: string): boolean => {
  if (vin.length !== 17) return false;
  const invalidChars = /[IOQ]/i;
  return !invalidChars.test(vin) && /^[A-Z0-9]+$/.test(vin);
};

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
  onVehicleUpdate,
}: ConnectionProps) {
  const { t, i18n } = useTranslation();
  const [manualVin, setManualVin] = useState("");
  const [vinHistory, setVinHistory] = useState<VinHistoryEntry[]>(loadVinHistory);
  const { toast, showToast, dismissToast } = useToast();
  const [connectionType, setConnectionType] = useState<"usb" | "wifi">("usb");
  const [wifiHost, setWifiHost] = useState("192.168.0.10");
  const [wifiPort, setWifiPort] = useState("35000");
  const [wifiAdapters, setWifiAdapters] = useState<WiFiAdapter[]>([]);
  const [isScanning, setIsScanning] = useState(false);
  const [showTroubleshoot, setShowTroubleshoot] = useState(false);
  const isConnected = status === "connected" || status === "demo";

  // Show troubleshooting guide on connection error
  useEffect(() => {
    if (status === "error") setShowTroubleshoot(true);
    if (status === "connected" || status === "demo") setShowTroubleshoot(false);
  }, [status]);

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

  const handleScanWifi = async () => {
    setIsScanning(true);
    try {
      const adapters = await invoke<WiFiAdapter[]>("scan_wifi");
      setWifiAdapters(adapters);
      if (adapters.length > 0) {
        showToast(t("connection.wifiFound", { count: adapters.length }));
      } else {
        showToast(t("connection.wifiNone"), "error");
      }
    } catch (e) {
      showToast(`${t("common.error")}: ${e instanceof Error ? e.message : String(e)}`, "error");
    }
    setIsScanning(false);
  };

  const handleConnectWifi = async () => {
    try {
      await invoke("connect_wifi", { host: wifiHost, port: parseInt(wifiPort) });
      showToast(t("connection.connected"));
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
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

      <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
        {/* Connection Settings */}
        <div className="glass-card p-5 space-y-4">
          <h3 className="text-sm font-semibold text-obd-text-secondary uppercase tracking-wider">
            {t("connection.configuration")}
          </h3>

          {/* Connection Type Selector */}
          <div className="flex gap-2">
            <button
              onClick={() => setConnectionType("usb")}
              disabled={isConnected}
              className={cn(
                "flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all border",
                connectionType === "usb"
                  ? "bg-obd-accent text-white border-obd-accent"
                  : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40",
                isConnected && "opacity-50 cursor-not-allowed"
              )}
            >
              {t("connection.usb")}
            </button>
            <button
              onClick={() => setConnectionType("wifi")}
              disabled={isConnected}
              className={cn(
                "flex-1 px-3 py-2 rounded-lg text-xs font-medium transition-all border",
                connectionType === "wifi"
                  ? "bg-obd-accent text-white border-obd-accent"
                  : "bg-obd-border/20 text-obd-text-muted border-obd-border/30 hover:bg-obd-border/40",
                isConnected && "opacity-50 cursor-not-allowed"
              )}
            >
              <Smartphone size={14} className="inline mr-1" />
              {t("connection.wifi")}
            </button>
          </div>

          {/* USB Configuration */}
          {connectionType === "usb" && (
            <>
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
            </>
          )}

          {/* WiFi Configuration */}
          {connectionType === "wifi" && (
            <>
              {/* WiFi Host */}
              <div className="space-y-1.5">
                <label className="text-xs text-obd-text-muted">{t("connection.wifiHost")}</label>
                <input
                  type="text"
                  value={wifiHost}
                  onChange={(e) => setWifiHost(e.target.value)}
                  placeholder="192.168.0.10"
                  disabled={isConnected}
                  className={cn("input-field text-xs", wifiHost && !isValidIpAddress(wifiHost) && "border-obd-danger")}
                />
                {wifiHost && !isValidIpAddress(wifiHost) && (
                  <p className="text-xs text-obd-danger">{t("connection.wifiHostInvalid")}</p>
                )}
              </div>

              {/* WiFi Port */}
              <div className="space-y-1.5">
                <label className="text-xs text-obd-text-muted">{t("connection.wifiPort")}</label>
                <input
                  type="text"
                  value={wifiPort}
                  onChange={(e) => setWifiPort(e.target.value)}
                  placeholder="35000"
                  disabled={isConnected}
                  className={cn("input-field text-xs", wifiPort && !isValidPort(wifiPort) && "border-obd-danger")}
                />
                {wifiPort && !isValidPort(wifiPort) && (
                  <p className="text-xs text-obd-danger">{t("connection.wifiPortInvalid")}</p>
                )}
              </div>

              {/* Scan WiFi button */}
              <button
                onClick={handleScanWifi}
                disabled={isConnected || isScanning}
                className={cn(
                  "w-full btn-ghost text-xs flex items-center justify-center gap-2",
                  (isConnected || isScanning) && "opacity-50 cursor-not-allowed"
                )}
              >
                {isScanning ? (
                  <RefreshCw size={14} className="animate-spin" />
                ) : (
                  <Wifi size={14} />
                )}
                {isScanning ? t("connection.scanning") : t("connection.wifiScan")}
              </button>

              {/* WiFi Adapters List */}
              {wifiAdapters.length > 0 && (
                <div className="space-y-1.5">
                  <label className="text-xs text-obd-text-muted">{t("connection.wifiAdapters")}</label>
                  <div className="space-y-1">
                    {wifiAdapters.map((adapter) => (
                      <div key={adapter.name} className="px-3 py-2 rounded-lg bg-white/[0.02] text-xs text-obd-text border border-obd-border/30">
                        {adapter.name} ({adapter.host}:{adapter.port})
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </>
          )}

          {/* Connect/Disconnect buttons */}
          <div className="flex gap-3 pt-2">
            {!isConnected ? (
              <>
                {connectionType === "usb" ? (
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
                    onClick={handleConnectWifi}
                    disabled={!wifiHost || !wifiPort || !isValidIpAddress(wifiHost) || !isValidPort(wifiPort) || status === "connecting"}
                    className={cn(
                      "btn-accent-solid flex-1 flex items-center justify-center gap-2",
                      (!wifiHost || !wifiPort || !isValidIpAddress(wifiHost) || !isValidPort(wifiPort) || status === "connecting") && "opacity-50 cursor-not-allowed"
                    )}
                  >
                    {status === "connecting" ? (
                      <RefreshCw size={16} className="animate-spin" />
                    ) : (
                      <Smartphone size={16} />
                    )}
                    {status === "connecting"
                      ? t("connection.connecting")
                      : t("connection.connect")}
                  </button>
                )}
              </>
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
              <div className="flex-1">
                <input
                  type="text"
                  value={manualVin}
                  onChange={(e) => setManualVin(e.target.value.toUpperCase())}
                  placeholder="VF3LCBHZ6JS123456"
                  maxLength={17}
                  className={cn("input-field font-mono text-xs w-full", manualVin && !isValidVin(manualVin) && "border-obd-danger")}
                />
                <p className="text-xs text-obd-text-muted mt-1">
                  {t("connection.vinLength", { current: manualVin.length })}
                </p>
                {manualVin && /[IOQ]/i.test(manualVin) && (
                  <p className="text-xs text-obd-danger mt-1">{t("connection.vinInvalidChars")}</p>
                )}
              </div>
              <button
                onClick={async () => {
                  if (isValidVin(manualVin)) {
                    try {
                      const info = await invoke<VehicleInfo>("set_manual_vin", { vin: manualVin });
                      onVehicleUpdate?.(info);
                      showToast(`${t("connection.vin")}: ${info.make || manualVin} ${info.year || ""}`);
                    } catch (e) {
                      showToast(String(e), "error");
                    }
                  } else if (manualVin.length > 0) {
                    showToast(t("connection.vinInvalid", { count: manualVin.length }), "error");
                  }
                }}
                disabled={!isValidVin(manualVin)}
                className={cn("btn-ghost text-xs px-3", manualVin && !isValidVin(manualVin) && "opacity-50")}
              >{t("common.ok")}</button>
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
                    onClick={() => setManualVin(entry.vin)}
                    className="flex items-center gap-3 px-3 py-2 rounded-lg bg-white/[0.02] hover:bg-white/[0.04] transition-colors group cursor-pointer"
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
                      onClick={(e) => { e.stopPropagation(); removeFromHistory(entry.vin); }}
                      className="p-1 rounded hover:bg-obd-danger/10 text-obd-text-muted hover:text-obd-danger transition-colors opacity-0 group-hover:opacity-100"
                      aria-label={t("common.delete")}
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

      {/* Troubleshooting Guide */}
      {showTroubleshoot && (
        <div className="glass-card p-4 border-obd-warning/30">
          <button
            onClick={() => setShowTroubleshoot((prev) => !prev)}
            className="flex items-center gap-2 w-full text-left"
          >
            <AlertCircle size={16} className="text-obd-warning" />
            <span className="text-sm font-semibold text-obd-warning flex-1">
              {t("connection.troubleshoot.title")}
            </span>
            <ChevronUp size={14} className="text-obd-text-muted" />
          </button>
          <ol className="mt-3 space-y-1.5 list-decimal list-inside">
            {[1, 2, 3, 4, 5, 6].map((n) => (
              <li key={n} className="text-xs text-obd-text-muted">
                {t(`connection.troubleshoot.tip${n}`)}
              </li>
            ))}
          </ol>
        </div>
      )}

      {/* Toast */}
      {toast && <Toast message={toast.message} type={toast.type} onDismiss={dismissToast} />}
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
