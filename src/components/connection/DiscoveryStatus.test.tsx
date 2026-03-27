import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import DiscoveryStatus from "./DiscoveryStatus";
import type { VehicleInfo } from "@/stores/connection";

describe("DiscoveryStatus", () => {
  const mockT = (key: string) => key;

  const mockVehicle: VehicleInfo = {
    vin: "1HGCV41JXMN109186",
    make: "Honda",
    model: "Civic",
    year: 2012,
    protocol: "ISO 15765-4 (CAN 11bit 500k)",
    elmVersion: "ELM327 v1.5a",
  };

  it("renders nothing when disconnected", () => {
    const { container } = render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"disconnected" as const}
        vehicle={null}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(container.firstChild).toBeNull();
  });

  it("shows progress when connected with VIN and not complete", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={50}
        isDiscoveryComplete={false}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.getByText("connection.analyzing")).toBeInTheDocument();
  });

  it("shows progress bar with correct width percentage", () => {
    const { container } = render(
      <DiscoveryStatus
        discoveryProgress={75}
        isDiscoveryComplete={false}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    const progressFill = container.querySelector('[style*="width"]');
    expect(progressFill).toBeInTheDocument();
  });

  it("shows VIN required when connected with empty VIN", () => {
    const vehicleNoVin = { ...mockVehicle, vin: "" };

    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={vehicleNoVin}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.getByText("connection.vinRequired")).toBeInTheDocument();
  });

  it("hides VIN required when VIN is present", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.vinRequired")).not.toBeInTheDocument();
  });

  it("shows clear cache button when hasVinCache is true and connected", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={true}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.getByText("connection.clearCache")).toBeInTheDocument();
  });

  it("hides clear cache button when no cache", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.clearCache")).not.toBeInTheDocument();
  });

  it("hides clear cache button when disconnected", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"disconnected" as const}
        vehicle={mockVehicle}
        hasVinCache={true}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.clearCache")).not.toBeInTheDocument();
  });

  it("calls onClearCache on button click", async () => {
    const onClearCache = vi.fn();
    const user = userEvent.setup();

    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={true}
        onClearCache={onClearCache}
        t={mockT}
      />
    );

    const clearButton = screen.getByText("connection.clearCache");
    await user.click(clearButton);

    expect(onClearCache).toHaveBeenCalled();
  });

  it("hides progress when discovery is complete", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={100}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={mockVehicle}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.analyzing")).not.toBeInTheDocument();
  });

  it("hides progress when no vehicle", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={50}
        isDiscoveryComplete={false}
        status={"connected" as const}
        vehicle={null}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.analyzing")).not.toBeInTheDocument();
  });

  it("hides VIN alert when vehicle is null", () => {
    render(
      <DiscoveryStatus
        discoveryProgress={0}
        isDiscoveryComplete={true}
        status={"connected" as const}
        vehicle={null}
        hasVinCache={false}
        onClearCache={vi.fn()}
        t={mockT}
      />
    );

    expect(screen.queryByText("connection.vinRequired")).not.toBeInTheDocument();
  });
});
