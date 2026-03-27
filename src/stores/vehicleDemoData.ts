import type { DtcCode, EcuInfo, MonitorStatus } from "./vehicleTypes";

export const DEMO_DTC_KEYS = [
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

export function buildDemoDtcs(t: (key: string) => string): DtcCode[] {
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

export const demoMonitors: MonitorStatus[] = [
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

export const DEMO_ECU_KEYS: Array<{ nameKey: string; address: string; protocol: string; dids: Record<string, string> }> = [
  { nameKey: "demo.ecu.engine", address: "0x7E0", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F191": "HW 2.3", "F194": "EP6DT", "F195": "1.6 THP 150", "F18C": "2018-03-15", "F187": "PSA 9807654321", "F189": "SW 4.1.2", "F17E": "ECM-PSA-2018" } },
  { nameKey: "demo.ecu.transmission", address: "0x7E1", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F191": "AL4/DP0", "F195": "TCM 3.0.1", "F18C": "2018-02-20" } },
  { nameKey: "demo.ecu.abs", address: "0x7E2", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F191": "MK60 v3", "F195": "ABS 2.1.0", "F18C": "2018-01-10" } },
  { nameKey: "demo.ecu.airbag", address: "0x7E3", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F191": "ACU4 v2", "F18C": "2018-04-01" } },
  { nameKey: "demo.ecu.bsi", address: "0x75D", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F18C": "BSI 2010", "F191": "BSI HW 1.5", "F195": "BSI SW 6.2", "F187": "BSI-96xxxxx", "F17E": "PSA-BSI-2018" } },
  { nameKey: "demo.ecu.hvac", address: "0x7E6", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F195": "HVAC 1.0" } },
  { nameKey: "demo.ecu.cluster", address: "0x7E5", protocol: "ISO 15765-4 CAN", dids: { "F190": "VF3LCBHZ6JS000000", "F195": "CLUST 2.3", "F18C": "2018-03-01" } },
];

export function buildDemoEcus(t: (key: string) => string): EcuInfo[] {
  return DEMO_ECU_KEYS.map((ecu) => ({
    name: t(ecu.nameKey),
    address: ecu.address,
    protocol: ecu.protocol,
    dids: ecu.dids,
  }));
}
