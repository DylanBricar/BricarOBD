import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import InfoRow from "./InfoRow";
import { Info } from "lucide-react";

describe("InfoRow", () => {
  it("renders label and value", () => {
    render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    expect(screen.getByText("Test Label")).toBeInTheDocument();
    expect(screen.getByText("Test Value")).toBeInTheDocument();
  });

  it("renders icon", () => {
    const { container } = render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    const svgs = container.querySelectorAll("svg");
    expect(svgs.length).toBeGreaterThan(0);
  });

  it("applies mono font when mono is true", () => {
    render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" mono />
    );

    const valueSpan = screen.getByText("Test Value");
    expect(valueSpan.className).toContain("font-mono");
  });

  it("does not apply mono font when mono is false", () => {
    render(
      <InfoRow
        icon={<Info />}
        label="Test Label"
        value="Test Value"
        mono={false}
      />
    );

    const valueSpan = screen.getByText("Test Value").parentElement;
    expect(valueSpan?.className).not.toContain("font-mono");
  });

  it("does not apply mono font when mono is undefined", () => {
    render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    const valueSpan = screen.getByText("Test Value").parentElement;
    expect(valueSpan?.className).not.toContain("font-mono");
  });

  it("renders with flex layout", () => {
    const { container } = render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper.className).toContain("flex");
    expect(wrapper.className).toContain("items-center");
    expect(wrapper.className).toContain("gap-2");
  });

  it("renders with background and border styling", () => {
    const { container } = render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    const wrapper = container.firstChild as HTMLElement;
    expect(wrapper.className).toContain("rounded-lg");
    expect(wrapper.className).toContain("bg-white/[0.02]");
  });

  it("applies text-xs class to label and value", () => {
    render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    expect(screen.getByText("Test Label").className).toContain("text-xs");
    expect(screen.getByText("Test Value").className).toContain("text-xs");
  });

  it("label has muted text color", () => {
    render(
      <InfoRow icon={<Info />} label="Test Label" value="Test Value" />
    );

    const labelSpan = screen.getByText("Test Label");
    expect(labelSpan.className).toContain("text-obd-text-muted");
  });
});
