import { useState, useEffect, useCallback, useRef } from "react";
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

  addPid(0x0c, "RPM", baseRpm, "tr/min", 0, 8000);
  addPid(0x0d, "Vitesse", baseSpeed, "km/h", 0, 250);
  addPid(0x05, "Temp. liquide refroid.", baseCoolant, "°C", -40, 215);
  addPid(0x04, "Charge moteur", baseLoad, "%", 0, 100);
  addPid(0x0f, "Temp. admission", 32 + Math.random() * 3, "°C", -40, 215);
  addPid(0x10, "Débit air (MAF)", 5.2 + Math.sin(now / 4000) * 2, "g/s", 0, 655);
  addPid(0x11, "Position papillon", 15 + Math.sin(now / 2000) * 8, "%", 0, 100);
  addPid(0x0b, "Pression admission", 95 + Math.sin(now / 3000) * 5, "kPa", 0, 255);
  addPid(0x0e, "Avance allumage", 12 + Math.sin(now / 2500) * 4, "°", -64, 63.5);
  addPid(0x2f, "Niveau carburant", 65 - (now % 100000) / 100000 * 2, "%", 0, 100);
  addPid(0x42, "Tension batterie", 14.1 + Math.sin(now / 8000) * 0.3, "V", 0, 65.5);
  addPid(0x46, "Temp. ambiante", 22 + Math.random() * 1, "°C", -40, 215);
  addPid(0x33, "Pression atmos.", 101 + Math.random() * 0.5, "kPa", 0, 255);
  addPid(0x06, "Correctif carburant CT", 2.3 + Math.random() * 1, "%", -100, 99.2);
  addPid(0x07, "Correctif carburant LT", 4.1 + Math.random() * 0.5, "%", -100, 99.2);
  addPid(0x03, "Statut carburant", 2, "", 0, 16);
  addPid(0x1c, "Standard OBD", 6, "", 0, 255);
  addPid(0x1f, "Durée moteur", Math.floor((now % 360000) / 1000), "s", 0, 65535);

  return pids;
}

let demoDataCache = new Map<number, PidValue>();

export const demoDtcs: DtcCode[] = [
  {
    code: "P0440",
    description: "Système de contrôle des émissions par évaporation - Dysfonctionnement",
    status: "active",
    source: "OBD Mode 03",
    repairTips: "Vérifier le bouchon de réservoir, les durites EVAP et la valve de purge.",
  },
  {
    code: "P0500",
    description: "Capteur de vitesse du véhicule - Dysfonctionnement",
    status: "pending",
    source: "OBD Mode 07",
    repairTips: "Inspecter le capteur VSS, le câblage et les connecteurs.",
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
    name: "Moteur (ECM)",
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
    name: "BSI (Boîtier Servitudes)",
    address: "0x75D",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F18C": "BSI 2010", "F191": "BSI HW 1.5", "F195": "BSI SW 6.2", "F187": "BSI-96xxxxx", "F17E": "PSA-BSI-2018" },
  },
  {
    name: "Climatisation (HVAC)",
    address: "0x7E6",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F195": "HVAC 1.0" },
  },
  {
    name: "Tableau de bord",
    address: "0x7E5",
    protocol: "ISO 15765-4 CAN",
    dids: { "F190": "VF3LCBHZ6JS000000", "F195": "CLUST 2.3", "F18C": "2018-03-01" },
  },
];

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
  const [isPolling, setIsPolling] = useState(false);
  const intervalRef = useRef<number | null>(null);
  const pollingModeRef = useRef<"demo" | "real">("demo");
  const manufacturerRef = useRef<string>("");

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

    intervalRef.current = window.setInterval(() => {
      const data = generateDemoData();
      demoDataCache = data;
      setPidData(new Map(data));
    }, intervalMs);
  }, []);

  const startRealPolling = useCallback((intervalMs: number = 1000, manufacturer: string = "") => {
    devInfo("ui", "Real polling @ " + intervalMs + " ms for " + manufacturer);
    if (intervalRef.current) {
      clearInterval(intervalRef.current);
    }
    pollingModeRef.current = "real";
    manufacturerRef.current = manufacturer;
    setIsPolling(true);

    // Load ECUs and monitors from backend
    invoke<EcuInfo[]>("scan_ecus").then(setEcus).catch(() => {});
    invoke<MonitorStatus[]>("get_monitors").then(setMonitors).catch(() => {});

    // Poll PIDs at interval via backend
    const pollFn = async () => {
      try {
        const cmd = manufacturer ? "get_pid_data_extended" : "get_pid_data";
        const args = manufacturer ? { manufacturer } : {};
        const pids = await invoke<PidValue[]>(cmd, args);
        const map = new Map<number, PidValue>();
        for (const p of pids) {
          map.set(p.pid, p);
        }
        setPidData(map);
      } catch (e) {
        devDebug("ui", "Poll error: " + String(e));
      }
    };

    pollFn(); // First poll immediately
    intervalRef.current = window.setInterval(pollFn, intervalMs);
  }, []);

  const loadMonitors = useCallback(async () => {
    try {
      const monitors = await invoke<MonitorStatus[]>("get_monitors");
      setMonitors(monitors);
    } catch {}
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
    try { localStorage.removeItem("bricarobd_dtc_history"); } catch {}
  }, []);

  const changeRefreshRate = useCallback((intervalMs: number) => {
    devInfo("ui", "Refresh rate: " + intervalMs + " ms");
    if (isPolling && intervalRef.current) {
      clearInterval(intervalRef.current);

      if (pollingModeRef.current === "real") {
        // Real mode: re-poll via backend
        const mfr = manufacturerRef.current;
        const pollFn = async () => {
          try {
            const cmd = mfr ? "get_pid_data_extended" : "get_pid_data";
            const args = mfr ? { manufacturer: mfr } : {};
            const pids = await invoke<PidValue[]>(cmd, args);
            const map = new Map<number, PidValue>();
            for (const p of pids) map.set(p.pid, p);
            setPidData(map);
          } catch {}
        };
        intervalRef.current = window.setInterval(pollFn, intervalMs);
      } else {
        // Demo mode: use JS generator
        intervalRef.current = window.setInterval(() => {
          const data = generateDemoData();
          demoDataCache = data;
          setPidData(new Map(data));
        }, intervalMs);
      }
    }
  }, [isPolling]);

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
        const updated = [...prev, ...newEntries];
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
  const [vehicleOps, setVehicleOps] = useState<any[]>([]);
  const [vehicleWriteOps, setVehicleWriteOps] = useState<any[]>([]);
  const [dbStats, setDbStats] = useState<{ operations: number; profiles: number; ecus: number } | null>(null);

  return {
    pidData,
    dtcs,
    dtcHistory,
    isPolling,
    ecus,
    monitors,
    vehicleOps,
    vehicleWriteOps,
    dbStats,
    startDemoPolling,
    startRealPolling,
    loadMonitors,
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
