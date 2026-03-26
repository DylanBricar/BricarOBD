import { useState } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { RefreshCw, Wifi } from "lucide-react";
import { cn } from "@/lib/utils";
import type { ConnectionStatus } from "@/stores/connection";

interface WiFiAdapter {
  host: string;
  port: number;
  name: string;
}

const isValidIpAddress = (ip: string): boolean => {
  const ipRegex = /^(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/;
  return ipRegex.test(ip);
};

const isValidPort = (port: string): boolean => {
  const portNum = parseInt(port);
  return !isNaN(portNum) && portNum >= 1 && portNum <= 65535;
};

interface WiFiSettingsProps {
  isConnected: boolean;
  status: ConnectionStatus;
  onConnectWifi: (host: string, port: number) => Promise<void>;
  showToast: (message: string, type?: "success" | "error") => void;
}

export default function WiFiSettings({ isConnected, status, onConnectWifi, showToast }: WiFiSettingsProps) {
  const { t } = useTranslation();
  const [wifiHost, setWifiHost] = useState("192.168.0.10");
  const [wifiPort, setWifiPort] = useState("35000");
  const [wifiAdapters, setWifiAdapters] = useState<WiFiAdapter[]>([]);
  const [isScanning, setIsScanning] = useState(false);

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

  const isConnecting = status === "connecting";
  const canConnect = !!wifiHost && !!wifiPort && isValidIpAddress(wifiHost) && isValidPort(wifiPort) && !isConnecting;

  return (
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
              <div
                key={adapter.name}
                onClick={() => setWifiHost(adapter.host)}
                className="px-3 py-2 rounded-lg bg-white/[0.02] text-xs text-obd-text border border-obd-border/30 cursor-pointer hover:bg-white/[0.05] transition-colors"
              >
                {adapter.name} ({adapter.host}:{adapter.port})
              </div>
            ))}
          </div>
        </div>
      )}

      {/* WiFi Connect Button */}
      {!isConnected && (
        <button
          onClick={() => onConnectWifi(wifiHost, parseInt(wifiPort))}
          disabled={!canConnect}
          className={cn(
            "btn-accent-solid w-full flex items-center justify-center gap-2",
            !canConnect && "opacity-50 cursor-not-allowed"
          )}
        >
          {isConnecting ? (
            <RefreshCw size={16} className="animate-spin" />
          ) : (
            <Wifi size={16} />
          )}
          {isConnecting ? t("connection.connecting") : t("connection.connect")}
        </button>
      )}
    </>
  );
}
