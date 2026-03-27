import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import ConfirmDialog from "./ConfirmDialog";
import type { AdvancedOperation } from "./types";

describe("ConfirmDialog", () => {
  const mockOnCancel = vi.fn();
  const mockOnConfirm = vi.fn();
  const mockT = (key: string) => key;

  beforeEach(() => {
    mockOnCancel.mockClear();
    mockOnConfirm.mockClear();
  });

  it("returns null when show is false", () => {
    const { container } = render(
      <ConfirmDialog
        show={false}
        pendingOperation={null}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders dialog when show is true", () => {
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    expect(screen.getByText("advanced.confirmDialog.title")).toBeInTheDocument();
  });

  it("renders fixed overlay with correct styling", () => {
    const { container } = render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    const overlay = container.querySelector(".fixed.inset-0");
    expect(overlay).toBeInTheDocument();
  });

  it("renders glass-card styling", () => {
    const { container } = render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    const card = container.querySelector(".glass-card");
    expect(card).toBeInTheDocument();
  });

  it("shows operation name when type is operation", () => {
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "write", name: "WriteValue", description: "Write value", risk_level: "high" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    expect(screen.getByText("WriteValue")).toBeInTheDocument();
  });

  it("shows raw command when type is raw", () => {
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "raw", cmd: "2210A1FF" }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    expect(screen.getByText("2210A1FF")).toBeInTheDocument();
  });

  it("renders AlertTriangle icon with obd-warning styling", () => {
    const { container } = render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    const icon = container.querySelector(".text-obd-warning");
    expect(icon).toBeInTheDocument();
  });

  it("renders message text", () => {
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    expect(screen.getByText("advanced.confirmDialog.message")).toBeInTheDocument();
  });

  it("calls onCancel when cancel button is clicked", async () => {
    const user = userEvent.setup();
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    await user.click(screen.getByText("advanced.confirmDialog.cancel"));
    expect(mockOnCancel).toHaveBeenCalledOnce();
  });

  it("calls onConfirm when confirm button is clicked", async () => {
    const user = userEvent.setup();
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    await user.click(screen.getByText("advanced.confirmDialog.confirm"));
    expect(mockOnConfirm).toHaveBeenCalledOnce();
  });

  it("confirms without arguments", async () => {
    const user = userEvent.setup();
    render(
      <ConfirmDialog
        show={true}
        pendingOperation={{ type: "operation", op: { id: "test", name: "TestOp", description: "Test operation", risk_level: "medium" } as AdvancedOperation }}
        onCancel={mockOnCancel}
        onConfirm={mockOnConfirm}
        t={mockT}
      />
    );
    await user.click(screen.getByText("advanced.confirmDialog.confirm"));
    expect(mockOnConfirm).toHaveBeenCalled();
  });
});
