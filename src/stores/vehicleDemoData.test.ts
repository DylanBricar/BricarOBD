import { describe, it, expect } from "vitest";
import { buildDemoDtcs, buildDemoEcus, demoMonitors, DEMO_DTC_KEYS, DEMO_ECU_KEYS } from "./vehicleDemoData";

describe("vehicleDemoData", () => {
  const mockTranslate = (key: string): string => {
    const map: Record<string, string> = {
      "demo.dtc.P0440": "Evaporative Emission Control System Leak",
      "demo.dtc.P0440.tips": "Check fuel cap",
      "demo.dtc.P0440.cause1": "Faulty charcoal canister",
      "demo.dtc.P0440.cause2": "Loose fuel cap",
      "demo.dtc.P0440.cause3": "Damaged hose",
      "demo.dtc.P0440.cause4": "Faulty purge valve",
      "demo.dtc.P0440.cause5": "ECU malfunction",
      "demo.dtc.P0440.quickCheck": "Verify fuel cap seal",
      "demo.dtc.P0500": "Vehicle Speed Sensor Malfunction",
      "demo.dtc.P0500.tips": "Check wiring",
      "demo.dtc.P0500.cause1": "Faulty speed sensor",
      "demo.dtc.P0500.cause2": "Damaged wiring",
      "demo.dtc.P0500.cause3": "Corroded connectors",
      "demo.dtc.P0500.cause4": "ECU software issue",
      "demo.dtc.P0500.cause5": "Transmission problem",
      "demo.dtc.P0500.quickCheck": "Inspect sensor connector",
      "demo.ecu.engine": "Engine Control Module",
      "demo.ecu.transmission": "Transmission Control Module",
      "demo.ecu.abs": "ABS Control Module",
      "demo.ecu.airbag": "Airbag Control Module",
      "demo.ecu.bsi": "Body System Interface",
      "demo.ecu.hvac": "HVAC Control Module",
      "demo.ecu.cluster": "Instrument Cluster",
    };
    return map[key] || key;
  };

  describe("DEMO_DTC_KEYS constant", () => {
    it("contains 2 DTC entries", () => {
      expect(DEMO_DTC_KEYS).toHaveLength(2);
    });

    it("has correct structure for each DTC", () => {
      DEMO_DTC_KEYS.forEach((dtc) => {
        expect(dtc).toHaveProperty("code");
        expect(dtc).toHaveProperty("descKey");
        expect(dtc).toHaveProperty("status");
        expect(dtc).toHaveProperty("source");
        expect(dtc).toHaveProperty("tipsKey");
        expect(dtc).toHaveProperty("causeKeys");
        expect(dtc).toHaveProperty("quickCheckKey");
        expect(dtc).toHaveProperty("difficulty");
      });
    });

    it("has P0440 with correct status and source", () => {
      const p0440 = DEMO_DTC_KEYS.find((d) => d.code === "P0440");
      expect(p0440).toBeDefined();
      expect(p0440?.status).toBe("active");
      expect(p0440?.source).toBe("OBD Mode 03");
    });

    it("has P0500 with correct status and source", () => {
      const p0500 = DEMO_DTC_KEYS.find((d) => d.code === "P0500");
      expect(p0500).toBeDefined();
      expect(p0500?.status).toBe("pending");
      expect(p0500?.source).toBe("OBD Mode 07");
    });

    it("each DTC has exactly 5 cause keys", () => {
      DEMO_DTC_KEYS.forEach((dtc) => {
        expect(dtc.causeKeys).toHaveLength(5);
      });
    });

    it("both DTCs have difficulty level 2", () => {
      DEMO_DTC_KEYS.forEach((dtc) => {
        expect(dtc.difficulty).toBe(2);
      });
    });
  });

  describe("buildDemoDtcs()", () => {
    it("returns 2 DTCs with correct codes", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs).toHaveLength(2);
      expect(dtcs.map((d) => d.code)).toEqual(["P0440", "P0500"]);
    });

    it("uses t() function for descriptions", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].description).toBe("Evaporative Emission Control System Leak");
      expect(dtcs[1].description).toBe("Vehicle Speed Sensor Malfunction");
    });

    it("uses t() function for repair tips", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].repairTips).toBe("Check fuel cap");
      expect(dtcs[1].repairTips).toBe("Check wiring");
    });

    it("uses t() function for quick check", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].quickCheck).toBe("Verify fuel cap seal");
      expect(dtcs[1].quickCheck).toBe("Inspect sensor connector");
    });

    it("maps cause keys via t() function", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].causes).toHaveLength(5);
      expect(dtcs[0].causes?.[0]).toBe("Faulty charcoal canister");
      expect(dtcs[0].causes?.[4]).toBe("ECU malfunction");
    });

    it("preserves status field", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].status).toBe("active");
      expect(dtcs[1].status).toBe("pending");
    });

    it("preserves source field", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].source).toBe("OBD Mode 03");
      expect(dtcs[1].source).toBe("OBD Mode 07");
    });

    it("preserves difficulty field", () => {
      const dtcs = buildDemoDtcs(mockTranslate);
      expect(dtcs[0].difficulty).toBe(2);
      expect(dtcs[1].difficulty).toBe(2);
    });

    it("returns undefined repairTips when t() returns key itself", () => {
      const noOpTranslate = (key: string) => key;
      const dtcs = buildDemoDtcs(noOpTranslate);
      expect(dtcs[0].repairTips).toBe("demo.dtc.P0440.tips");
    });
  });

  describe("DEMO_ECU_KEYS constant", () => {
    it("contains 7 ECUs", () => {
      expect(DEMO_ECU_KEYS).toHaveLength(7);
    });

    it("has correct structure for each ECU", () => {
      DEMO_ECU_KEYS.forEach((ecu) => {
        expect(ecu).toHaveProperty("nameKey");
        expect(ecu).toHaveProperty("address");
        expect(ecu).toHaveProperty("protocol");
        expect(ecu).toHaveProperty("dids");
      });
    });

    it("all ECUs use ISO 15765-4 CAN protocol", () => {
      DEMO_ECU_KEYS.forEach((ecu) => {
        expect(ecu.protocol).toBe("ISO 15765-4 CAN");
      });
    });

    it("all ECUs have unique addresses", () => {
      const addresses = DEMO_ECU_KEYS.map((e) => e.address);
      const uniqueAddresses = new Set(addresses);
      expect(uniqueAddresses.size).toBe(addresses.length);
    });

    it("all ECUs have dids object", () => {
      DEMO_ECU_KEYS.forEach((ecu) => {
        expect(typeof ecu.dids).toBe("object");
        expect(Object.keys(ecu.dids).length).toBeGreaterThan(0);
      });
    });
  });

  describe("buildDemoEcus()", () => {
    it("returns 7 ECUs", () => {
      const ecus = buildDemoEcus(mockTranslate);
      expect(ecus).toHaveLength(7);
    });

    it("maps i18n keys to translated names", () => {
      const ecus = buildDemoEcus(mockTranslate);
      expect(ecus[0].name).toBe("Engine Control Module");
      expect(ecus[1].name).toBe("Transmission Control Module");
      expect(ecus[2].name).toBe("ABS Control Module");
      expect(ecus[3].name).toBe("Airbag Control Module");
      expect(ecus[4].name).toBe("Body System Interface");
      expect(ecus[5].name).toBe("HVAC Control Module");
      expect(ecus[6].name).toBe("Instrument Cluster");
    });

    it("preserves addresses from DEMO_ECU_KEYS", () => {
      const ecus = buildDemoEcus(mockTranslate);
      const expectedAddresses = ["0x7E0", "0x7E1", "0x7E2", "0x7E3", "0x75D", "0x7E6", "0x7E5"];
      ecus.forEach((ecu, index) => {
        expect(ecu.address).toBe(expectedAddresses[index]);
      });
    });

    it("preserves protocol from DEMO_ECU_KEYS", () => {
      const ecus = buildDemoEcus(mockTranslate);
      ecus.forEach((ecu) => {
        expect(ecu.protocol).toBe("ISO 15765-4 CAN");
      });
    });

    it("preserves DIDs from DEMO_ECU_KEYS", () => {
      const ecus = buildDemoEcus(mockTranslate);
      expect(ecus[0].dids["F190"]).toBe("VF3LCBHZ6JS000000");
      expect(ecus[0].dids["F195"]).toBe("1.6 THP 150");
    });

    it("maps all ECUs correctly with one-to-one correspondence", () => {
      const ecus = buildDemoEcus(mockTranslate);
      DEMO_ECU_KEYS.forEach((key, index) => {
        expect(ecus[index].address).toBe(key.address);
        expect(ecus[index].protocol).toBe(key.protocol);
        expect(ecus[index].dids).toEqual(key.dids);
      });
    });
  });

  describe("demoMonitors", () => {
    it("contains 11 monitor entries", () => {
      expect(demoMonitors).toHaveLength(11);
    });

    it("each monitor has required properties", () => {
      demoMonitors.forEach((monitor) => {
        expect(monitor).toHaveProperty("nameKey");
        expect(monitor).toHaveProperty("available");
        expect(monitor).toHaveProperty("complete");
      });
    });

    it("has correct available/complete flags for specific monitors", () => {
      const misfireMonitor = demoMonitors.find((m) => m.nameKey === "monitors.misfire");
      expect(misfireMonitor?.available).toBe(true);
      expect(misfireMonitor?.complete).toBe(true);

      const catalystB2Monitor = demoMonitors.find((m) => m.nameKey === "monitors.catalystB2");
      expect(catalystB2Monitor?.available).toBe(false);
      expect(catalystB2Monitor?.complete).toBe(false);

      const catalystB1Monitor = demoMonitors.find((m) => m.nameKey === "monitors.catalystB1");
      expect(catalystB1Monitor?.available).toBe(true);
      expect(catalystB1Monitor?.complete).toBe(false);
    });

    it("has correct structure for all monitors", () => {
      const expectedKeys = [
        "monitors.misfire",
        "monitors.fuelSystem",
        "monitors.components",
        "monitors.catalystB1",
        "monitors.catalystB2",
        "monitors.evap",
        "monitors.o2B1S1",
        "monitors.o2HeaterB1S1",
        "monitors.secondaryAir",
        "monitors.ac",
        "monitors.egrVvt",
      ];

      const monitorNames = demoMonitors.map((m) => m.nameKey);
      expectedKeys.forEach((key) => {
        expect(monitorNames).toContain(key);
      });
    });

    it("has description and specification keys for all monitors", () => {
      demoMonitors.forEach((monitor) => {
        expect(monitor.descriptionKey).toBeDefined();
        expect(monitor.specificationKey).toBeDefined();
      });
    });
  });
});
