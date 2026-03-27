import { useSyncExternalStore, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { devInfo, devError } from "@/lib/devlog";

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
  elmVersion?: string;
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

let globalState = { ...defaultState };
let listeners: Set<() => void> = new Set();

function notify() {
  listeners.forEach((l) => l());
}

function subscribe(callback: () => void) {
  listeners.add(callback);
  return () => {
    listeners.delete(callback);
  };
}

function getSnapshot() {
  return globalState;
}

const setPort = (port: string) => {
  globalState = { ...globalState, port };
  notify();
};

const setBaudRate = (baudRate: number) => {
  globalState = { ...globalState, baudRate };
  notify();
};

const setStatus = (status: ConnectionStatus) => {
  globalState = { ...globalState, status };
  notify();
};

const setVehicle = (vehicle: VehicleInfo | null) => {
  globalState = { ...globalState, vehicle };
  notify();
};

const setError = (error: string | null) => {
  globalState = { ...globalState, error };
  notify();
};

const setPorts = (ports: string[]) => {
  globalState = { ...globalState, availablePorts: ports };
  notify();
};

const scanPorts = async () => {
  try {
    const ports = await invoke<Array<{name: string, description: string}>>("list_serial_ports");
    devInfo("ui", "Ports found: " + ports.length);
    globalState = { ...globalState, availablePorts: ports.map(p => p.name) };
    notify();
  } catch (e) {
    console.error("[BricarOBD] Failed to scan ports: " + String(e));
  }
};

const connect = async () => {
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
};

const disconnect = async () => {
  devInfo("ui", "Disconnected");
  try { await invoke("disconnect_obd"); } catch {}
  globalState = { ...defaultState, availablePorts: globalState.availablePorts };
  notify();
};

const updateVehicle = (vehicle: VehicleInfo) => {
  globalState = { ...globalState, vehicle };
  notify();
};

const connectWifi = async (host: string, port: number) => {
  devInfo("ui", "WiFi connecting to " + host + ":" + port);
  globalState = { ...globalState, status: "connecting", error: null };
  notify();
  try {
    const vehicle = await invoke<VehicleInfo>("connect_wifi", { host, port });
    devInfo("ui", "WiFi connected: " + vehicle.make + " " + vehicle.model);
    globalState = { ...globalState, status: "connected", vehicle };
    notify();
  } catch (e) {
    devError("ui", "WiFi connection error: " + String(e));
    globalState = { ...globalState, status: "error", error: String(e) };
    notify();
  }
};

const connectDemo = async () => {
  devInfo("ui", "Demo mode activated");
  try {
    const vehicle = await invoke<VehicleInfo>("connect_demo");
    globalState = { ...globalState, status: "demo", vehicle, error: null };
  } catch {
    globalState = { ...globalState, status: "demo", vehicle: { vin: "DEMO", make: "Demo", model: "Demo Vehicle", year: 2018, protocol: "Demo", elmVersion: "Demo v1.0" }, error: null };
  }
  notify();
};

const stableActions = {
  setPort,
  setBaudRate,
  setStatus,
  setVehicle,
  setError,
  setPorts,
  scanPorts,
  connect,
  disconnect,
  updateVehicle,
  connectWifi,
  connectDemo,
};

export function useConnectionStore() {
  const state = useSyncExternalStore(subscribe, getSnapshot);
  return useMemo(() => ({ ...state, ...stableActions }), [state]);
}
