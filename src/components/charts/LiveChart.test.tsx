import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import LiveChart from "./LiveChart";

// Mock ResizeObserver
global.ResizeObserver = vi.fn().mockImplementation(() => ({
  observe: vi.fn(),
  unobserve: vi.fn(),
  disconnect: vi.fn(),
}));

// Mock recharts ResponsiveContainer to avoid rendering issues
vi.mock("recharts", () => ({
  AreaChart: ({ children }: any) => <div data-testid="area-chart">{children}</div>,
  Area: () => <div data-testid="area" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  ResponsiveContainer: ({ children }: any) => <div data-testid="responsive-container">{children}</div>,
  Tooltip: () => <div data-testid="tooltip" />,
}));

describe("LiveChart", () => {
  it("renders chart container", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test Chart"
        unit="unit"
      />
    );

    expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
  });

  it("displays label", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Engine Temperature"
        unit="°C"
      />
    );

    expect(screen.getByText("Engine Temperature")).toBeInTheDocument();
  });

  it("displays current value and unit", () => {
    render(
      <LiveChart
        data={[10, 20, 50]}
        label="Speed"
        unit="km/h"
      />
    );

    expect(screen.getByText(/50\.0/)).toBeInTheDocument();
    expect(screen.getByText("km/h")).toBeInTheDocument();
  });

  it("handles empty data", () => {
    render(
      <LiveChart
        data={[]}
        label="Empty Chart"
        unit="unit"
      />
    );

    expect(screen.getByText("Empty Chart")).toBeInTheDocument();
    expect(screen.getByText(/0\.0/)).toBeInTheDocument();
  });

  it("displays glass-card styling", () => {
    const { container } = render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
      />
    );

    const card = container.querySelector(".glass-card");
    expect(card).toBeInTheDocument();
  });

  it("applies custom className", () => {
    const { container } = render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
        className="custom-class"
      />
    );

    const card = container.querySelector(".glass-card");
    expect(card?.className).toContain("custom-class");
  });

  it("accepts custom color", () => {
    const { container } = render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
        color="rgb(255, 0, 0)"
      />
    );

    expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
  });

  it("accepts custom height", () => {
    const { container } = render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
        height={200}
      />
    );

    expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
  });

  it("renders area chart component", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
      />
    );

    expect(screen.getByTestId("area-chart")).toBeInTheDocument();
  });

  it("renders tooltip", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
      />
    );

    expect(screen.getByTestId("tooltip")).toBeInTheDocument();
  });

  it("renders with showAxis false by default", () => {
    const { container } = render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
      />
    );

    expect(screen.queryByTestId("x-axis")).not.toBeInTheDocument();
    expect(screen.queryByTestId("y-axis")).not.toBeInTheDocument();
  });

  it("renders axes when showAxis is true", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
        showAxis={true}
      />
    );

    expect(screen.getByTestId("x-axis")).toBeInTheDocument();
    expect(screen.getByTestId("y-axis")).toBeInTheDocument();
  });

  it("formats current value to one decimal place", () => {
    render(
      <LiveChart
        data={[10.5, 20.7, 30.2]}
        label="Test"
        unit="unit"
      />
    );

    expect(screen.getByText(/30\.2/)).toBeInTheDocument();
  });

  it("accepts minDomain and maxDomain props", () => {
    render(
      <LiveChart
        data={[10, 20, 30]}
        label="Test"
        unit="unit"
        minDomain={0}
        maxDomain={100}
      />
    );

    expect(screen.getByTestId("responsive-container")).toBeInTheDocument();
  });

  it("handles single data point", () => {
    render(
      <LiveChart
        data={[50]}
        label="Single Point"
        unit="unit"
      />
    );

    expect(screen.getByText(/50\.0/)).toBeInTheDocument();
  });
});
