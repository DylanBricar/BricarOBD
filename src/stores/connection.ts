import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { devInfo, devWarn, devError } from "@/lib/devlog";

export type ConnectionStatus =
  | "disconnected"
  | "connecting"
  | "connected"
  | "error"
  | "demo";

export interface VehicleInfo {
  vin: string;
  make: string;
  model: string;
  year: number;
  protocol: string;
  elmVersion: string;
}

export interface ConnectionState {
  status: ConnectionStatus;
  port: string;
  baudRate: number;
  vehicle: VehicleInfo | null;
  error: string | null;
  availablePorts: string[];
}

const defaultState: ConnectionState = {
  status: "disconnected",
  port: "",
  baudRate: 38400,
  vehicle: null,
  error: null,
  availablePorts: [],
};

// Simple global state (will be replaced by Tauri events later)
let globalState = { ...defaultState };
let listeners: Set<() => void> = new Set();

function notify() {
  listeners.forEach((l) => l());
}

export function useConnectionStore() {
  const [, setTick] = useState(0);

  useEffect(() => {
    const forceUpdate = () => setTick((t) => t + 1);
    listeners.add(forceUpdate);
    return () => {
      listeners.delete(forceUpdate);
    };
  }, []);

  return {
    ...globalState,
    setPort: (port: string) => {
      globalState = { ...globalState, port };
      notify();
    },
    setBaudRate: (baudRate: number) => {
      globalState = { ...globalState, baudRate };
      notify();
    },
    setStatus: (status: ConnectionStatus) => {
      globalState = { ...globalState, status };
      notify();
    },
    setVehicle: (vehicle: VehicleInfo | null) => {
      globalState = { ...globalState, vehicle };
      notify();
    },
    setError: (error: string | null) => {
      globalState = { ...globalState, error };
      notify();
    },
    setPorts: (ports: string[]) => {
      globalState = { ...globalState, availablePorts: ports };
      notify();
    },
    scanPorts: async () => {
      try {
        const ports = await invoke<Array<{name: string, description: string}>>("list_serial_ports");
        devInfo("ui", "Ports found: " + ports.length);
        globalState = { ...globalState, availablePorts: ports.map(p => p.name) };
        notify();
      } catch {}
    },
    connect: async () => {
      devInfo("ui", "Connecting to " + globalState.port + " at " + globalState.baudRate);
      globalState = { ...globalState, status: "connecting", error: null };
      notify();
      try {
        const vehicle = await invoke<VehicleInfo>("connect_obd", { port: globalState.port, baudRate: globalState.baudRate });
        devInfo("ui", "Connected: " + vehicle.make + " " + vehicle.model);
        globalState = { ...globalState, status: "connected", vehicle };
        notify();
      } catch (e) {
        devError("ui", "Connection error: " + String(e));
        globalState = { ...globalState, status: "error", error: String(e) };
        notify();
      }
    },
    disconnect: async () => {
      devInfo("ui", "Disconnected");
      try { await invoke("disconnect_obd"); } catch {}
      globalState = { ...defaultState, availablePorts: globalState.availablePorts };
      notify();
    },
    updateVehicle: (vehicle: VehicleInfo) => {
      globalState = { ...globalState, vehicle };
      notify();
    },
    connectDemo: async () => {
      devInfo("ui", "Demo mode activated");
      try {
        const vehicle = await invoke<VehicleInfo>("connect_demo");
        globalState = { ...globalState, status: "demo", vehicle, error: null };
      } catch {
        globalState = { ...globalState, status: "demo", vehicle: { vin: "DEMO", make: "Peugeot", model: "207 (Démo)", year: 2018, protocol: "Demo", elmVersion: "Demo v1.0" }, error: null };
      }
      notify();
    },
  };
}
