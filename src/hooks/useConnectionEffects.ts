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

  const handleClearDiscoveryTimeout = useCallback(() => {
    if (discoveryPollRef.current) clearTimeout(discoveryPollRef.current);
    if (progressIntervalRef.current) clearInterval(progressIntervalRef.current);
  }, []);

  useEffect(() => {
    let cancelled = false;
    devInfo("ui", "Connection: " + status);

    if (status === "demo") {
      devInfo("ui", "Demo polling started");
      setIsDiscoveryComplete(true);
      if (!cancelled) invoke("discover_vehicle_params", { manufacturer: "Peugeot", vin: "" }).catch(() => {});
      vehicleActions.startDemoPolling();
    } else if (status === "connected") {
      setIsDiscoveryComplete(false);
      const make = vehicle?.make || "";
      devInfo("ui", "Starting vehicle discovery for " + make);

      vehicleActions.startRealPolling(1000, make, true);

      // Sequence: DTCs first, then ECU scan + monitors (backend waits via acquire_with_wait)
      invoke<DtcCode[]>("read_all_dtcs", { lang: language })
        .then(codes => {
          if (cancelled) return;
          devInfo("ui", "DTCs loaded: " + codes.length);
          vehicleActions.setDtcs(codes);
        })
        .catch(() => {})
        .finally(() => {
          if (cancelled) return;
          // Start ECU scan after DTCs release the OBD lock, then monitors after scan finishes
          // Monitors must wait — scan changes ELM327 headers which breaks 0101 broadcast
          invoke<EcuInfo[]>("scan_ecus")
            .then(ecus => { if (!cancelled) vehicleActions.setEcus(ecus); })
            .catch(() => {})
            .finally(() => {
              if (cancelled) return;
              invoke<MonitorStatus[]>("get_monitors").then(m => { if (!cancelled) vehicleActions.setMonitors(m); }).catch(() => {});
            });
        });

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

      const pollDiscoveryProgress = async () => {
        try {
          const result = await invoke<{ standardPids: number; manufacturerDids: number; fromCache?: boolean }>("discover_vehicle_params", { manufacturer: make, vin: vehicle?.vin || "" });
          if (cancelled) return;
          devInfo("ui", `Discovery: ${result.standardPids} PIDs + ${result.manufacturerDids} DIDs`);
          clearInterval(progressInterval);
          progressIntervalRef.current = null;
          setDiscoveryProgress(100);
          setIsDiscoveryComplete(true);
          setHasVinCache(true);
          if (discoveryPollRef.current) clearTimeout(discoveryPollRef.current);
          showToast(t("connection.analysisComplete"));
        } catch (e) {
          if (cancelled) return;
          devInfo("ui", "Discovery failed: " + String(e));
          clearInterval(progressInterval);
          progressIntervalRef.current = null;
          setDiscoveryProgress(100);
          setIsDiscoveryComplete(true);
        }
      };
      pollDiscoveryProgress();

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
  }, [status, vehicle?.make, vehicleActions, language, showToast, t, handleClearDiscoveryTimeout]);

  return {
    discoveryProgress,
    isDiscoveryComplete,
    hasVinCache,
    setHasVinCache,
    setIsDiscoveryComplete,
  };
}
