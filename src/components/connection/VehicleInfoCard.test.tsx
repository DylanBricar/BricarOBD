import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import VehicleInfoCard from "./VehicleInfoCard";
import type { VehicleInfo } from "@/stores/connection";

vi.mock("./InfoRow", () => ({
  default: ({ label, value }: any) => (
    <div data-testid={`info-row-${label}`}>
      {label}: {value}
    </div>
  ),
}));

describe("VehicleInfoCard", () => {
  const mockT = (key: string) => key;
  const mockVehicle: VehicleInfo = {
    make: "Toyota",
    model: "Camry",
    year: 2020,
    vin: "JTDBLRV30LD123456",
    protocol: "ISO-TP",
    elmVersion: "1.5",
  };

  it("shows disconnected message when vehicle is null", () => {
    render(<VehicleInfoCard vehicle={null} status="connected" t={mockT} />);
    expect(screen.getByText("connection.disconnected")).toBeInTheDocument();
  });

  it("displays vehicle make and model", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(screen.getByText("Toyota Camry")).toBeInTheDocument();
  });

  it("displays vehicle year", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(screen.getByText("2020")).toBeInTheDocument();
  });

  it("displays VIN with InfoRow", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(screen.getByTestId("info-row-connection.vin")).toBeInTheDocument();
  });

  it("displays protocol with InfoRow", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(
      screen.getByTestId("info-row-connection.protocol")
    ).toBeInTheDocument();
  });

  it("displays ELM version with InfoRow", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(
      screen.getByTestId("info-row-connection.elmVersion")
    ).toBeInTheDocument();
  });

  it("shows error message when status is error", () => {
    render(<VehicleInfoCard vehicle={mockVehicle} status="error" t={mockT} />);
    expect(screen.getByText("connection.errorMessage")).toBeInTheDocument();
  });

  it("does not show error message when status is connected", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(screen.queryByText("connection.errorMessage")).not.toBeInTheDocument();
  });

  it("shows title", () => {
    render(
      <VehicleInfoCard vehicle={mockVehicle} status="connected" t={mockT} />
    );
    expect(screen.getByText("connection.vehicle")).toBeInTheDocument();
  });
});
