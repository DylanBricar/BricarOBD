import { useState, useEffect, useCallback, useRef, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { invoke } from "@tauri-apps/api/core";
import { devInfo, devDebug } from "@/lib/devlog";

export interface PidValue {
  pid: number;
  name: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  history: number[];
  timestamp: number;
}

export interface DtcCode {
  code: string;
  description: string;
  status: "active" | "pending" | "permanent";
  source: string;
  repairTips?: string;
  causes?: string[];
  quickCheck?: string;
  difficulty?: number;
  ecuContext?: string;
}

export interface DtcHistoryEntry extends DtcCode {
  seenAt: number;
}

export interface EcuInfo {
  name: string;
  address: string;
  protocol: string;
  dids: Record<string, string>;
}

export interface MonitorStatus {
  nameKey: string;
  available: boolean;
  complete: boolean;
  descriptionKey?: string;
  specificationKey?: string;
}

export interface Mode06Result {
  tid: number;
  mid: number;
  name: string;
  unit: string;
  testValue: number;
  minLimit: number;
  maxLimit: number;
  passed: boolean;
}

export interface FreezeFrameData {
  dtcCode: string;
  frameNumber: number;
  pids: PidValue[];
}

export interface VehicleOperation {
  [key: string]: any;
}

export interface WriteOperation {
  [key: string]: any;
}

// Demo DTC keys — descriptions use i18n to avoid hardcoded English
const DEMO_DTC_KEYS = [
  {
    code: "P0440",
    descKey: "demo.dtc.P0440",
    status: "active" as const,
    source: "OBD Mode 03",
    tipsKey: "demo.dtc.P0440.tips",
    causeKeys: ["demo.dtc.P0440.cause1", "demo.dtc.P0440.cause2", "demo.dtc.P0440.cause3", "demo.dtc.P0440.cause4", "demo.dtc.P0440.cause5"],
    quickCheckKey: "demo.dtc.P0440.quickCheck",
    difficulty: 2,
  },
  {
    code: "P0500",
    descKey: "demo.dtc.P0500",
    status: "pending" as const,
    source: "OBD Mode 07",
    tipsKey: "demo.dtc.P0500.tips",
    causeKeys: ["demo.dtc.P0500.cause1", "demo.dtc.P0500.cause2", "demo.dtc.P0500.cause3", "demo.dtc.P0500.cause4", "demo.dtc.P0500.cause5"],
    quickCheckKey: "demo.dtc.P0500.quickCheck",
    difficulty: 2,
  },
];

function buildDemoDtcs(t: (key: string) => string): DtcCode[] {
  return DEMO_DTC_KEYS.map((d) => ({
    code: d.code,
    description: t(d.descKey),
    status: d.status,
    source: d.source,
    repairTips: t(d.tipsKey),
    causes: d.causeKeys.map((k) => t(k)),
    quickCheck: t(d.quickCheckKey),
    difficulty: d.difficulty,
  }));
}

const demoMonitors: MonitorStatus[] = [
  { nameKey: "monitors.misfire", available: true, complete: true, descriptionKey: "monitors.misfireDesc", specificationKey: "monitors.misfireSpec" },
  { nameKey: "monitors.fuelSystem", available: true, complete: true, descriptionKey: "monitors.fuelSystemDesc", specificationKey: "monitors.fuelSystemSpec" },
  { nameKey: "monitors.components", available: true, complete: true, descriptionKey: "monitors.componentsDesc", specificationKey: "monitors.componentsSpec" },
  { nameKey: "monitors.catalystB1", available: true, complete: false, descriptionKey: "monitors.catalystB1Desc", specificationKey: "monitors.catalystB1Spec" },
  { nameKey: "monitors.catalystB2", available: false, complete: false, descriptionKey: "monitors.catalystB2Desc", specificationKey: "monitors.catalystB2Spec" },
  { nameKey: "monitors.evap", available: true, complete: false, descriptionKey: "monitors.evapDesc", specificationKey: "monitors.evapSpec" },
  { nameKey: "monitors.o2B1S1", available: true, complete: true, descriptionKey: "monitors.o2B1S1Desc", specificationKey: "monitors.o2B1S1Spec" },
  { nameKey: "monitors.o2HeaterB1S1", available: true, complete: true, descriptionKey: "monitors.o2HeaterB1S1Desc", specificationKey: "monitors.o2HeaterB1S1Spec" },
  { nameKey: "monitors.secondaryAir", available: false, complete: false, descriptionKey: "monitors.secondaryAirDesc", specificationKey: "monitors.secondaryAirSpec" },
  { nameKey: "monitors.ac", available: false, complete: false, descriptionKey: "monitors.acDesc", specificationKey: "monitors.acSpec" },
  { nameKey: "monitors.egrVvt", available: true, complete: true, descriptionKey: "monitors.egrVvtDesc", specificationKey: "monitors.egrVvtSpec" },
];

const demoEcus: EcuInfo[] = [
  {
    name: "Engine (ECM)",
    address: "0x7E0",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F191": "HW 2.3", "F194": "EP6DT", "F195": "1.6 THP 150", "F18C": "2018-03-15", "F187": "PSA 9807654321", "F189": "SW 4.1.2", "F17E": "ECM-PSA-2018" },
  },
  {
    name: "Transmission (TCM)",
    address: "0x7E1",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F191": "AL4/DP0", "F195": "TCM 3.0.1", "F18C": "2018-02-20" },
  },
  {
    name: "ABS/ESP",
    address: "0x7E2",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F191": "MK60 v3", "F195": "ABS 2.1.0", "F18C": "2018-01-10" },
  },
  {
    name: "Airbag (SRS)",
    address: "0x7E3",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F191": "ACU4 v2", "F18C": "2018-04-01" },
  },
  {
    name: "BSI (Body Systems Interface)",
    address: "0x75D",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F18C": "BSI 2010", "F191": "BSI HW 1.5", "F195": "BSI SW 6.2", "F187": "BSI-96xxxxx", "F17E": "PSA-BSI-2018" },
  },
  {
    name: "HVAC",
    address: "0x7E6",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F195": "HVAC 1.0" },
  },
  {
    name: "Instrument Cluster",
    address: "0x7E5",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F195": "CLUST 2.3", "F18C": "2018-03-01" },
  },
];

// Shared real-mode poll function factory — avoids duplication between startRealPolling and changeRefreshRate
function createRealPollFn(
  manufacturer: string,
  setPidData: React.Dispatch<React.SetStateAction<Map<number, PidValue>>>,
) {
  return async () => {
    try {
      const cmd = manufacturer ? "get_pid_data_extended" : "get_pid_data";
      const args = manufacturer ? { manufacturer } : {};
      const pids = await invoke<PidValue[]>(cmd, args);
      const map = new Map<number, PidValue>();
      for (const p of pids) map.set(p.pid, p);
      setPidData(map);
    } catch (e) {
      devDebug("ui", `Poll error: ${String(e)}`);
    }
  };
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
  const manufacturerRef = useRef<string>("");
  const isLoadingMode06Ref = useRef(false);
  const isLoadingFreezeFrameRef = useRef(false);
  const { i18n } = useTranslation();

  const { t } = useTranslation();

  const demoDtcs = useMemo(() => buildDemoDtcs(t), [t]);

  const setDtcsWithHistory = useCallback((newDtcs: DtcCode[]) => {
    devInfo("ui", "DTCs updated: " + newDtcs.length);
    setDtcs(newDtcs);
    if (newDtcs.length > 0) {
      setDtcHistory((prev) => {
        const now = Date.now();
        const codes = new Set(prev.map((h) => h.code));
        const newEntries = newDtcs
          .filter((d) => !codes.has(d.code))
          .map((d) => ({ ...d, seenAt: now }));
        const updated = [...prev, ...newEntries].slice(-500);
        try { localStorage.setItem("bricarobd_dtc_history", JSON.stringify(updated)); } catch {}
        return updated;
      });
    }
  }, []);

  const startDemoPolling = useCallback((intervalMs: number = 500) => {
    devInfo("ui", "Demo polling @ " + intervalMs + " ms");
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "demo";
    manufacturerRef.current = "";
    setIsPolling(true);
    setDtcsWithHistory(demoDtcs);
    setEcus(demoEcus);
    setMonitors(demoMonitors);

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
      return [...prev, ...pastDtcs];
    });

    const pollFn = async () => {
      try {
        const data = await invoke<PidValue[]>("get_pid_data");
        const map = new Map<number, PidValue>();
        data.forEach(p => map.set(p.pid, p));
        setPidData(map);
      } catch (e) {
        devDebug("ui", `Demo poll error: ${String(e)}`);
      }
    };

    pollFn(); // First poll immediately
    intervalRef.current = window.setInterval(pollFn, intervalMs);
  }, [setDtcsWithHistory, demoDtcs, t]);

  const startRealPolling = useCallback((intervalMs: number = 1000, manufacturer: string = "", skipEcuScan: boolean = false) => {
    devInfo("ui", "Real polling @ " + intervalMs + " ms for " + manufacturer);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "real";
    manufacturerRef.current = manufacturer;
    setIsPolling(true);

    // Reset PID failure blacklist on new connection
    invoke("reset_pid_blacklist").catch(() => {});

    // Load ECUs and monitors from backend (skip on VIN update to avoid 60s rescan)
    if (!skipEcuScan) {
      invoke<EcuInfo[]>("scan_ecus").then(setEcus).catch(() => {});
      invoke<MonitorStatus[]>("get_monitors").then(setMonitors).catch(() => {});
    }

    // Use shared poll function
    const pollFn = createRealPollFn(manufacturer, setPidData);

    pollFn(); // First poll immediately
    intervalRef.current = window.setInterval(pollFn, intervalMs);
  }, []);

  const loadMonitors = useCallback(async () => {
    try {
      const monitors = await invoke<MonitorStatus[]>("get_monitors");
      setMonitors(monitors);
    } catch {}
  }, []);

  const loadMode06Results = useCallback(async () => {
    if (isLoadingMode06Ref.current) return;
    isLoadingMode06Ref.current = true;
    setIsLoadingMode06(true);
    try {
      const results = await invoke<Mode06Result[]>("get_mode06_results", { lang: i18n.language });
      setMode06Results(results);
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
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
  }, []);

  const stopPolling = useCallback(() => {
    devInfo("ui", "Polling stopped — clearing all vehicle data");
    setIsPolling(false);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
      intervalRef.current = null;
    }
    // Clear ALL vehicle data on disconnect — UI must match reality
    setPidData(new Map());
    setDtcs([]);
    setDtcHistory([]);
    setEcus([]);
    setMonitors([]);
    setMode06Results([]);
    setFreezeFrame([]);
    try { localStorage.removeItem("bricarobd_dtc_history"); } catch {}
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
      const pollFn = createRealPollFn(manufacturerRef.current, setPidData);
      intervalRef.current = window.setInterval(pollFn, intervalMs);
    } else {
      const pollFn = async () => {
        try {
          const data = await invoke<PidValue[]>("get_pid_data");
          const map = new Map<number, PidValue>();
          data.forEach(p => map.set(p.pid, p));
          setPidData(map);
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
    devInfo("ui", "DTCs + history cleared");
    setDtcs([]);
    setDtcHistory([]);
    try { localStorage.removeItem("bricarobd_dtc_history"); } catch {}
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
    setEcus,
    setMonitors,
    setVehicleOps,
    setVehicleWriteOps,
    setDbStats,
  };
}
