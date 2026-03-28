import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DemoModeButton from "./DemoModeButton";

describe("DemoModeButton", () => {
  const mockT = (key: string) => key;
  const mockOnClick = vi.fn();

  beforeEach(() => {
    mockOnClick.mockClear();
  });

  it("renders demo button text", () => {
    render(
      <DemoModeButton isConnected={false} onClick={mockOnClick} t={mockT} />
    );
    expect(screen.getByText("connection.demo")).toBeInTheDocument();
  });

  it("renders demo description text", () => {
    render(
      <DemoModeButton isConnected={false} onClick={mockOnClick} t={mockT} />
    );
    expect(screen.getByText("connection.demoDesc")).toBeInTheDocument();
  });

  it("button is enabled when not connected", () => {
    render(
      <DemoModeButton isConnected={false} onClick={mockOnClick} t={mockT} />
    );
    const button = screen.getByRole("button");
    expect(button).not.toBeDisabled();
  });

  it("button is disabled when connected", () => {
    render(
      <DemoModeButton isConnected={true} onClick={mockOnClick} t={mockT} />
    );
    const button = screen.getByRole("button");
    expect(button).toBeDisabled();
  });

  it("calls onClick when clicked and enabled", () => {
    render(
      <DemoModeButton isConnected={false} onClick={mockOnClick} t={mockT} />
    );
    const button = screen.getByRole("button");
    fireEvent.click(button);
    expect(mockOnClick).toHaveBeenCalledTimes(1);
  });

  it("does not call onClick when disabled", () => {
    render(
      <DemoModeButton isConnected={true} onClick={mockOnClick} t={mockT} />
    );
    const button = screen.getByRole("button");
    fireEvent.click(button);
    expect(mockOnClick).not.toHaveBeenCalled();
  });

  it("applies disabled styling when connected", () => {
    const { container } = render(
      <DemoModeButton isConnected={true} onClick={mockOnClick} t={mockT} />
    );
    const button = container.querySelector("button");
    expect(button).toHaveClass("opacity-30", "cursor-not-allowed");
  });
});
