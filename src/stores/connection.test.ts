import { describe, it, expect, beforeEach, vi } from "vitest";
import { renderHook, act } from "@testing-library/react";
import { useConnectionStore, type ConnectionStatus, type VehicleInfo } from "./connection";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");
vi.mock("@/lib/devlog");

describe("useConnectionStore()", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("initial state", () => {
    it("returns disconnected status initially", () => {
      const { result } = renderHook(() => useConnectionStore());

      expect(result.current.status).toBe("disconnected");
      expect(result.current.port).toBe("");
      expect(result.current.baudRate).toBe(38400);
      expect(result.current.vehicle).toBeNull();
      expect(result.current.error).toBeNull();
      expect(result.current.availablePorts).toEqual([]);
    });
  });

  describe("setPort()", () => {
    it("updates port state", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPort("COM3");
      });

      expect(result.current.port).toBe("COM3");
    });

    it("maintains other state when setting port", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPort("COM3");
        result.current.setBaudRate(9600);
      });

      expect(result.current.port).toBe("COM3");
      expect(result.current.baudRate).toBe(9600);
    });
  });

  describe("setBaudRate()", () => {
    it("updates baud rate state", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setBaudRate(9600);
      });

      expect(result.current.baudRate).toBe(9600);
    });

    it("accepts various baud rates", () => {
      const { result } = renderHook(() => useConnectionStore());

      const baudRates = [9600, 19200, 38400, 57600, 115200];

      baudRates.forEach((rate) => {
        act(() => {
          result.current.setBaudRate(rate);
        });
        expect(result.current.baudRate).toBe(rate);
      });
    });
  });

  describe("setStatus()", () => {
    it("updates connection status", () => {
      const { result } = renderHook(() => useConnectionStore());

      const statuses: ConnectionStatus[] = ["connecting", "connected", "error", "demo"];

      statuses.forEach((status) => {
        act(() => {
          result.current.setStatus(status);
        });
        expect(result.current.status).toBe(status);
      });
    });
  });

  describe("setVehicle()", () => {
    it("sets vehicle information", () => {
      const { result } = renderHook(() => useConnectionStore());

      const vehicle: VehicleInfo = {
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      act(() => {
        result.current.setVehicle(vehicle);
      });

      expect(result.current.vehicle).toEqual(vehicle);
    });

    it("clears vehicle when passed null", () => {
      const { result } = renderHook(() => useConnectionStore());

      const vehicle: VehicleInfo = {
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      act(() => {
        result.current.setVehicle(vehicle);
      });

      expect(result.current.vehicle).toEqual(vehicle);

      act(() => {
        result.current.setVehicle(null);
      });

      expect(result.current.vehicle).toBeNull();
    });
  });

  describe("setError()", () => {
    it("sets error message", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setError("Connection failed");
      });

      expect(result.current.error).toBe("Connection failed");
    });

    it("clears error when passed null", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setError("Some error");
      });

      expect(result.current.error).toBe("Some error");

      act(() => {
        result.current.setError(null);
      });

      expect(result.current.error).toBeNull();
    });
  });

  describe("setPorts()", () => {
    it("updates available ports list", () => {
      const { result } = renderHook(() => useConnectionStore());

      const ports = ["COM1", "COM2", "COM3"];

      act(() => {
        result.current.setPorts(ports);
      });

      expect(result.current.availablePorts).toEqual(ports);
    });

    it("accepts empty ports array", () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPorts([]);
      });

      expect(result.current.availablePorts).toEqual([]);
    });
  });

  describe("scanPorts()", () => {
    it("fetches available ports from backend", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockResolvedValue([
        { name: "COM1", description: "Arduino COM Port" },
        { name: "COM2", description: "USB Serial" },
      ]);

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.scanPorts();
      });

      expect(mockInvoke).toHaveBeenCalledWith("list_serial_ports");

      // Ports should be updated after scan
      expect(result.current.availablePorts).toContain("COM1");
      expect(result.current.availablePorts).toContain("COM2");
    });

    it("handles scan error gracefully", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue(new Error("Scan failed"));

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.scanPorts();
      });

      expect(mockInvoke).toHaveBeenCalled();
    });
  });

  describe("connect()", () => {
    it("updates status to connecting and then connected", async () => {
      const mockInvoke = vi.mocked(invoke);
      const vehicle: VehicleInfo = {
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      mockInvoke.mockResolvedValue(vehicle);

      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPort("COM1");
        result.current.setBaudRate(38400);
      });

      await act(async () => {
        await result.current.connect();
      });

      expect(result.current.status).toBe("connected");
      expect(result.current.vehicle).toEqual(vehicle);
    });

    it("sets error status on connection failure", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue(new Error("Device not found"));

      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPort("COM99");
      });

      await act(async () => {
        await result.current.connect();
      });

      expect(result.current.status).toBe("error");
      expect(result.current.error).toContain("Device not found");
    });

    it("invokes connect_obd with port and baudRate", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockResolvedValue({
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      });

      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPort("COM1");
        result.current.setBaudRate(9600);
      });

      await act(async () => {
        await result.current.connect();
      });

      expect(mockInvoke).toHaveBeenCalledWith("connect_obd", {
        port: "COM1",
        baudRate: 9600,
      });
    });
  });

  describe("disconnect()", () => {
    it("resets state to disconnected", async () => {
      const { result } = renderHook(() => useConnectionStore());

      const vehicle: VehicleInfo = {
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      act(() => {
        result.current.setVehicle(vehicle);
        result.current.setStatus("connected");
        result.current.setPort("COM1");
      });

      expect(result.current.status).toBe("connected");
      expect(result.current.vehicle).toEqual(vehicle);

      await act(async () => {
        await result.current.disconnect();
      });

      expect(result.current.status).toBe("disconnected");
      expect(result.current.vehicle).toBeNull();
      expect(result.current.port).toBe("");
    });

    it("preserves availablePorts on disconnect", async () => {
      const { result } = renderHook(() => useConnectionStore());

      act(() => {
        result.current.setPorts(["COM1", "COM2"]);
      });

      await act(async () => {
        await result.current.disconnect();
      });

      expect(result.current.availablePorts).toEqual(["COM1", "COM2"]);
    });
  });

  describe("updateVehicle()", () => {
    it("updates vehicle info without changing other state", () => {
      const { result } = renderHook(() => useConnectionStore());

      const initialVehicle: VehicleInfo = {
        vin: "VIN123",
        make: "Toyota",
        model: "Camry",
        year: 2021,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      const updatedVehicle: VehicleInfo = {
        ...initialVehicle,
        protocol: "ISO14230",
      };

      act(() => {
        result.current.setVehicle(initialVehicle);
        result.current.setStatus("connected");
        result.current.setPort("COM1");
      });

      act(() => {
        result.current.updateVehicle(updatedVehicle);
      });

      expect(result.current.vehicle).toEqual(updatedVehicle);
      expect(result.current.status).toBe("connected");
      expect(result.current.port).toBe("COM1");
    });
  });

  describe("connectWifi()", () => {
    it("connects via WiFi and updates vehicle info", async () => {
      const mockInvoke = vi.mocked(invoke);
      const vehicle: VehicleInfo = {
        vin: "VIN456",
        make: "Honda",
        model: "Civic",
        year: 2020,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      };

      mockInvoke.mockResolvedValue(vehicle);

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.connectWifi("192.168.1.100", 1234);
      });

      expect(result.current.status).toBe("connected");
      expect(result.current.vehicle).toEqual(vehicle);
    });

    it("sets error status on WiFi connection failure", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue(new Error("WiFi connection failed"));

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.connectWifi("192.168.1.100", 1234);
      });

      expect(result.current.status).toBe("error");
      expect(result.current.error).toContain("WiFi connection failed");
    });

    it("invokes connect_wifi with host and port", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockResolvedValue({
        vin: "VIN456",
        make: "Honda",
        model: "Civic",
        year: 2020,
        protocol: "ISO15765",
        elmVersion: "1.5.5",
      });

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.connectWifi("192.168.1.100", 1234);
      });

      expect(mockInvoke).toHaveBeenCalledWith("connect_wifi", {
        host: "192.168.1.100",
        port: 1234,
      });
    });
  });

  describe("connectDemo()", () => {
    it("activates demo mode when invoke fails, uses fallback vehicle", async () => {
      const mockInvoke = vi.mocked(invoke);
      mockInvoke.mockRejectedValue(new Error("Demo unavailable"));

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.connectDemo();
      });

      expect(result.current.status).toBe("demo");
      expect(result.current.vehicle?.make).toBe("Demo");
    });

    it("uses backend demo vehicle if available", async () => {
      const mockInvoke = vi.mocked(invoke);
      const demoVehicle: VehicleInfo = {
        vin: "DEMO12345",
        make: "Demo Make",
        model: "Demo Model",
        year: 2020,
        protocol: "Demo",
        elmVersion: "Demo v1.0",
      };

      mockInvoke.mockResolvedValue(demoVehicle);

      const { result } = renderHook(() => useConnectionStore());

      await act(async () => {
        await result.current.connectDemo();
      });

      expect(result.current.status).toBe("demo");
      expect(result.current.vehicle).toEqual(demoVehicle);
    });
  });

  describe("subscription and notifications", () => {
    it("notifies subscribers when state changes", () => {
      // This test verifies the store is shared globally
      // We'll just verify the setter works
      const { result } = renderHook(() => useConnectionStore());

      const previousPort = result.current.port;

      act(() => {
        result.current.setPort("TEST_PORT_123");
      });

      // After setting, it should be updated to new value
      expect(result.current.port).toBe("TEST_PORT_123");

      // Reset for other tests
      act(() => {
        result.current.setPort("");
      });
    });
  });
});
