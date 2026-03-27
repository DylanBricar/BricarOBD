import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import WiFiSettings from "./WiFiSettings";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string) => {
    if (cmd === "scan_wifi") {
      return [
        { host: "192.168.0.10", port: 35000, name: "ELM1" },
        { host: "192.168.0.11", port: 35000, name: "ELM2" },
      ];
    }
    return null;
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: any) => {
      const keys: Record<string, string> = {
        "connection.wifiHost": "WiFi Host",
        "connection.wifiPort": "WiFi Port",
        "connection.wifiHostInvalid": "Invalid IP address",
        "connection.wifiPortInvalid": "Invalid port",
        "connection.wifiScan": "Scan WiFi",
        "connection.scanning": "Scanning...",
        "connection.wifiAdapters": "Found Adapters",
        "connection.connect": "Connect",
        "connection.connecting": "Connecting...",
        "connection.wifiFound": `Found ${opts?.count || 0} adapters`,
        "connection.wifiNone": "No WiFi adapters found",
        "common.error": "Error",
      };
      return keys[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

describe("WiFiSettings", () => {
  it("renders host input", () => {
    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    expect(screen.getByText("WiFi Host")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("192.168.0.10")).toBeInTheDocument();
  });

  it("renders port input", () => {
    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    expect(screen.getByText("WiFi Port")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("35000")).toBeInTheDocument();
  });

  it("renders scan button", () => {
    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    expect(screen.getByText("Scan WiFi")).toBeInTheDocument();
  });

  it("renders connect button when not connected", () => {
    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    expect(screen.getByText("Connect")).toBeInTheDocument();
  });

  it("hides connect button when connected", () => {
    const { container } = render(
      <WiFiSettings
        isConnected={true}
        status="connected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const buttons = Array.from(container.querySelectorAll("button"));
    const connectButton = buttons.find((btn) =>
      btn.textContent?.includes("Connect") && !btn.textContent?.includes("Connecting")
    );

    expect(connectButton).toBeUndefined();
  });

  it("validates IP address format when invalid", async () => {
    const user = userEvent.setup();
    const { container } = render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const hostInput = screen.getByPlaceholderText("192.168.0.10") as HTMLInputElement;
    await user.clear(hostInput);
    await user.type(hostInput, "invalid");

    expect(screen.getByText("Invalid IP address")).toBeInTheDocument();
  });

  it("validates port range when invalid", async () => {
    const user = userEvent.setup();
    const { container } = render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const portInput = screen.getByPlaceholderText("35000") as HTMLInputElement;
    await user.clear(portInput);
    await user.type(portInput, "99999");

    expect(screen.getByText("Invalid port")).toBeInTheDocument();
  });

  it("accepts valid IP address", async () => {
    const user = userEvent.setup();

    const { container } = render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const hostInput = screen.getByPlaceholderText("192.168.0.10") as HTMLInputElement;
    await user.clear(hostInput);
    await user.type(hostInput, "192.168.1.100");

    expect(hostInput.value).toBe("192.168.1.100");
  });

  it("accepts valid port", async () => {
    const user = userEvent.setup();

    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const portInput = screen.getByPlaceholderText("35000") as HTMLInputElement;
    await user.clear(portInput);
    await user.type(portInput, "35000");

    expect(portInput.value).toBe("35000");
  });

  it("disables inputs when connected", () => {
    render(
      <WiFiSettings
        isConnected={true}
        status="connected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const hostInput = screen.getByPlaceholderText("192.168.0.10") as HTMLInputElement;
    expect(hostInput.disabled).toBe(true);
  });

  it("disables scan button when connected", () => {
    const { container } = render(
      <WiFiSettings
        isConnected={true}
        status="connected"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const scanButton = Array.from(container.querySelectorAll("button")).find(
      (btn) => btn.textContent?.includes("Scan WiFi")
    ) as HTMLButtonElement;

    expect(scanButton.disabled).toBe(true);
  });

  it("disables connect button while connecting", () => {
    render(
      <WiFiSettings
        isConnected={false}
        status="connecting"
        onConnectWifi={vi.fn()}
        showToast={vi.fn()}
      />
    );

    const connectButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("Connect")
    ) as HTMLButtonElement;

    expect(connectButton.disabled).toBe(true);
  });

  it("calls onConnectWifi with host and port", async () => {
    const onConnectWifi = vi.fn();
    const user = userEvent.setup();

    const { container } = render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={onConnectWifi}
        showToast={vi.fn()}
      />
    );

    const hostInput = screen.getByPlaceholderText("192.168.0.10") as HTMLInputElement;
    const portInput = screen.getByPlaceholderText("35000") as HTMLInputElement;
    const connectButton = Array.from(container.querySelectorAll("button")).find(
      (btn) => btn.textContent?.includes("Connect")
    );

    await user.clear(hostInput);
    await user.type(hostInput, "192.168.1.100");
    await user.clear(portInput);
    await user.type(portInput, "35000");

    if (connectButton) {
      await user.click(connectButton);
      expect(onConnectWifi).toHaveBeenCalledWith("192.168.1.100", 35000);
    }
  });

  it("selects WiFi adapter when clicked", async () => {
    const user = userEvent.setup();
    const onConnectWifi = vi.fn();

    render(
      <WiFiSettings
        isConnected={false}
        status="disconnected"
        onConnectWifi={onConnectWifi}
        showToast={vi.fn()}
      />
    );

    const scanButton = screen.getByText("Scan WiFi");
    await user.click(scanButton);

    await screen.findByText("Found Adapters");

    const adapter = screen.getByText(/ELM1/);
    await user.click(adapter);

    const hostInput = screen.getByPlaceholderText("192.168.0.10") as HTMLInputElement;
    expect(hostInput.value).toBe("192.168.0.10");
  });
});
