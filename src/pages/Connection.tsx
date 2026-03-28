import { useState, useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Plug, RefreshCw, Loader2, Wifi, Bluetooth } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { cn } from "@/lib/utils";
import WiFiSettings from "@/components/connection/WiFiSettings";
import ManualVinInput from "@/components/connection/ManualVinInput";
import Troubleshooting from "@/components/connection/Troubleshooting";
import VinHistoryPanel from "@/components/connection/VinHistoryPanel";
import DiscoveryStatus from "@/components/connection/DiscoveryStatus";
import ConnectionTypeSelector from "@/components/connection/ConnectionTypeSelector";
import UsbConfiguration from "@/components/connection/UsbConfiguration";
import DemoModeButton from "@/components/connection/DemoModeButton";
import VehicleInfoCard from "@/components/connection/VehicleInfoCard";
import { useToast } from "@/hooks/useToast";
import { useAndroidBridge } from "@/hooks/useAndroidBridge";
import { Toast } from "@/components/Toast";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";

function getAndroidUsb(): any {
  return (window as any).AndroidUsb ?? null;
}

interface ConnectionProps {
  status: ConnectionStatus;
  port: string;
  baudRate: number;
  vehicle: VehicleInfo | null;
  availablePorts: string[];
  onConnect: () => void;
  onDisconnect: () => void;
  onDemoConnect: () => void;
  onConnectWifi: (host: string, port: number) => Promise<void>;
  onConnectBle?: (deviceName: string) => Promise<void>;
  onPortChange: (port: string) => void;
  onBaudRateChange: (baud: number) => void;
  onScanPorts: () => void;
  discoveryProgress: number;
  isDiscoveryComplete: boolean;
  hasVinCache: boolean;
  onClearCache: () => Promise<void>;
  onVehicleUpdate?: (vehicle: VehicleInfo) => void;
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


export default function Connection({
  status,
  port,
  baudRate,
  vehicle,
  availablePorts,
  onConnect,
  onDisconnect,
  onDemoConnect,
  onConnectWifi,
  onConnectBle,
  onPortChange,
  onBaudRateChange,
  onScanPorts,
  discoveryProgress,
  isDiscoveryComplete,
  hasVinCache,
  onClearCache,
  onVehicleUpdate,
}: ConnectionProps) {
  const { t, i18n } = useTranslation();
  const [manualVin, setManualVin] = useState("");
  const [vinHistory, setVinHistory] = useState<VinHistoryEntry[]>(loadVinHistory);
  const { toast, showToast, dismissToast } = useToast();
  const [connectionType, setConnectionType] = useState<"usb" | "wifi" | "usb_android" | "ble">("usb");
  const [isAndroid, setIsAndroid] = useState(!!getAndroidUsb());
  const [showTroubleshoot, setShowTroubleshoot] = useState(false);
  const isConnected = status === "connected" || status === "demo";
  const android = useAndroidBridge(baudRate, onConnectWifi, onConnectBle, onDisconnect, showToast);

  // Poll for Android bridge injection (may not be ready at first render)
  useEffect(() => {
    if (isAndroid) return;
    let attempts = 0;
    const timer = setInterval(() => {
      if (getAndroidUsb()) {
        setIsAndroid(true);
        clearInterval(timer);
      }
      if (++attempts >= 10) clearInterval(timer);
    }, 200);
    return () => clearInterval(timer);
  }, [isAndroid]);

  // Show troubleshooting guide on connection error
  useEffect(() => {
    if (status === "error") setShowTroubleshoot(true);
    if (status === "connected" || status === "demo") setShowTroubleshoot(false);
  }, [status]);

  // Save vehicle to history when connected (not for demo)
  useEffect(() => {
    if (vehicle && vehicle.vin && status !== "demo") {
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
  }, [vehicle?.vin, vehicle?.make, status]);

  const removeFromHistory = (vin: string) => {
    setVinHistory((prev) => {
      const updated = prev.filter((e) => e.vin !== vin);
      saveVinHistory(updated);
      return updated;
    });
  };

  const handleVinHistorySelect = async (vin: string) => {
    setManualVin(vin);
    try {
      const info = await invoke<any>("set_manual_vin", { vin });
      onVehicleUpdate?.(info);
      showToast(`${t("connection.vin")}: ${info.make || vin} ${info.year || ""}`);
    } catch (e) {
      showToast(String(e), "error");
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

          <ConnectionTypeSelector
            connectionType={connectionType}
            onTypeChange={setConnectionType}
            isConnected={isConnected}
            isAndroid={isAndroid}
            t={t}
          />

          {/* USB Configuration */}
          {connectionType === "usb" && (
            <UsbConfiguration
              port={port}
              baudRate={baudRate}
              isConnected={isConnected}
              status={status}
              availablePorts={availablePorts}
              onPortChange={onPortChange}
              onBaudRateChange={onBaudRateChange}
              onScanPorts={onScanPorts}
              onConnect={onConnect}
              onDisconnect={onDisconnect}
              t={t}
            />
          )}

          {/* WiFi Configuration */}
          {connectionType === "wifi" && (
            <WiFiSettings isConnected={isConnected} status={status} onConnectWifi={onConnectWifi} showToast={showToast} />
          )}

          {/* USB Android Configuration */}
          {connectionType === "usb_android" && !isConnected && (
            <div className="space-y-3">
              <div className="flex gap-2">
                <button
                  onClick={android.handleScanUsb}
                  className="btn-accent flex items-center gap-1.5 text-xs"
                >
                  <RefreshCw size={14} />
                  {t("connection.usbScan")}
                </button>
              </div>
              {android.usbDevices.length > 0 && (
                <>
                  <div className="space-y-1.5">
                    <label className="text-xs text-obd-text-muted">{t("connection.port")}</label>
                    <select
                      value={android.selectedUsbDevice}
                      onChange={(e) => android.setSelectedUsbDevice(e.target.value)}
                      className="input-field appearance-none"
                    >
                      {android.usbDevices.map((d) => (
                        <option key={d.deviceId} value={d.deviceId}>
                          {d.name} (VID:{d.vendorId})
                        </option>
                      ))}
                    </select>
                  </div>
                  <button
                    onClick={android.handleConnectUsb}
                    disabled={!android.selectedUsbDevice || status === "connecting"}
                    className={cn(
                      "btn-accent-solid w-full flex items-center justify-center gap-2",
                      (!android.selectedUsbDevice || status === "connecting") && "opacity-50 cursor-not-allowed"
                    )}
                  >
                    {status === "connecting" ? (
                      <Loader2 size={16} className="animate-spin" />
                    ) : (
                      <Plug size={16} />
                    )}
                    {status === "connecting" ? t("connection.connecting") : t("connection.connect")}
                  </button>
                </>
              )}
            </div>
          )}

          {/* BLE Configuration */}
          {connectionType === "ble" && !isConnected && (
            <div className="space-y-3">
              <div className="flex gap-2">
                <button
                  onClick={android.handleScanBle}
                  className="btn-accent flex items-center gap-1.5 text-xs"
                >
                  <RefreshCw size={14} />
                  {t("connection.bleScan")}
                </button>
              </div>
              {android.bleDevices.length > 0 && (
                <>
                  <div className="space-y-1.5">
                    <label className="text-xs text-obd-text-muted">{t("connection.bleDevice")}</label>
                    <select
                      value={android.selectedBleDevice}
                      onChange={(e) => android.setSelectedBleDevice(e.target.value)}
                      className="input-field appearance-none"
                    >
                      {android.bleDevices.map((d) => (
                        <option key={d.address} value={d.address}>
                          {d.name} ({d.address})
                        </option>
                      ))}
                    </select>
                  </div>
                  <button
                    onClick={android.handleConnectBle}
                    disabled={!android.selectedBleDevice || status === "connecting"}
                    className={cn(
                      "btn-accent-solid w-full flex items-center justify-center gap-2",
                      (!android.selectedBleDevice || status === "connecting") && "opacity-50 cursor-not-allowed"
                    )}
                  >
                    {status === "connecting" ? (
                      <Loader2 size={16} className="animate-spin" />
                    ) : (
                      <Bluetooth size={16} />
                    )}
                    {status === "connecting" ? t("connection.connecting") : t("connection.connect")}
                  </button>
                </>
              )}
            </div>
          )}

          {connectionType === "wifi" && isConnected && (
            <div className="flex gap-3 pt-2">
              <button
                onClick={onDisconnect}
                className="btn-danger flex-1 flex items-center justify-center gap-2"
              >
                <Wifi size={16} />
                {t("connection.disconnect")}
              </button>
            </div>
          )}
          {connectionType === "usb_android" && isConnected && (
            <div className="flex gap-3 pt-2">
              <button
                onClick={android.handleDisconnectUsb}
                className="btn-danger flex-1 flex items-center justify-center gap-2"
              >
                <Plug size={16} />
                {t("connection.disconnect")}
              </button>
            </div>
          )}
          {connectionType === "ble" && isConnected && (
            <div className="flex gap-3 pt-2">
              <button
                onClick={android.handleDisconnectBle}
                className="btn-danger flex-1 flex items-center justify-center gap-2"
              >
                <Bluetooth size={16} />
                {t("connection.disconnect")}
              </button>
            </div>
          )}

          <DemoModeButton isConnected={isConnected} onClick={onDemoConnect} t={t} />

          <DiscoveryStatus
            discoveryProgress={discoveryProgress}
            isDiscoveryComplete={isDiscoveryComplete}
            status={status}
            vehicle={vehicle}
            hasVinCache={hasVinCache}
            onClearCache={onClearCache}
            t={t}
          />

          {/* Manual VIN */}
          <ManualVinInput value={manualVin} onChange={setManualVin} onVehicleUpdate={onVehicleUpdate} showToast={showToast} />
        </div>

        {/* Right column: Vehicle Info + VIN History */}
        <div className="space-y-6">
          <VehicleInfoCard vehicle={vehicle} status={status} t={t} />

          {/* VIN History */}
          {vinHistory.length > 0 && (
            <VinHistoryPanel
              vinHistory={vinHistory}
              onSelectVin={handleVinHistorySelect}
              onRemoveFromHistory={removeFromHistory}
              onCacheCleared={() => showToast(t("connection.clearCache"))}
              isClearing={false}
              t={t}
              language={i18n.language}
            />
          )}
        </div>
      </div>

      {/* Troubleshooting Guide */}
      {showTroubleshoot && (
        <Troubleshooting onClose={() => setShowTroubleshoot(false)} />
      )}

      {/* Toast */}
      {toast && <Toast message={toast.message} type={toast.type} onDismiss={dismissToast} />}
    </div>
  );
}
