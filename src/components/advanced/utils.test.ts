import { describe, it, expect } from "vitest";
import { parseServiceId, isCommandBlocked } from "./utils";

describe("parseServiceId()", () => {
  it("parses spaced hex format '2E F1 90'", () => {
    const result = parseServiceId("2E F1 90");
    expect(result).toBe(0x2e);
  });

  it("parses unspaced hex format '2EF190'", () => {
    const result = parseServiceId("2EF190");
    expect(result).toBe(0x2e);
  });

  it("parses single byte '01'", () => {
    const result = parseServiceId("01");
    expect(result).toBe(0x01);
  });

  it("parses spaced single byte '01 0C'", () => {
    const result = parseServiceId("01 0C");
    expect(result).toBe(0x01);
  });

  it("handles lowercase input by converting to uppercase", () => {
    const result = parseServiceId("2e f1 90");
    expect(result).toBe(0x2e);
  });

  it("handles mixed case input", () => {
    const result = parseServiceId("2E f1 90");
    expect(result).toBe(0x2e);
  });

  it("returns null for empty string", () => {
    const result = parseServiceId("");
    expect(result).toBeNull();
  });

  it("returns null for whitespace only", () => {
    const result = parseServiceId("   ");
    expect(result).toBeNull();
  });

  it("returns null for invalid hex characters", () => {
    const result = parseServiceId("ZZ");
    expect(result).toBeNull();
  });

  it("returns null for single invalid hex character", () => {
    const result = parseServiceId("X");
    expect(result).toBeNull();
  });

  it("extracts first service ID from multi-byte command", () => {
    const result = parseServiceId("22 F1 90");
    expect(result).toBe(0x22);
  });

  it("handles leading whitespace", () => {
    const result = parseServiceId("  2E F1 90");
    expect(result).toBe(0x2e);
  });

  it("handles trailing whitespace", () => {
    const result = parseServiceId("2E F1 90  ");
    expect(result).toBe(0x2e);
  });

  it("handles multiple spaces between bytes", () => {
    const result = parseServiceId("2E   F1   90");
    expect(result).toBe(0x2e);
  });

  it("parses common OBD service IDs correctly", () => {
    expect(parseServiceId("01")).toBe(0x01);
    expect(parseServiceId("03")).toBe(0x03);
    expect(parseServiceId("04")).toBe(0x04);
    expect(parseServiceId("06")).toBe(0x06);
    expect(parseServiceId("07")).toBe(0x07);
    expect(parseServiceId("09")).toBe(0x09);
  });

  it("parses manufacturer service IDs", () => {
    expect(parseServiceId("22 F1 90")).toBe(0x22);
  });
});

describe("isCommandBlocked()", () => {
  describe("blocked service IDs", () => {
    it("blocks 0x11 (ECUReset)", () => {
      expect(isCommandBlocked("11 01")).toBe(true);
      expect(isCommandBlocked("11")).toBe(true);
    });

    it("blocks 0x27 (SecurityAccess)", () => {
      expect(isCommandBlocked("27 01")).toBe(true);
      expect(isCommandBlocked("27 02 1234")).toBe(true);
    });

    it("blocks 0x28 (CommunicationControl)", () => {
      expect(isCommandBlocked("28 00")).toBe(true);
      expect(isCommandBlocked("28")).toBe(true);
    });

    it("blocks 0x34 (RequestDownload)", () => {
      expect(isCommandBlocked("34 00")).toBe(true);
      expect(isCommandBlocked("34")).toBe(true);
    });

    it("blocks 0x35 (RequestUpload)", () => {
      expect(isCommandBlocked("35 00")).toBe(true);
      expect(isCommandBlocked("35")).toBe(true);
    });

    it("blocks 0x36 (TransferData)", () => {
      expect(isCommandBlocked("36 00")).toBe(true);
      expect(isCommandBlocked("36")).toBe(true);
    });

    it("blocks 0x37 (RequestTransferExit)", () => {
      expect(isCommandBlocked("37 00")).toBe(true);
      expect(isCommandBlocked("37")).toBe(true);
    });

    it("blocks 0x3D (WriteMemoryByAddress)", () => {
      expect(isCommandBlocked("3D 00")).toBe(true);
      expect(isCommandBlocked("3D")).toBe(true);
    });

    it("blocks all 8 dangerous service IDs", () => {
      const dangerousIds = [0x11, 0x27, 0x28, 0x34, 0x35, 0x36, 0x37, 0x3d];
      dangerousIds.forEach((id) => {
        const hex = id.toString(16).toUpperCase().padStart(2, "0");
        expect(isCommandBlocked(hex)).toBe(true);
      });
    });
  });

  describe("blocked AT commands", () => {
    it("blocks ATMA", () => {
      expect(isCommandBlocked("ATMA")).toBe(true);
    });

    it("blocks ATBD", () => {
      expect(isCommandBlocked("ATBD")).toBe(true);
    });

    it("blocks ATBI", () => {
      expect(isCommandBlocked("ATBI")).toBe(true);
    });

    it("blocks ATPP", () => {
      expect(isCommandBlocked("ATPP")).toBe(true);
    });

    it("blocks ATWS", () => {
      expect(isCommandBlocked("ATWS")).toBe(true);
    });

    it("blocks AT commands case-insensitively", () => {
      expect(isCommandBlocked("atma")).toBe(true);
      expect(isCommandBlocked("Atma")).toBe(true);
      expect(isCommandBlocked("AtMa")).toBe(true);
    });

    it("allows other AT commands", () => {
      expect(isCommandBlocked("ATZ")).toBe(false);
      expect(isCommandBlocked("ATE0")).toBe(false);
      expect(isCommandBlocked("ATL0")).toBe(false);
      expect(isCommandBlocked("ATH1")).toBe(false);
    });
  });

  describe("allowed OBD commands", () => {
    it("allows Mode 01 (Show Data)", () => {
      expect(isCommandBlocked("01 0C")).toBe(false);
      expect(isCommandBlocked("01")).toBe(false);
    });

    it("allows Mode 02 (Freeze Frame)", () => {
      expect(isCommandBlocked("02 0C")).toBe(false);
    });

    it("allows Mode 03 (DTC Status)", () => {
      expect(isCommandBlocked("03")).toBe(false);
    });

    it("allows Mode 04 (Clear DTCs)", () => {
      expect(isCommandBlocked("04")).toBe(false);
    });

    it("allows Mode 05 (Test Results)", () => {
      expect(isCommandBlocked("05")).toBe(false);
    });

    it("allows Mode 06 (Continuous Monitors)", () => {
      expect(isCommandBlocked("06")).toBe(false);
    });

    it("allows Mode 07 (Pending DTCs)", () => {
      expect(isCommandBlocked("07")).toBe(false);
    });

    it("allows Mode 08 (Control)", () => {
      expect(isCommandBlocked("08")).toBe(false);
    });

    it("allows Mode 09 (Vehicle Information)", () => {
      expect(isCommandBlocked("09 02")).toBe(false);
    });

    it("allows Mode 0A (Permanent DTCs)", () => {
      expect(isCommandBlocked("0A")).toBe(false);
    });

    it("allows Mode 22 (ReadDataByIdentifier - Manufacturer)", () => {
      expect(isCommandBlocked("22 F1 90")).toBe(false);
    });

    it("allows Mode 3E (TesterPresent)", () => {
      expect(isCommandBlocked("3E")).toBe(false);
      expect(isCommandBlocked("3E 00")).toBe(false);
    });

    it("allows other safe modes", () => {
      expect(isCommandBlocked("10 01")).toBe(false);
      expect(isCommandBlocked("19 00")).toBe(false);
    });
  });

  describe("edge cases", () => {
    it("handles empty string", () => {
      expect(isCommandBlocked("")).toBe(false);
    });

    it("handles whitespace only", () => {
      expect(isCommandBlocked("   ")).toBe(false);
    });

    it("handles command with leading/trailing whitespace", () => {
      expect(isCommandBlocked("  11 01  ")).toBe(true);
      expect(isCommandBlocked("  01 0C  ")).toBe(false);
    });

    it("distinguishes between similar service IDs", () => {
      expect(isCommandBlocked("3C")).toBe(false);
      expect(isCommandBlocked("3D")).toBe(true);
    });

    it("distinguishes between similar AT commands", () => {
      expect(isCommandBlocked("ATW")).toBe(false);
      expect(isCommandBlocked("ATWS")).toBe(true);
    });

    it("handles unspaced service ID format", () => {
      expect(isCommandBlocked("110102")).toBe(true);
      expect(isCommandBlocked("010C")).toBe(false);
    });
  });

  describe("combined validation", () => {
    it("checks service ID first for hex commands", () => {
      expect(isCommandBlocked("11 AA BB")).toBe(true);
    });

    it("distinguishes hex commands from AT commands", () => {
      expect(isCommandBlocked("01 0C")).toBe(false);
      expect(isCommandBlocked("ATMA")).toBe(true);
    });

    it("handles mode 2E (Write DID) - allowed for advanced", () => {
      expect(isCommandBlocked("2E F1 90 01 02")).toBe(false);
    });

    it("handles mode 2F (InputOutputControl) - allowed for advanced", () => {
      expect(isCommandBlocked("2F F1 90")).toBe(false);
    });

    it("handles mode 30 (ReadExtendedDataRecord) - allowed for advanced", () => {
      expect(isCommandBlocked("30 87")).toBe(false);
    });

    it("handles mode 31 (ReadDDL) - allowed for advanced", () => {
      expect(isCommandBlocked("31 F1 90")).toBe(false);
    });
  });
});
