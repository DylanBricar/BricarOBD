import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import StatusBar from "./StatusBar";
import type { VehicleInfo } from "@/stores/connection";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, _opts?: any) => {
      if (key === "status.connecting") return "Connecting...";
      if (key === "status.connected") return "Connected";
      if (key === "status.demo") return "Demo Mode";
      if (key === "status.disconnected") return "Disconnected";
      if (key === "status.error") return "Error";
      if (key === "nav.polling") return "Polling";
      return key;
    },
    i18n: { language: "en" },
  }),
}));

describe("StatusBar", () => {
  it("renders connected status", () => {
    render(
      <StatusBar
        status="connected"
        vehicle={null}
      />
    );

    expect(screen.getByText("Connected")).toBeInTheDocument();
  });

  it("renders disconnected status", () => {
    render(
      <StatusBar
        status="disconnected"
        vehicle={null}
      />
    );

    expect(screen.getByText("Disconnected")).toBeInTheDocument();
  });

  it("renders demo status", () => {
    render(
      <StatusBar
        status="demo"
        vehicle={null}
      />
    );

    expect(screen.getByText("Demo Mode")).toBeInTheDocument();
  });

  it("renders connecting status", () => {
    render(
      <StatusBar
        status="connecting"
        vehicle={null}
      />
    );

    expect(screen.getByText("Connecting...")).toBeInTheDocument();
  });

  it("displays vehicle information when connected", () => {
    const vehicle: VehicleInfo = {
      vin: "VF3LCBHZ6JS123456",
      make: "Peugeot",
      model: "308",
      year: 2016,
      protocol: "CAN 500k",
      elmVersion: "ELM327 v1.5",
    };

    render(
      <StatusBar
        status="connected"
        vehicle={vehicle}
      />
    );

    expect(screen.getByText("Peugeot 308")).toBeInTheDocument();
    expect(screen.getByText("CAN 500k")).toBeInTheDocument();
    expect(screen.getByText("ELM327 v1.5")).toBeInTheDocument();
  });

  it("displays version number", () => {
    render(
      <StatusBar
        status="connected"
        vehicle={null}
      />
    );

    expect(screen.getByText(/BricarOBD v/)).toBeInTheDocument();
  });

  it("shows polling indicator when isPolling is true", () => {
    render(
      <StatusBar
        status="connected"
        vehicle={null}
        isPolling={true}
      />
    );

    expect(screen.getByText("Polling")).toBeInTheDocument();
  });

  it("hides polling indicator when isPolling is false", () => {
    const { container } = render(
      <StatusBar
        status="connected"
        vehicle={null}
        isPolling={false}
      />
    );

    expect(container.textContent).not.toContain("Polling");
  });

  it("handles vehicle without elmVersion", () => {
    const vehicle: VehicleInfo = {
      vin: "VF3LCBHZ6JS123456",
      make: "Ford",
      model: "Focus",
      year: 2020,
      protocol: "CAN 250k",
    };

    render(
      <StatusBar
        status="connected"
        vehicle={vehicle}
      />
    );

    expect(screen.getByText("Ford Focus")).toBeInTheDocument();
    expect(screen.getByText("CAN 250k")).toBeInTheDocument();
  });

  it("renders footer element", () => {
    const { container } = render(
      <StatusBar
        status="connected"
        vehicle={null}
      />
    );

    expect(container.querySelector("footer")).toBeInTheDocument();
  });
});
