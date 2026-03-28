import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DtcClearConfirm from "./DtcClearConfirm";

describe("DtcClearConfirm", () => {
  const mockT = (key: string) => key;
  const mockOnConfirm = vi.fn();
  const mockOnCancel = vi.fn();

  beforeEach(() => {
    mockOnConfirm.mockClear();
    mockOnCancel.mockClear();
  });

  it("renders dialog with alertdialog role", () => {
    const { container } = render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const dialog = container.querySelector('[role="alertdialog"]');
    expect(dialog).toBeInTheDocument();
  });

  it("shows confirm title", () => {
    render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText("dtc.confirmClear")).toBeInTheDocument();
  });

  it("shows confirm message", () => {
    render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    expect(screen.getByText("dtc.confirmClearMsg")).toBeInTheDocument();
  });

  it("calls onConfirm when confirm button clicked", () => {
    render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const confirmButton = screen.getByText("common.confirm");
    fireEvent.click(confirmButton);
    expect(mockOnConfirm).toHaveBeenCalledTimes(1);
  });

  it("calls onCancel when cancel button clicked", () => {
    render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const cancelButton = screen.getByText("common.cancel");
    fireEvent.click(cancelButton);
    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it("calls onCancel when backdrop clicked", () => {
    const { container } = render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const backdrop = container.querySelector('[role="alertdialog"]');
    fireEvent.click(backdrop!);
    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it("does not call onCancel when card content clicked", () => {
    const { container } = render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const card = container.querySelector(".glass-card");
    fireEvent.click(card!);
    expect(mockOnCancel).not.toHaveBeenCalled();
  });

  it("renders with aria-modal true", () => {
    const { container } = render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const dialog = container.querySelector('[role="alertdialog"]');
    expect(dialog).toHaveAttribute("aria-modal", "true");
  });

  it("has aria-labelledby pointing to title", () => {
    const { container } = render(
      <DtcClearConfirm t={mockT} onConfirm={mockOnConfirm} onCancel={mockOnCancel} />
    );
    const dialog = container.querySelector('[role="alertdialog"]');
    expect(dialog).toHaveAttribute("aria-labelledby", "dtc-confirm-title");
    const title = container.querySelector("#dtc-confirm-title");
    expect(title).toBeInTheDocument();
  });
});
