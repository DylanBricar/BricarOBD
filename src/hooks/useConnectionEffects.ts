import { useState, useEffect, useRef, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { devInfo } from "@/lib/devlog";
import type { ConnectionStatus, VehicleInfo } from "@/stores/connection";
import type { DtcCode, EcuInfo, MonitorStatus, VehicleOperation, WriteOperation } from "@/stores/vehicle";

interface VehicleActions {
  startDemoPolling: (intervalMs?: number) => void;
  startRealPolling: (intervalMs?: number, manufacturer?: string, skipEcuScan?: boolean) => void;
  stopPolling: () => void;
  setDtcs: (dtcs: DtcCode[]) => void;
  setMonitors: (monitors: MonitorStatus[]) => void;
  setEcus: (ecus: EcuInfo[]) => void;
  setVehicleOps: (ops: VehicleOperation[]) => void;
  setVehicleWriteOps: (ops: WriteOperation[]) => void;
}

interface UseConnectionEffectsReturn {
  discoveryProgress: number;
  isDiscoveryComplete: boolean;
  hasVinCache: boolean;
  setHasVinCache: (value: boolean) => void;
  setIsDiscoveryComplete: (value: boolean) => void;
}

export function useConnectionEffects(
  status: ConnectionStatus,
  vehicle: VehicleInfo | null,
  vehicleActions: VehicleActions,
  language: string,
  showToast: (message: string) => void,
  t: (key: string) => string,
): UseConnectionEffectsReturn {
  const [discoveryProgress, setDiscoveryProgress] = useState(0);
  const [isDiscoveryComplete, setIsDiscoveryComplete] = useState(false);
  const [hasVinCache, setHasVinCache] = useState(false);
  const discoveryPollRef = useRef<number | null>(null);
  const progressIntervalRef = useRef<number | null>(null);
  const tRef = useRef(t);
  const langRef = useRef(language);

  useEffect(() => {
    tRef.current = t;
    langRef.current = language;
  }, [t, language]);

  const handleClearDiscoveryTimeout = useCallback(() => {
    if (discoveryPollRef.current) clearTimeout(discoveryPollRef.current);
    if (progressIntervalRef.current) clearInterval(progressIntervalRef.current);
  }, []);

  useEffect(() => {
    let cancelled = false;
    devInfo("ui", "Connection: " + status);

    handleClearDiscoveryTimeout();

    if (status === "demo") {
      devInfo("ui", "Demo polling started");
      setIsDiscoveryComplete(true);
      if (!cancelled) invoke("discover_vehicle_params", { manufacturer: "Peugeot", vin: "" }).catch(() => {});
      vehicleActions.startDemoPolling();
    } else if (status === "connected") {
      setIsDiscoveryComplete(false);
      const make = vehicle?.make || "";
      devInfo("ui", "Starting vehicle discovery for " + make);
      vehicleActions.stopPolling();

      // Simulate progressive loading while waiting for discovery
      let simulatedProgress = 5;
      setDiscoveryProgress(simulatedProgress);
      const progressInterval = window.setInterval(() => {
        if (simulatedProgress < 90) {
          simulatedProgress += Math.random() * 8 + 2;
          if (simulatedProgress > 90) simulatedProgress = 90;
          setDiscoveryProgress(Math.round(simulatedProgress));
        }
      }, 600);
      progressIntervalRef.current = progressInterval;

      const bootstrapDiagnostics = async (includeBaseline: boolean): Promise<void> => {
        try {
          if (includeBaseline) {
            const codes = await invoke<DtcCode[]>("read_all_dtcs", { lang: langRef.current });
            if (cancelled) return;
            devInfo("ui", "DTCs loaded: " + codes.length);
            vehicleActions.setDtcs(codes);

            const ecus = await invoke<EcuInfo[]>("scan_ecus", { manufacturer: make });
            if (cancelled) return;
            vehicleActions.setEcus(ecus);

            const monitors = await invoke<MonitorStatus[]>("get_monitors");
            if (cancelled) return;
            vehicleActions.setMonitors(monitors);
          }

          const result = await invoke<{
            standardPids: number;
            manufacturerDids: number;
            fromCache?: boolean;
            complete?: boolean;
            error?: string;
          }>("discover_vehicle_params", { manufacturer: make, vin: vehicle?.vin || "" });
          if (cancelled) return;
          devInfo("ui", `Discovery: ${result.standardPids} PIDs + ${result.manufacturerDids} DIDs`);
          clearInterval(progressInterval);
          progressIntervalRef.current = null;
          const complete = result.complete !== false;
          setDiscoveryProgress(complete ? 100 : 90);
          setIsDiscoveryComplete(complete);
          setHasVinCache(Boolean(result.fromCache || complete));
          if (discoveryPollRef.current) clearTimeout(discoveryPollRef.current);
          if (complete) {
            showToast(tRef.current("connection.analysisComplete"));
          } else {
            devInfo("ui", "Discovery incomplete: " + (result.error || "unknown error"));
            discoveryPollRef.current = window.setTimeout(() => {
              if (!cancelled) void bootstrapDiagnostics(false);
            }, 5_000);
          }
          vehicleActions.startRealPolling(1000, make, true);
        } catch (e) {
          if (cancelled) return;
          devInfo("ui", "Discovery failed: " + String(e));
          clearInterval(progressInterval);
          progressIntervalRef.current = null;
          setDiscoveryProgress(90);
          setIsDiscoveryComplete(false);
          discoveryPollRef.current = window.setTimeout(() => {
            if (!cancelled) void bootstrapDiagnostics(false);
          }, 5_000);
          vehicleActions.startRealPolling(1000, make, true);
        }
      };
      void bootstrapDiagnostics(true);

      const vehicleMake = vehicle?.make || "";
      invoke<VehicleOperation[]>("get_vehicle_operations", { vehicle: vehicleMake, limit: 500 })
        .then(ops => {
          if (cancelled) return;
          devInfo("ui", "Vehicle ops: " + ops.length);
          vehicleActions.setVehicleOps(ops);
        })
        .catch(() => {});
      invoke<WriteOperation[]>("get_write_operations", { ecuName: "%", vehicle: vehicleMake })
        .then(ops => { if (!cancelled) vehicleActions.setVehicleWriteOps(ops); })
        .catch(() => {});
    } else if (status === "disconnected") {
      vehicleActions.stopPolling();
      setIsDiscoveryComplete(false);
      setDiscoveryProgress(0);
      setHasVinCache(false);
      handleClearDiscoveryTimeout();
    }

    return () => {
      cancelled = true;
      handleClearDiscoveryTimeout();
    };
  }, [status, vehicle?.make, vehicle?.vin, vehicleActions, showToast, handleClearDiscoveryTimeout]);

  return {
    discoveryProgress,
    isDiscoveryComplete,
    hasVinCache,
    setHasVinCache,
    setIsDiscoveryComplete,
  };
}
