import { useState, useEffect, useMemo, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { WebviewWindow } from "@tauri-apps/api/webviewWindow";
import { useTranslation } from "react-i18next";
import ErrorBoundary from "@/components/ErrorBoundary";
import Sidebar from "@/components/layout/Sidebar";
import StatusBar from "@/components/layout/StatusBar";
import PageRouter from "@/components/layout/PageRouter";
import { useConnectionStore } from "@/stores/connection";
import DevConsole from "@/components/DevConsole";
import { useVehicleData } from "@/stores/vehicle";
import type { DtcCode, EcuInfo, VehicleOperation, WriteOperation } from "@/stores/vehicle";
import { devInfo, devError } from "@/lib/devlog";
import { useThemeStore, setThemeMode, type ThemeMode } from "@/stores/theme";
import { useToast } from "@/hooks/useToast";
import { useConnectionEffects } from "@/hooks/useConnectionEffects";
import { useAutoUpdate } from "@/hooks/useAutoUpdate";
import { Toast } from "@/components/Toast";
import UpdateBanner from "@/components/UpdateBanner";

export default function App() {
  const { t, i18n } = useTranslation();
  const [activePage, setActivePage] = useState("connection");
  const [isReading, setIsReading] = useState(false);
  const [isEcuScanning, setIsEcuScanning] = useState(false);
  const [isClearing, setIsClearing] = useState(false);
  const [showDevConsole, setShowDevConsole] = useState(false);
  const { toast: toastMessage, showToast, dismissToast } = useToast();
  const connection = useConnectionStore();
  const vehicle = useVehicleData();
  const { mode: themeMode } = useThemeStore();
  const autoUpdate = useAutoUpdate();
  const vehicleActions = useMemo(() => ({
    startDemoPolling: vehicle.startDemoPolling,
    startRealPolling: vehicle.startRealPolling,
    stopPolling: vehicle.stopPolling,
    setDtcs: vehicle.setDtcs,
    setMonitors: vehicle.setMonitors,
    setEcus: vehicle.setEcus,
    setVehicleOps: vehicle.setVehicleOps,
    setVehicleWriteOps: vehicle.setVehicleWriteOps,
  }), [
    vehicle.startDemoPolling,
    vehicle.startRealPolling,
    vehicle.stopPolling,
    vehicle.setDtcs,
    vehicle.setMonitors,
    vehicle.setEcus,
    vehicle.setVehicleOps,
    vehicle.setVehicleWriteOps,
  ]);

  const { discoveryProgress, isDiscoveryComplete, hasVinCache, setHasVinCache, setIsDiscoveryComplete } = useConnectionEffects(
    connection.status,
    connection.vehicle,
    vehicleActions,
    i18n.language,
    showToast,
    t,
  );

  const hasVin = !!connection.vehicle?.vin;
  const isDemo = connection.status === "demo";
  const canNavigate = isDemo || (connection.status === "connected" && hasVin && isDiscoveryComplete);
  const isConnected =
    connection.status === "connected" || connection.status === "demo";

  // App mount
  useEffect(() => {
    devInfo("ui", "App mounted");
    connection.scanPorts();
  }, []);

  // Load DB stats on mount
  useEffect(() => {
    let cancelled = false;
    invoke<{ operations: number; profiles: number; ecus: number }>("get_database_stats")
      .then(stats => {
        if (cancelled) return;
        devInfo("ui", "DB loaded: " + JSON.stringify(stats));
        vehicle.setDbStats(stats);
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, []);

  // Load settings on mount
  useEffect(() => {
    let cancelled = false;
    invoke<{ language: string; defaultBaudRate: number; theme: string }>("get_settings")
      .then(settings => {
        if (cancelled) return;
        if (settings.language && settings.language !== i18n.language) {
          i18n.changeLanguage(settings.language);
        }
        if (settings.defaultBaudRate) {
          connection.setBaudRate(settings.defaultBaudRate);
        }
        if (settings.theme && ["system", "dark", "light"].includes(settings.theme)) {
          setThemeMode(settings.theme as ThemeMode);
        }
      })
      .catch(() => {});
    return () => { cancelled = true; };
  }, []);

  // Sync language with Rust backend
  useEffect(() => {
    invoke("set_language", { lang: i18n.language }).catch(() => {});
  }, [i18n.language]);

  // Persist settings when language, baud rate, or theme changes
  useEffect(() => {
    invoke("save_settings", { settings: { language: i18n.language, defaultBaudRate: connection.baudRate, theme: themeMode } }).catch(() => {});
  }, [i18n.language, connection.baudRate, themeMode]);

  // Navigate based on connection status
  const handleNavigate = useCallback((page: string) => {
    if (!canNavigate && page !== "connection" && page !== "advanced") {
      devInfo("ui", "Navigation blocked: not ready");
      return;
    }
    devInfo("ui", "Navigate: " + page);
    setActivePage(page);
  }, [canNavigate]);

  // Keyboard shortcuts for tab navigation
  useEffect(() => {
    const pages = ["connection", "dashboard", "liveData", "dtc", "ecuInfo", "monitors", "history", "advanced"];
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.metaKey) {
        const num = parseInt(e.key);
        if (num >= 1 && num <= 8) {
          e.preventDefault();
          const page = pages[num - 1];
          if (page === "connection" || page === "advanced" || canNavigate) {
            handleNavigate(page);
          }
        }
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => window.removeEventListener("keydown", handleKeyDown);
  }, [canNavigate, handleNavigate]);

  useEffect(() => {
    if (isConnected && activePage === "connection" && canNavigate) {
      setActivePage("dashboard");
    } else if (!isConnected && activePage !== "connection") {
      setIsReading(false);
      if (connection.vehicle) {
        invoke("save_session_cmd", {
          vin: connection.vehicle.vin,
          make: connection.vehicle.make,
          model: connection.vehicle.model,
          dtc_count: vehicle.dtcs.length,
          notes: vehicle.dtcs.map(d => d.code).join(", ") || t("dtc.noCode"),
          data: "",
        }).catch(() => {});
      }
      setActivePage("connection");
    }
  }, [isConnected, canNavigate]);


  const handleReadAll = async () => {
    setIsReading(true);
    try {
      const codes = await invoke<DtcCode[]>("read_all_dtcs", { lang: i18n.language });
      vehicle.setDtcs(codes);
    } catch (e) {
      devError("ui", "Read DTCs failed: " + String(e));
    }
    setIsReading(false);
  };

  const handleClearAll = async () => {
    setIsClearing(true);
    try {
      const result = await invoke<string>("clear_dtcs", { confirmed: true });
      vehicle.clearAllDtcs();
      if (result === "PARTIAL") {
        showToast(t("dtc.clearPartial"), "error");
      } else {
        showToast(t("dtc.clearSuccess"));
      }
    } catch (e) {
      showToast(String(e), "error");
    }
    setIsClearing(false);
  };

  const onClearCache = useCallback(async () => {
    try {
      await invoke("clear_vin_cache", { vin: connection.vehicle?.vin });
      setHasVinCache(false);
    } catch (e) {
      devError("ui", "Clear cache error: " + String(e));
    }
  }, [connection.vehicle?.vin, setHasVinCache]);

  const onVehicleUpdate = useCallback((info: any) => {
    connection.updateVehicle(info);
    const make = info.make || "";
    if (connection.status === "connected") {
      setIsDiscoveryComplete(false);
      vehicle.startRealPolling(1000, make, true);
      invoke<DtcCode[]>("read_all_dtcs", { lang: i18n.language })
        .then(codes => vehicle.setDtcs(codes)).catch(() => {});
    }
    invoke<VehicleOperation[]>("get_vehicle_operations", { vehicle: make, limit: 500 })
      .then(ops => vehicle.setVehicleOps(ops)).catch(() => {});
    invoke<WriteOperation[]>("get_write_operations", { ecuName: "%", vehicle: make })
      .then(ops => vehicle.setVehicleWriteOps(ops)).catch(() => {});
  }, [connection, vehicle, i18n.language, setIsDiscoveryComplete]);

  const handleToggleDevConsole = useCallback(async () => {
    try {
      const webview = new WebviewWindow("dev-console", {
        url: "#/devconsole",
        title: "BricarOBD — Dev Console",
        width: 1200,
        height: 600,
      });
      webview.once("tauri://error", () => {
        setShowDevConsole(prev => !prev);
      });
    } catch {
      setShowDevConsole(prev => !prev);
    }
  }, []);

  const handleEcuScan = useCallback(async () => {
    setIsEcuScanning(true);
    try {
      const ecus = await invoke<EcuInfo[]>("scan_ecus");
      vehicle.setEcus(ecus);
    } catch (e) {
      devInfo("ui", "ECU scan error: " + String(e));
    }
    setIsEcuScanning(false);
  }, [vehicle.setEcus]);

  return (
    <ErrorBoundary>
      <div className="flex h-screen w-screen overflow-hidden bg-obd-bg">
        <Sidebar
          activePage={activePage}
          onNavigate={handleNavigate}
          connectionStatus={connection.status}
          canNavigate={canNavigate}
          discoveryProgress={discoveryProgress}
          hasVin={hasVin}
          onToggleDevConsole={handleToggleDevConsole}
          dtcCount={vehicle.dtcs.length}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <UpdateBanner state={autoUpdate.state} onDownload={autoUpdate.downloadAndInstall} onDismiss={autoUpdate.dismiss} />
          <main className="flex-1 overflow-y-auto">
            <PageRouter
              activePage={activePage}
              connection={connection}
              vehicle={vehicle}
              discoveryProgress={discoveryProgress}
              isDiscoveryComplete={isDiscoveryComplete}
              hasVinCache={hasVinCache}
              onClearCache={onClearCache}
              onVehicleUpdate={onVehicleUpdate}
              onReadAll={handleReadAll}
              onClearAll={handleClearAll}
              isReading={isReading}
              isClearing={isClearing}
              isEcuScanning={isEcuScanning}
              onEcuScan={handleEcuScan}
            />
          </main>
          <StatusBar status={connection.status} vehicle={connection.vehicle} isPolling={vehicle.isPolling} />
        </div>
        {toastMessage && <Toast message={toastMessage.message} type={toastMessage.type} onDismiss={dismissToast} />}
        {showDevConsole && <DevConsole isStandalone={false} onClose={() => setShowDevConsole(false)} />}
      </div>
    </ErrorBoundary>
  );
}
