import { useState, useEffect, useCallback, useRef } from "react";
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

// Demo data generator
function generateDemoData(): Map<number, PidValue> {
  const now = Date.now();
  const baseRpm = 850 + Math.sin(now / 3000) * 200 + Math.random() * 50;
  const baseSpeed = Math.max(0, Math.sin(now / 10000) * 60 + 30 + Math.random() * 5);
  const baseCoolant = 85 + Math.sin(now / 20000) * 8 + Math.random() * 2;
  const baseLoad = 25 + Math.sin(now / 5000) * 15 + Math.random() * 5;

  const pids = new Map<number, PidValue>();

  const addPid = (pid: number, name: string, value: number, unit: string, min: number, max: number) => {
    const existing = demoDataCache.get(pid);
    const history = existing ? [...existing.history.slice(-59), value] : [value];
    pids.set(pid, { pid, name, value, unit, min, max, history, timestamp: now });
  };

  // PID names use neutral/technical labels (backend provides localized names in real mode)
  addPid(0x0c, "RPM", baseRpm, "tr/min", 0, 8000);
  addPid(0x0d, "Speed", baseSpeed, "km/h", 0, 250);
  addPid(0x05, "Coolant Temp", baseCoolant, "°C", -40, 215);
  addPid(0x04, "Engine Load", baseLoad, "%", 0, 100);
  addPid(0x0f, "Intake Temp", 32 + Math.random() * 3, "°C", -40, 215);
  addPid(0x10, "MAF Rate", 5.2 + Math.sin(now / 4000) * 2, "g/s", 0, 655);
  addPid(0x11, "Throttle Pos", 15 + Math.sin(now / 2000) * 8, "%", 0, 100);
  addPid(0x0b, "Intake Pressure", 95 + Math.sin(now / 3000) * 5, "kPa", 0, 255);
  addPid(0x0e, "Timing Advance", 12 + Math.sin(now / 2500) * 4, "°", -64, 63.5);
  addPid(0x2f, "Fuel Level", 65 - (now % 100000) / 100000 * 2, "%", 0, 100);
  addPid(0x42, "Battery Voltage", 14.1 + Math.sin(now / 8000) * 0.3, "V", 0, 65.5);
  addPid(0x46, "Ambient Temp", 22 + Math.random() * 1, "°C", -40, 215);
  addPid(0x33, "Baro Pressure", 101 + Math.random() * 0.5, "kPa", 0, 255);
  addPid(0x06, "STFT Bank 1", 2.3 + Math.random() * 1, "%", -100, 99.2);
  addPid(0x07, "LTFT Bank 1", 4.1 + Math.random() * 0.5, "%", -100, 99.2);
  addPid(0x03, "Fuel Status", 2, "", 0, 16);
  addPid(0x1c, "OBD Standard", 6, "", 0, 255);
  addPid(0x1f, "Run Time", Math.floor((now % 360000) / 1000), "s", 0, 65535);

  return pids;
}

let demoDataCache = new Map<number, PidValue>();

export const demoDtcs: DtcCode[] = [
  {
    code: "P0440",
    description: "Evaporative Emission Control System Malfunction",
    status: "active",
    source: "OBD Mode 03",
    repairTips: "Check fuel cap, EVAP hoses and purge valve.",
    causes: [
      "Loose or missing fuel filler cap",
      "Damaged EVAP hoses or connections",
      "Faulty purge valve",
      "EVAP canister leak",
      "Fuel pump seal leaking"
    ],
    quickCheck: "Start with the fuel cap. Many vehicles throw this code simply due to a loose cap. If tight, inspect hoses for cracks.",
    difficulty: 2,
  },
  {
    code: "P0500",
    description: "Vehicle Speed Sensor Malfunction",
    status: "pending",
    source: "OBD Mode 07",
    repairTips: "Inspect VSS sensor, wiring and connectors.",
    causes: [
      "Defective vehicle speed sensor",
      "Loose or corroded connectors",
      "Broken wiring harness",
      "Transmission issue",
      "ABS sensor malfunction"
    ],
    quickCheck: "Check the VSS sensor located on the transmission. Verify wiring is secure and clean corrosion if needed.",
    difficulty: 2,
  },
];

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
  const [freezeFrame, setFreezeFrame] = useState<FreezeFrameData | null>(null);
  const [isLoadingMode06, setIsLoadingMode06] = useState(false);
  const [isLoadingFreezeFrame, setIsLoadingFreezeFrame] = useState(false);
  const [isPolling, setIsPolling] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const pollingModeRef = useRef<"demo" | "real">("demo");
  const manufacturerRef = useRef<string>("");
  const { i18n } = useTranslation();

  const startDemoPolling = useCallback((intervalMs: number = 500) => {
    devInfo("ui", "Demo polling @ " + intervalMs + " ms");
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "demo";
    manufacturerRef.current = "";
    setIsPolling(true);
    setDtcs(demoDtcs);
    setEcus(demoEcus);
    setMonitors(demoMonitors);

    // Pre-load demo history with past DTCs (resolved codes no longer active)
    setDtcHistory((prev) => {
      const existing = new Set(prev.map((h) => h.code));
      const pastDtcs: DtcHistoryEntry[] = [
        // P0440 was seen 3 days ago (same code as current active — will be filtered out of history display)
        ...(!existing.has("P0440") ? [{
          code: "P0440", description: "Evaporative Emission Control System Malfunction",
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 3 * 86400000,
        }] : []),
        // P0171 was seen 2 weeks ago and is now resolved
        ...(!existing.has("P0171") ? [{
          code: "P0171", description: "System Too Lean (Bank 1)",
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 14 * 86400000,
          repairTips: "Check MAF sensor, vacuum leaks, fuel pressure",
        }] : []),
        // P0300 was seen 1 month ago and is now resolved
        ...(!existing.has("P0300") ? [{
          code: "P0300", description: "Random/Multiple Cylinder Misfire Detected",
          status: "active" as const, source: "OBD Mode 03", seenAt: Date.now() - 30 * 86400000,
          repairTips: "Check spark plugs, ignition coils, fuel injectors",
        }] : []),
      ];
      return [...prev, ...pastDtcs];
    });

    intervalRef.current = window.setInterval(() => {
      const data = generateDemoData();
      demoDataCache = data;
      setPidData(new Map(data));
    }, intervalMs);
  }, []);

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
    if (isLoadingMode06) return;
    setIsLoadingMode06(true);
    try {
      const results = await invoke<Mode06Result[]>("get_mode06_results", { lang: i18n.language });
      setMode06Results(results);
    } catch (e) {
      devInfo("ui", "Mode 06 error: " + String(e));
    } finally {
      setIsLoadingMode06(false);
    }
  }, [isLoadingMode06, i18n.language]);

  const loadFreezeFrame = useCallback(async () => {
    if (isLoadingFreezeFrame) return;
    setIsLoadingFreezeFrame(true);
    try {
      const data = await invoke<FreezeFrameData | null>("get_freeze_frame", { lang: i18n.language });
      setFreezeFrame(data);
    } catch (e) {
      devInfo("ui", "Freeze frame error: " + String(e));
    } finally {
      setIsLoadingFreezeFrame(false);
    }
  }, [isLoadingFreezeFrame, i18n.language]);

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
    setFreezeFrame(null);
    try { localStorage.removeItem("bricarobd_dtc_history"); } catch {}
  }, []);

  const changeRefreshRate = useCallback((intervalMs: number) => {
    devInfo("ui", "Refresh rate: " + intervalMs + " ms");
    if (intervalRef.current) {
      clearInterval(intervalRef.current);

      if (pollingModeRef.current === "real") {
        const pollFn = createRealPollFn(manufacturerRef.current, setPidData);
        intervalRef.current = window.setInterval(pollFn, intervalMs);
      } else {
        intervalRef.current = window.setInterval(() => {
          const data = generateDemoData();
          demoDataCache = data;
          setPidData(new Map(data));
        }, intervalMs);
      }
    }
  }, []);

  useEffect(() => {
    return () => {
      if (intervalRef.current) clearInterval(intervalRef.current);
    };
  }, []);

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
        // Persist to localStorage
        try { localStorage.setItem("bricarobd_dtc_history", JSON.stringify(updated)); } catch {}
        return updated;
      });
    }
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
