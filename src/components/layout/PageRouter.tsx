import { Suspense, lazy, useCallback } from "react";
import { useTranslation } from "react-i18next";
import ErrorBoundary from "@/components/ErrorBoundary";
import type { PidValue, DtcCode, DtcHistoryEntry, EcuInfo, MonitorStatus, Mode06Result, FreezeFrameData } from "@/stores/vehicle";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";

const Connection = lazy(() => import("@/pages/Connection"));
const Dashboard = lazy(() => import("@/pages/Dashboard"));
const LiveData = lazy(() => import("@/pages/LiveData"));
const DTC = lazy(() => import("@/pages/DTC"));
const ECUInfo = lazy(() => import("@/pages/ECUInfo"));
const Monitors = lazy(() => import("@/pages/Monitors"));
const History = lazy(() => import("@/pages/History"));
const Advanced = lazy(() => import("@/pages/Advanced"));

function LoadingFallback() {
  const { t } = useTranslation();
  return (
    <div className="flex items-center justify-center h-full">
      <div className="text-center">
        <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-obd-border border-t-obd-accent"></div>
        <p className="mt-4 text-obd-text/60">{t("common.loading")}</p>
      </div>
    </div>
  );
}

interface PageRouterProps {
  activePage: string;
  connection: {
    status: ConnectionStatus;
    vehicle: VehicleInfo | null;
    port: string;
    baudRate: number;
    availablePorts: string[];
    connect: () => void;
    disconnect: () => void;
    connectDemo: () => void;
    connectWifi: (host: string, port: number) => Promise<void>;
    connectBle?: (deviceName: string) => Promise<void>;
    setPort: (port: string) => void;
    setBaudRate: (baudRate: number) => void;
    scanPorts: () => void;
  };
  vehicle: {
    pidData: Map<number, PidValue>;
    dtcs: DtcCode[];
    dtcHistory: DtcHistoryEntry[];
    ecus: EcuInfo[];
    monitors: MonitorStatus[];
    mode06Results: Mode06Result[];
    isLoadingMode06: boolean;
    loadMode06Results: () => void;
    freezeFrame: FreezeFrameData[];
    isLoadingFreezeFrame: boolean;
    loadFreezeFrame: () => void;
    isPolling: boolean;
    startRealPolling: (ms: number, manufacturer: string) => void;
    startDemoPolling: (ms: number) => void;
    pausePolling: () => void;
    changeRefreshRate: (ms: number) => void;
    setEcus: (ecus: EcuInfo[]) => void;
    loadMonitors: () => Promise<void>;
  };
  discoveryProgress: number;
  isDiscoveryComplete: boolean;
  hasVinCache: boolean;
  onClearCache: () => Promise<void>;
  onVehicleUpdate: (info: any) => void;
  onReadAll: () => void;
  onClearAll: () => void;
  isReading: boolean;
  isClearing: boolean;
  isEcuScanning: boolean;
  onEcuScan: () => Promise<void>;
}

export default function PageRouter({
  activePage, connection, vehicle,
  discoveryProgress, isDiscoveryComplete, hasVinCache,
  onClearCache, onVehicleUpdate,
  onReadAll, onClearAll, isReading, isClearing,
  isEcuScanning, onEcuScan,
}: PageRouterProps) {
  const handleStartPolling = useCallback((ms: number) => {
    if (connection.status === "connected") {
      vehicle.startRealPolling(ms, connection.vehicle?.make || "");
    } else {
      vehicle.startDemoPolling(ms);
    }
  }, [connection.status, connection.vehicle?.make, vehicle.startRealPolling, vehicle.startDemoPolling]);

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
              onConnectBle={connection.connectBle}
              onPortChange={connection.setPort}
              onBaudRateChange={connection.setBaudRate}
              onScanPorts={connection.scanPorts}
              discoveryProgress={discoveryProgress}
              isDiscoveryComplete={isDiscoveryComplete}
              hasVinCache={hasVinCache}
              onClearCache={onClearCache}
              onVehicleUpdate={onVehicleUpdate}
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
              onStartPolling={handleStartPolling}
              onPausePolling={vehicle.pausePolling}
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
              onReadAll={onReadAll}
              onClearAll={onClearAll}
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
            <ECUInfo ecus={vehicle.ecus} isScanning={isEcuScanning} onScan={onEcuScan} />
          </Suspense>
        </ErrorBoundary>
      );
    case "monitors":
      return (
        <ErrorBoundary>
          <Suspense fallback={<LoadingFallback />}>
            <Monitors monitors={vehicle.monitors} onRefresh={vehicle.loadMonitors} />
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
}
