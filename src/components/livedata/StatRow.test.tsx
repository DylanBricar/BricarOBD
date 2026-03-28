import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import { StatRow } from "./StatRow";

describe("StatRow", () => {
  it("renders label", () => {
    render(<StatRow label="RPM" value="2500" unit="rpm" />);
    expect(screen.getByText("RPM")).toBeInTheDocument();
  });

  it("renders value", () => {
    render(<StatRow label="RPM" value="2500" unit="rpm" />);
    expect(screen.getByText("2500")).toBeInTheDocument();
  });

  it("renders unit", () => {
    render(<StatRow label="RPM" value="2500" unit="rpm" />);
    expect(screen.getByText("rpm")).toBeInTheDocument();
  });

  it("renders label, value, and unit together", () => {
    render(<StatRow label="Temperature" value="85" unit="°C" />);
    expect(screen.getByText("Temperature")).toBeInTheDocument();
    expect(screen.getByText("85")).toBeInTheDocument();
    expect(screen.getByText("°C")).toBeInTheDocument();
  });

  it("displays value and unit in a monospace font", () => {
    const { container } = render(<StatRow label="Speed" value="60" unit="km/h" />);
    const valueSpan = container.querySelector(".font-mono");
    expect(valueSpan).toBeInTheDocument();
    expect(valueSpan).toHaveTextContent("60 km/h");
  });

  it("layout has label on left and value on right", () => {
    const { container } = render(<StatRow label="Left" value="100" unit="unit" />);
    const flexDiv = container.querySelector(".flex.items-center.justify-between");
    expect(flexDiv).toBeInTheDocument();
  });

  it("has proper vertical padding", () => {
    const { container } = render(<StatRow label="Test" value="1" unit="x" />);
    const flexDiv = container.querySelector(".py-1\\.5");
    expect(flexDiv).toBeInTheDocument();
  });
});
