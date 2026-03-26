import { useState, useEffect, useRef, Suspense, lazy } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import ErrorBoundary from "@/components/ErrorBoundary";
import Sidebar from "@/components/layout/Sidebar";
import StatusBar from "@/components/layout/StatusBar";
import { useConnectionStore } from "@/stores/connection";
import DevConsole from "@/components/DevConsole";
import { useVehicleData } from "@/stores/vehicle";
import type { DtcCode, EcuInfo, MonitorStatus, VehicleOperation, WriteOperation } from "@/stores/vehicle";
import { devInfo, devError } from "@/lib/devlog";

const Connection = lazy(() => import("@/pages/Connection"));
const Dashboard = lazy(() => import("@/pages/Dashboard"));
const LiveData = lazy(() => import("@/pages/LiveData"));
const DTC = lazy(() => import("@/pages/DTC"));
const ECUInfo = lazy(() => import("@/pages/ECUInfo"));
const Monitors = lazy(() => import("@/pages/Monitors"));
const History = lazy(() => import("@/pages/History"));
const Advanced = lazy(() => import("@/pages/Advanced"));

export default function App() {
  const { t, i18n } = useTranslation();
  const [activePage, setActivePage] = useState("connection");
  const [isReading, setIsReading] = useState(false);
  const [isEcuScanning, setIsEcuScanning] = useState(false);
  const [isClearing, setIsClearing] = useState(false);
  const [toastMessage, setToastMessage] = useState<{ message: string; type: "success" | "error" } | null>(null);
  const [showDevConsole, setShowDevConsole] = useState(false);
  const toastTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const connection = useConnectionStore();
  const vehicle = useVehicleData();

  const isConnected =
    connection.status === "connected" || connection.status === "demo";

  // App mount
  useEffect(() => {
    devInfo("ui", "App mounted");
    connection.scanPorts();
  }, []);

  // Load DB stats on mount
  useEffect(() => {
    invoke<{ operations: number; profiles: number; ecus: number }>("get_database_stats")
      .then(stats => {
        devInfo("ui", "DB loaded: " + JSON.stringify(stats));
        vehicle.setDbStats(stats);
      })
      .catch(() => {});
  }, []);

  // Load settings on mount
  useEffect(() => {
    invoke<{ language: string; defaultBaudRate: number }>("get_settings")
      .then(settings => {
        if (settings.language && settings.language !== i18n.language) {
          i18n.changeLanguage(settings.language);
        }
        if (settings.defaultBaudRate) {
          connection.setBaudRate(settings.defaultBaudRate);
        }
      })
      .catch(() => {});
  }, []);

  // Sync language with Rust backend
  useEffect(() => {
    invoke("set_language", { lang: i18n.language }).catch(() => {});
  }, [i18n.language]);

  // Persist settings when language or baud rate changes
  useEffect(() => {
    invoke("save_settings", { settings: { language: i18n.language, defaultBaudRate: connection.baudRate } }).catch(() => {});
  }, [i18n.language, connection.baudRate]);

  // Start polling + load DTCs + load vehicle operations when connected
  useEffect(() => {
    devInfo("ui", "Connection: " + connection.status);
    if (connection.status === "demo") {
      devInfo("ui", "Demo polling started");
      invoke("discover_vehicle_params", { manufacturer: "Peugeot" }).catch(() => {});
      vehicle.startDemoPolling();
    } else if (connection.status === "connected") {
      // Real vehicle: discover supported params first, then poll
      const make = connection.vehicle?.make || "";
      devInfo("ui", "Starting vehicle discovery for " + make);

      // Run discovery, then start polling with discovered params only
      invoke<{ standardPids: number; manufacturerDids: number }>("discover_vehicle_params", { manufacturer: make })
        .then(result => {
          devInfo("ui", `Discovery: ${result.standardPids} PIDs + ${result.manufacturerDids} DIDs`);
          vehicle.startRealPolling(1000, make, true); // skipEcuScan since discovery did the heavy lifting
        })
        .catch(() => {
          devInfo("ui", "Discovery failed, starting polling with fallback");
          vehicle.startRealPolling(1000, make);
        });

      // Load ECUs and monitors in parallel with discovery
      invoke<EcuInfo[]>("scan_ecus").then(ecus => vehicle.setEcus(ecus)).catch(() => {});
      invoke<MonitorStatus[]>("get_monitors").then(m => vehicle.setMonitors(m)).catch(() => {});

      invoke<DtcCode[]>("read_all_dtcs", { lang: i18n.language }).then(codes => {
        devInfo("ui", "DTCs loaded: " + codes.length);
        vehicle.setDtcs(codes);
      }).catch(() => {});

      // Load vehicle-specific operations from the 3.17M DB
      const vehicleMake = connection.vehicle?.make || "";
      invoke<VehicleOperation[]>("get_vehicle_operations", { vehicle: vehicleMake, limit: 500 })
        .then(ops => {
          devInfo("ui", "Vehicle ops: " + ops.length);
          vehicle.setVehicleOps(ops);
        })
        .catch(() => {});
      invoke<WriteOperation[]>("get_write_operations", { ecuName: "%", vehicle: vehicleMake })
        .then(ops => vehicle.setVehicleWriteOps(ops))
        .catch(() => {});
    } else if (connection.status === "disconnected") {
      vehicle.stopPolling();
      setIsReading(false);
    }
  }, [connection.status]);

  // Navigate based on connection status
  useEffect(() => {
    if (isConnected && activePage === "connection") {
      setActivePage("dashboard");
    } else if (!isConnected && activePage !== "connection") {
      // Auto-save session on disconnect
      if (connection.vehicle) {
        invoke("save_session_cmd", {
          vin: connection.vehicle.vin,
          make: connection.vehicle.make,
          model: connection.vehicle.model,
          dtc_count: vehicle.dtcs.length,
          notes: vehicle.dtcs.map(d => d.code).join(", ") || "No DTCs",
        }).catch(() => {});
      }
      setActivePage("connection");
    }
  }, [isConnected]);

  // Cleanup toast timer on unmount
  useEffect(() => {
    return () => {
      if (toastTimerRef.current) {
        clearTimeout(toastTimerRef.current);
      }
    };
  }, []);

  const handleNavigate = (page: string) => {
    devInfo("ui", "Navigate: " + page);
    setActivePage(page);
  };

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
      await invoke("clear_dtcs", { confirmed: true });
      vehicle.clearAllDtcs();
      setToastMessage({ message: t("dtc.clearSuccess"), type: "success" });
    } catch (e) {
      setToastMessage({ message: String(e), type: "error" });
    }
    setIsClearing(false);
    if (toastTimerRef.current) {
      clearTimeout(toastTimerRef.current);
    }
    toastTimerRef.current = setTimeout(() => setToastMessage(null), 5000);
  };

  const LoadingFallback = () => (
    <div className="flex items-center justify-center h-full">
      <div className="text-center">
        <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-obd-border border-t-obd-accent"></div>
        <p className="mt-4 text-obd-text/60">{t("common.loading")}</p>
      </div>
    </div>
  );

  const renderPage = () => {
    switch (activePage) {
      case "connection":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <Connection
                status={connection.status}
                port={connection.port}
                baudRate={connection.baudRate}
                vehicle={connection.vehicle}
                availablePorts={connection.availablePorts}
                onConnect={connection.connect}
                onDisconnect={connection.disconnect}
                onDemoConnect={connection.connectDemo}
                onConnectWifi={connection.connectWifi}
                onPortChange={connection.setPort}
                onBaudRateChange={connection.setBaudRate}
                onVehicleUpdate={(info) => {
                  connection.updateVehicle(info);
                  const make = info.make || "";
                  if (connection.status === "connected") {
                    vehicle.startRealPolling(1000, make, true);
                    invoke<DtcCode[]>("read_all_dtcs", { lang: i18n.language })
                      .then(codes => vehicle.setDtcs(codes)).catch(() => {});
                  }
                  invoke<VehicleOperation[]>("get_vehicle_operations", { vehicle: make, limit: 500 })
                    .then(ops => vehicle.setVehicleOps(ops)).catch(() => {});
                  invoke<WriteOperation[]>("get_write_operations", { ecuName: "%", vehicle: make })
                    .then(ops => vehicle.setVehicleWriteOps(ops)).catch(() => {});
                }}
              />
            </Suspense>
          </ErrorBoundary>
        );
      case "dashboard":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <Dashboard pidData={vehicle.pidData} />
            </Suspense>
          </ErrorBoundary>
        );
      case "liveData":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <LiveData
                pidData={vehicle.pidData}
                isPolling={vehicle.isPolling}
                onStartPolling={(ms: number) => {
                  if (connection.status === "connected") {
                    vehicle.startRealPolling(ms, connection.vehicle?.make || "");
                  } else {
                    vehicle.startDemoPolling(ms);
                  }
                }}
                onPausePolling={vehicle.pausePolling}
                onStopPolling={vehicle.stopPolling}
                onChangeRefreshRate={vehicle.changeRefreshRate}
              />
            </Suspense>
          </ErrorBoundary>
        );
      case "dtc":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <DTC
                dtcs={vehicle.dtcs}
                dtcHistory={vehicle.dtcHistory}
                vehicle={connection.vehicle}
                onReadAll={handleReadAll}
                onClearAll={handleClearAll}
                isReading={isReading}
                isClearing={isClearing}
                mode06Results={vehicle.mode06Results}
                isLoadingMode06={vehicle.isLoadingMode06}
                onLoadMode06={vehicle.loadMode06Results}
                freezeFrame={vehicle.freezeFrame}
                isLoadingFreezeFrame={vehicle.isLoadingFreezeFrame}
                onLoadFreezeFrame={vehicle.loadFreezeFrame}
              />
            </Suspense>
          </ErrorBoundary>
        );
      case "ecuInfo":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <ECUInfo ecus={vehicle.ecus} isScanning={isEcuScanning} onScan={async () => {
                setIsEcuScanning(true);
                try {
                  const ecus = await invoke<EcuInfo[]>("scan_ecus");
                  vehicle.setEcus(ecus);
                } catch (e) {
                  devInfo("ui", "ECU scan error: " + String(e));
                }
                setIsEcuScanning(false);
              }} />
            </Suspense>
          </ErrorBoundary>
        );
      case "monitors":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <Monitors monitors={vehicle.monitors} />
            </Suspense>
          </ErrorBoundary>
        );
      case "history":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <History />
            </Suspense>
          </ErrorBoundary>
        );
      case "advanced":
        return (
          <ErrorBoundary>
            <Suspense fallback={<LoadingFallback />}>
              <Advanced />
            </Suspense>
          </ErrorBoundary>
        );
      default:
        return null;
    }
  };

  return (
    <ErrorBoundary>
      <div className="flex h-screen w-screen overflow-hidden bg-obd-bg">
        <Sidebar
          activePage={activePage}
          onNavigate={handleNavigate}
          connectionStatus={connection.status}
          onToggleDevConsole={() => setShowDevConsole(!showDevConsole)}
        />
        <div className="flex-1 flex flex-col overflow-hidden">
          <main className="flex-1 overflow-y-auto">{renderPage()}</main>
          <StatusBar status={connection.status} vehicle={connection.vehicle} />
        </div>
        {toastMessage && (
          <div className={cn("fixed bottom-4 right-4 max-w-md px-4 py-3 rounded-lg shadow-lg flex items-start gap-3 animate-slide-in z-50 text-white", toastMessage.type === "success" ? "bg-obd-success/90" : "bg-obd-danger/90")}>
            <p className="text-xs flex-1 leading-relaxed">{toastMessage.message}</p>
            <button onClick={() => setToastMessage(null)} className="flex-shrink-0 hover:opacity-70 text-white" aria-label={t("common.close")}>
              ✕
            </button>
          </div>
        )}
        {showDevConsole && <DevConsole onClose={() => setShowDevConsole(false)} />}
      </div>
    </ErrorBoundary>
  );
}
