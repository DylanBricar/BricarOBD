import { useState, useEffect, useCallback, useMemo, useRef } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { devInfo, devDebug } from "@/lib/devlog";
import type { PidValue, DtcCode, DtcHistoryEntry, EcuInfo, MonitorStatus, Mode06Result, FreezeFrameData, VehicleOperation, WriteOperation } from "./vehicleTypes";
import { buildDemoDtcs, demoMonitors, buildDemoEcus } from "./vehicleDemoData";

export type { PidValue, DtcCode, DtcHistoryEntry, EcuInfo, MonitorStatus, Mode06Result, FreezeFrameData, VehicleOperation, WriteOperation } from "./vehicleTypes";

function createRealPollFn(
  manufacturer: string,
  setPidData: React.Dispatch<React.SetStateAction<Map<number, PidValue>>>,
  pollInProgressRef: React.MutableRefObject<boolean>,
) {
  return async () => {
    if (pollInProgressRef.current) return;
    pollInProgressRef.current = true;
    try {
      const cmd = manufacturer ? "get_pid_data_extended" : "get_pid_data";
      const args = manufacturer ? { manufacturer } : {};
      const pids = await invoke<PidValue[]>(cmd, args);
      if (pids.length === 0) return;
      setPidData(prev => {
        const merged = new Map(prev);
        for (const p of pids) merged.set(p.pid, p);
        return merged;
      });
    } catch (e) {
      devDebug("ui", `Poll error: ${String(e)}`);
    } finally {
      pollInProgressRef.current = false;
    }
  };
}

// External store for vehicle state (survives component unmounts)
interface VehicleState {
  pidData: Map<number, PidValue>;
  dtcs: DtcCode[];
  dtcHistory: DtcHistoryEntry[];
  ecus: EcuInfo[];
  monitors: MonitorStatus[];
  mode06Results: Mode06Result[];
  freezeFrame: FreezeFrameData[];
  isLoadingMode06: boolean;
  isLoadingFreezeFrame: boolean;
  isPolling: boolean;
  vehicleOps: VehicleOperation[];
  vehicleWriteOps: WriteOperation[];
  dbStats: { operations: number; profiles: number; ecus: number } | null;
}

const initialVehicleState: VehicleState = {
  pidData: new Map(),
  dtcs: [],
  dtcHistory: (() => {
    try {
      const saved = localStorage.getItem("bricarobd_dtc_history");
      return saved ? JSON.parse(saved) : [];
    } catch { return []; }
  })(),
  ecus: [],
  monitors: [],
  mode06Results: [],
  freezeFrame: [],
  isLoadingMode06: false,
  isLoadingFreezeFrame: false,
  isPolling: false,
  vehicleOps: [],
  vehicleWriteOps: [],
  dbStats: null,
};

let vehicleState: VehicleState = { ...initialVehicleState };
const vehicleListeners = new Set<() => void>();

function emitVehicleChange() {
  vehicleListeners.forEach((l) => l());
}

export function useVehicleData() {
  const [pidData, setPidData] = useState<Map<number, PidValue>>(new Map());
  const [dtcs, setDtcs] = useState<DtcCode[]>([]);
  const [dtcHistory, setDtcHistory] = useState<DtcHistoryEntry[]>(() => {
    try {
      const saved = localStorage.getItem("bricarobd_dtc_history");
      return saved ? JSON.parse(saved) : [];
    } catch { return []; }
  });
  const [ecus, setEcus] = useState<EcuInfo[]>([]);
  const [monitors, setMonitors] = useState<MonitorStatus[]>([]);
  const [mode06Results, setMode06Results] = useState<Mode06Result[]>([]);
  const [freezeFrame, setFreezeFrame] = useState<FreezeFrameData[]>([]);
  const [isLoadingMode06, setIsLoadingMode06] = useState(false);
  const [isLoadingFreezeFrame, setIsLoadingFreezeFrame] = useState(false);
  const [isPolling, setIsPolling] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const pollingModeRef = useRef<"demo" | "real">("demo");
  const manufacturerRef = useRef("");
  const pollInProgressRef = useRef(false);
  const isLoadingMode06Ref = useRef(false);
  const isLoadingFreezeFrameRef = useRef(false);
  const { t, i18n } = useTranslation();

  const demoDtcs = useMemo(() => buildDemoDtcs(t), [t]);
  const demoEcus = useMemo(() => buildDemoEcus(t), [t]);

  const setDtcsWithHistory = useCallback((newDtcs: DtcCode[]) => {
    if (newDtcs.length === 0) return;
    devInfo("ui", "DTCs updated: " + newDtcs.length);
    setDtcs(newDtcs);
    vehicleState.dtcs = newDtcs;
    setDtcHistory((prev) => {
      const now = Date.now();
      const codes = new Set(prev.map((h) => h.code));
      const newEntries = newDtcs
        .filter((d) => !codes.has(d.code))
        .map((d) => ({ ...d, seenAt: now }));
      const updated = [...prev, ...newEntries].slice(-500);
      try { localStorage.setItem("bricarobd_dtc_history", JSON.stringify(updated)); } catch {}
      vehicleState.dtcHistory = updated;
      emitVehicleChange();
      return updated;
    });
    emitVehicleChange();
  }, []);

  const startDemoPolling = useCallback((intervalMs: number = 500) => {
    devInfo("ui", "Demo polling @ " + intervalMs + " ms");
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "demo";
    manufacturerRef.current = "";
    setIsPolling(true);
    vehicleState.isPolling = true;
    setDtcsWithHistory(demoDtcs);
    setEcus(demoEcus);
    vehicleState.ecus = demoEcus;
    setMonitors(demoMonitors);
    vehicleState.monitors = demoMonitors;

    // Pre-load demo history with past DTCs (resolved codes no longer active)
    setDtcHistory((prev) => {
      const existing = new Set(prev.map((h) => h.code));
      const pastDtcs: DtcHistoryEntry[] = [
        ...(!existing.has("P0440") ? [{
          code: "P0440", description: t("demo.dtc.P0440"),
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 3 * 86400000,
        }] : []),
        ...(!existing.has("P0171") ? [{
          code: "P0171", description: t("demo.dtc.P0171"),
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 14 * 86400000,
          repairTips: t("demo.dtc.P0171.tips"),
        }] : []),
        ...(!existing.has("P0300") ? [{
          code: "P0300", description: t("demo.dtc.P0300"),
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 30 * 86400000,
          repairTips: t("demo.dtc.P0300.tips"),
        }] : []),
      ];
      const result = [...prev, ...pastDtcs];
      vehicleState.dtcHistory = result;
      emitVehicleChange();
      return result;
    });

    const pollFn = async () => {
      try {
        const data = await invoke<PidValue[]>("get_pid_data");
        setPidData(prev => {
          const merged = new Map(prev);
          for (const p of data) merged.set(p.pid, p);
          vehicleState.pidData = new Map(merged);
          emitVehicleChange();
          return merged;
        });
      } catch (e) {
        devDebug("ui", `Demo poll error: ${String(e)}`);
      }
    };

    pollFn(); // First poll immediately
    intervalRef.current = window.setInterval(pollFn, intervalMs);
    emitVehicleChange();
  }, [setDtcsWithHistory, demoDtcs, t]);

  const startRealPolling = useCallback((intervalMs: number = 1000, manufacturer: string = "", skipEcuScan: boolean = false) => {
    devInfo("ui", "Real polling @ " + intervalMs + " ms for " + manufacturer);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "real";
    manufacturerRef.current = manufacturer;
    setIsPolling(true);
    vehicleState.isPolling = true;

    // Reset PID failure blacklist on new connection
    invoke("reset_pid_blacklist").catch(() => {});

    // Load ECUs and monitors from backend (skip on VIN update to avoid 60s rescan)
    if (!skipEcuScan) {
      invoke<EcuInfo[]>("scan_ecus").then(e => { setEcus(e); vehicleState.ecus = e; emitVehicleChange(); }).catch(() => {});
      invoke<MonitorStatus[]>("get_monitors").then(m => { setMonitors(m); vehicleState.monitors = m; emitVehicleChange(); }).catch(() => {});
    }

    // Use shared poll function
    const pollFn = createRealPollFn(manufacturer, setPidData, pollInProgressRef);

    pollFn(); // First poll immediately
    intervalRef.current = window.setInterval(pollFn, intervalMs);
    emitVehicleChange();
  }, [pollInProgressRef]);

  const loadMonitors = useCallback(async () => {
    try {
      const monitors = await invoke<MonitorStatus[]>("get_monitors");
      setMonitors(monitors);
      vehicleState.monitors = monitors;
      emitVehicleChange();
    } catch {}
  }, []);

  const loadMode06Results = useCallback(async () => {
    if (isLoadingMode06Ref.current) return;
    isLoadingMode06Ref.current = true;
    setIsLoadingMode06(true);
    try {
      const results = await invoke<Mode06Result[]>("get_mode06_results", { lang: i18n.language });
      setMode06Results(results);
      vehicleState.mode06Results = results;
      emitVehicleChange();
    } catch (e) {
      devInfo("ui", "Mode 06 error: " + String(e));
    } finally {
      isLoadingMode06Ref.current = false;
      setIsLoadingMode06(false);
    }
  }, [i18n.language]);

  const loadFreezeFrame = useCallback(async () => {
    if (isLoadingFreezeFrameRef.current) return;
    isLoadingFreezeFrameRef.current = true;
    setIsLoadingFreezeFrame(true);
    try {
      const data = await invoke<FreezeFrameData[]>("get_freeze_frame", { lang: i18n.language });
      setFreezeFrame(data);
      vehicleState.freezeFrame = data;
      emitVehicleChange();
    } catch (e) {
      devInfo("ui", "Freeze frame error: " + String(e));
    } finally {
      isLoadingFreezeFrameRef.current = false;
      setIsLoadingFreezeFrame(false);
    }
  }, [i18n.language]);

  const pausePolling = useCallback(() => {
    devInfo("ui", "Polling paused");
    setIsPolling(false);
    vehicleState.isPolling = false;
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    emitVehicleChange();
  }, []);

  const stopPolling = useCallback(() => {
    devInfo("ui", "Polling stopped — clearing all vehicle data");
    setIsPolling(false);
    vehicleState.isPolling = false;
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    // Clear ALL vehicle data on disconnect — UI must match reality
    setPidData(new Map());
    vehicleState.pidData = new Map();
    setDtcs([]);
    vehicleState.dtcs = [];
    setEcus([]);
    vehicleState.ecus = [];
    setMonitors([]);
    vehicleState.monitors = [];
    setMode06Results([]);
    vehicleState.mode06Results = [];
    setFreezeFrame([]);
    vehicleState.freezeFrame = [];
    emitVehicleChange();
  }, []);

  const changeRefreshRate = useCallback((intervalMs: number) => {
    devInfo("ui", "Refresh rate: " + intervalMs + " ms");
    // Always clear previous interval first to prevent stacking
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }

    if (pollingModeRef.current !== "demo" && pollingModeRef.current !== "real") return;

    if (pollingModeRef.current === "real") {
      const pollFn = createRealPollFn(manufacturerRef.current, setPidData, pollInProgressRef);
      intervalRef.current = window.setInterval(pollFn, intervalMs);
    } else {
      const pollFn = async () => {
        try {
          const data = await invoke<PidValue[]>("get_pid_data");
          setPidData(prev => {
            const merged = new Map(prev);
            for (const p of data) merged.set(p.pid, p);
            vehicleState.pidData = new Map(merged);
            emitVehicleChange();
            return merged;
          });
        } catch (e) {
          devDebug("ui", `Demo poll error: ${String(e)}`);
        }
      };
      intervalRef.current = window.setInterval(pollFn, intervalMs);
    }
  }, []);

  useEffect(() => {
    isLoadingMode06Ref.current = isLoadingMode06;
  }, [isLoadingMode06]);

  useEffect(() => {
    isLoadingFreezeFrameRef.current = isLoadingFreezeFrame;
  }, [isLoadingFreezeFrame]);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

  const clearAllDtcs = useCallback(() => {
    devInfo("ui", "DTCs cleared");
    setDtcs([]);
  }, []);

  // Vehicle-specific operations from the 3.17M DB
  const [vehicleOps, setVehicleOps] = useState<VehicleOperation[]>([]);
  const [vehicleWriteOps, setVehicleWriteOps] = useState<WriteOperation[]>([]);
  const [dbStats, setDbStats] = useState<{ operations: number; profiles: number; ecus: number } | null>(null);

  return {
    pidData,
    dtcs,
    dtcHistory,
    isPolling,
    ecus,
    monitors,
    mode06Results,
    freezeFrame,
    isLoadingMode06,
    isLoadingFreezeFrame,
    vehicleOps,
    vehicleWriteOps,
    dbStats,
    startDemoPolling,
    startRealPolling,
    loadMonitors,
    loadMode06Results,
    loadFreezeFrame,
    pausePolling,
    stopPolling,
    changeRefreshRate,
    setDtcs: setDtcsWithHistory,
    clearAllDtcs,
    setEcus: (ecus: EcuInfo[]) => { setEcus(ecus); vehicleState.ecus = ecus; emitVehicleChange(); },
    setMonitors: (monitors: MonitorStatus[]) => { setMonitors(monitors); vehicleState.monitors = monitors; emitVehicleChange(); },
    setVehicleOps: (ops: VehicleOperation[]) => { setVehicleOps(ops); vehicleState.vehicleOps = ops; emitVehicleChange(); },
    setVehicleWriteOps: (ops: WriteOperation[]) => { setVehicleWriteOps(ops); vehicleState.vehicleWriteOps = ops; emitVehicleChange(); },
    setDbStats: (stats: { operations: number; profiles: number; ecus: number } | null) => { setDbStats(stats); vehicleState.dbStats = stats; emitVehicleChange(); },
  };
}
