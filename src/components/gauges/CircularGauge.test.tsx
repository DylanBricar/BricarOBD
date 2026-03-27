import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import CircularGauge from "./CircularGauge";

describe("CircularGauge", () => {
  it("renders with value, label, and unit", () => {
    const { container } = render(
      <CircularGauge
        value={50}
        min={0}
        max={100}
        label="Engine Temperature"
        unit="°C"
      />
    );

    expect(screen.getByText("Engine Temperature")).toBeInTheDocument();
    expect(screen.getByText("°C")).toBeInTheDocument();
    expect(container.querySelector("svg")).toBeInTheDocument();
  });

  it("displays formatted value with correct decimals", () => {
    const { container } = render(
      <CircularGauge
        value={50.5}
        min={0}
        max={100}
        label="Speed"
        unit="km/h"
        decimals={1}
      />
    );

    expect(container.textContent).toContain("50.5");
  });

  it("rounds value to nearest integer by default", () => {
    const { container } = render(
      <CircularGauge
        value={50.7}
        min={0}
        max={100}
        label="RPM"
        unit="rpm"
      />
    );

    expect(container.textContent).toContain("51");
  });

  it("clamps value between min and max", () => {
    const { container: containerOver } = render(
      <CircularGauge
        value={150}
        min={0}
        max={100}
        label="Test"
        unit="unit"
      />
    );

    expect(containerOver.textContent).toContain("100");

    const { container: containerUnder } = render(
      <CircularGauge
        value={-10}
        min={0}
        max={100}
        label="Test"
        unit="unit"
      />
    );

    expect(containerUnder.textContent).toContain("0");
  });

  it("accepts custom size prop", () => {
    const { container } = render(
      <CircularGauge
        value={50}
        min={0}
        max={100}
        label="Custom Size"
        unit="unit"
        size={200}
      />
    );

    const svg = container.querySelector("svg");
    expect(svg).toHaveAttribute("width", "200");
    expect(svg).toHaveAttribute("height", "200");
  });

  it("renders SVG with required elements", () => {
    const { container } = render(
      <CircularGauge
        value={50}
        min={0}
        max={100}
        label="Test"
        unit="unit"
      />
    );

    const circles = container.querySelectorAll("circle");
    expect(circles.length).toBeGreaterThan(0);

    const texts = container.querySelectorAll("text");
    expect(texts.length).toBeGreaterThan(0);
  });

  it("applies custom className", () => {
    const { container } = render(
      <CircularGauge
        value={50}
        min={0}
        max={100}
        label="Test"
        unit="unit"
        className="custom-class"
      />
    );

    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper.className).toContain("custom-class");
  });
});
