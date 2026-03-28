import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from "react-i18next";

interface AndroidUsbBridge {
  listDevices(): string;
  requestPermission(deviceId: number): boolean;
  startBridge(deviceId: number, baudRate: number): string;
  stopBridge(): void;
  isRunning(): boolean;
}

interface AndroidBleBridge {
  hasPermissions(): boolean;
  requestPermissions(): void;
  scanDevices(timeoutMs: number): string;
  startBridge(deviceAddress: string): string;
  stopBridge(): void;
  isRunning(): boolean;
}

interface UsbDevice {
  name: string;
  deviceId: string;
  vendorId: string;
  productId: string;
}

interface BleDevice {
  name: string;
  address: string;
}

function getAndroidUsb(): AndroidUsbBridge | null {
  return (window as unknown as { AndroidUsb?: AndroidUsbBridge }).AndroidUsb ?? null;
}

function getAndroidBle(): AndroidBleBridge | null {
  return (window as unknown as { AndroidBle?: AndroidBleBridge }).AndroidBle ?? null;
}

export function useAndroidBridge(
  baudRate: number,
  onConnectWifi: (host: string, port: number) => Promise<void>,
  onConnectBle: ((deviceName: string) => Promise<void>) | undefined,
  onDisconnect: () => void,
  showToast: (message: string, type?: "success" | "error") => void,
) {
  const { t } = useTranslation();
  const [usbDevices, setUsbDevices] = useState<UsbDevice[]>([]);
  const [selectedUsbDevice, setSelectedUsbDevice] = useState("");
  const [bleDevices, setBleDevices] = useState<BleDevice[]>([]);
  const [selectedBleDevice, setSelectedBleDevice] = useState("");

  const handleScanUsb = useCallback(async () => {
    try {
      const android = getAndroidUsb();
      if (!android) return;
      const parsed = JSON.parse(android.listDevices());
      if (!Array.isArray(parsed)) {
        showToast(`${t("common.error")}: ${t("connection.invalidDeviceList")}`, "error");
        return;
      }
      setUsbDevices(parsed);
      if (parsed.length > 0 && !selectedUsbDevice) {
        setSelectedUsbDevice(parsed[0].deviceId);
      }
      if (parsed.length === 0) {
        showToast(t("connection.usbNone"), "error");
      } else {
        showToast(t("connection.usbFound", { count: parsed.length }));
      }
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [selectedUsbDevice, showToast, t]);

  const handleConnectUsb = useCallback(async () => {
    try {
      const android = getAndroidUsb();
      if (!android) return;
      const devId = parseInt(selectedUsbDevice);
      if (isNaN(devId)) return;

      const hasPermission = android.requestPermission(devId);
      if (!hasPermission) {
        showToast(t("connection.usbPermissionPending"));
        return;
      }

      showToast(t("connection.usbBridgeStarting"));
      const result = JSON.parse(android.startBridge(devId, baudRate));
      if (!result?.ok || typeof result.port !== "number") {
        showToast(t("connection.usbBridgeError", { error: result?.error ?? "unknown" }), "error");
        return;
      }

      showToast(t("connection.usbBridgeReady"));
      await onConnectWifi("127.0.0.1", result.port);
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [selectedUsbDevice, baudRate, onConnectWifi, showToast, t]);

  const handleDisconnectUsb = useCallback(() => {
    try {
      getAndroidUsb()?.stopBridge();
    } catch (_) { /* bridge may not be running */ }
    onDisconnect();
  }, [onDisconnect]);

  const handleScanBle = useCallback(async () => {
    try {
      const androidBle = getAndroidBle();
      if (androidBle) {
        if (!androidBle.hasPermissions()) {
          androidBle.requestPermissions();
          showToast(t("connection.blePermissionPending"));
          return;
        }
        const parsed = JSON.parse(androidBle.scanDevices(5000));
        if (!Array.isArray(parsed)) { showToast(t("connection.bleNone"), "error"); return; }
        setBleDevices(parsed);
        if (parsed.length > 0 && !selectedBleDevice) setSelectedBleDevice(parsed[0].address);
        if (parsed.length === 0) showToast(t("connection.bleNone"), "error");
        else showToast(t("connection.bleFound", { count: parsed.length }));
        return;
      }
      const devices = await invoke<BleDevice[]>("scan_ble", { timeoutMs: 5000 });
      setBleDevices(devices);
      if (devices.length > 0 && !selectedBleDevice) setSelectedBleDevice(devices[0].address);
      if (devices.length === 0) showToast(t("connection.bleNone"), "error");
      else showToast(t("connection.bleFound", { count: devices.length }));
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [selectedBleDevice, showToast, t]);

  const handleConnectBle = useCallback(async () => {
    if (!selectedBleDevice) return;
    console.log("[BricarOBD] BLE connecting to", selectedBleDevice);
    try {
      const androidBle = getAndroidBle();
      if (androidBle) {
        showToast(t("connection.usbBridgeStarting"));
        const result = JSON.parse(androidBle.startBridge(selectedBleDevice));
        if (!result?.ok || typeof result.port !== "number") {
          showToast(t("connection.usbBridgeError", { error: result?.error ?? "unknown" }), "error");
          return;
        }
        await onConnectWifi("127.0.0.1", result.port);
        return;
      }
      if (onConnectBle) {
        await onConnectBle(selectedBleDevice);
      } else {
        showToast(t("common.error"), "error");
      }
    } catch (e) {
      showToast(`${t("common.error")}: ${e}`, "error");
    }
  }, [selectedBleDevice, onConnectBle, onConnectWifi, showToast, t]);

  const handleDisconnectBle = useCallback(() => {
    try {
      getAndroidBle()?.stopBridge();
    } catch (_) { /* bridge may not be running */ }
    onDisconnect();
  }, [onDisconnect]);

  return {
    usbDevices, selectedUsbDevice, setSelectedUsbDevice,
    bleDevices, selectedBleDevice, setSelectedBleDevice,
    handleScanUsb, handleConnectUsb, handleDisconnectUsb,
    handleScanBle, handleConnectBle, handleDisconnectBle,
  };
}
