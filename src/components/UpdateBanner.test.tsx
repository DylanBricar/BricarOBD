import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import UpdateBanner from "./UpdateBanner";
import type { UpdateState } from "@/hooks/useAutoUpdate";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: any) =>
      opts?.version ? `${key} ${opts.version}` : key,
    i18n: { language: "en" },
  }),
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

describe("UpdateBanner", () => {
  const mockOnDownload = vi.fn();
  const mockOnDismiss = vi.fn();

  beforeEach(() => {
    mockOnDownload.mockClear();
    mockOnDismiss.mockClear();
  });

  it("returns null for idle status", () => {
    const state: UpdateState = { status: "idle" };
    const { container } = render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("returns null for checking status", () => {
    const state: UpdateState = { status: "checking" };
    const { container } = render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(container.firstChild).toBeNull();
  });

  it("shows up-to-date message", () => {
    const state: UpdateState = { status: "upToDate" };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(screen.getByText("update.upToDate")).toBeInTheDocument();
  });

  it("shows available state with download button", () => {
    const state: UpdateState = { status: "available", version: "2.1.0", body: undefined };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(screen.getByText("update.available 2.1.0")).toBeInTheDocument();
    expect(screen.getByText("update.install")).toBeInTheDocument();
  });

  it("calls onDownload when download button clicked", () => {
    const state: UpdateState = { status: "available", version: "2.1.0", body: undefined };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    const downloadButton = screen.getByText("update.install");
    fireEvent.click(downloadButton);
    expect(mockOnDownload).toHaveBeenCalledTimes(1);
  });

  it("calls onDismiss when dismiss button clicked in available state", () => {
    const state: UpdateState = { status: "available", version: "2.1.0", body: undefined };
    const { container } = render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    const dismissButtons = container.querySelectorAll("button");
    const xButton = Array.from(dismissButtons).find((btn) =>
      btn.querySelector("svg")
    );
    fireEvent.click(xButton!);
    expect(mockOnDismiss).toHaveBeenCalledTimes(1);
  });

  it("shows downloading state", () => {
    const state: UpdateState = { status: "downloading", progress: 0 };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(screen.getByText("update.downloading")).toBeInTheDocument();
  });

  it("shows ready state", () => {
    const state: UpdateState = { status: "ready" };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(screen.getByText("update.restarting")).toBeInTheDocument();
  });

  it("shows error state with dismiss button", () => {
    const state: UpdateState = { status: "error", message: "Network error" };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(screen.getByText("update.error")).toBeInTheDocument();
    const dismissButton = screen.getByRole("button");
    expect(dismissButton).toBeInTheDocument();
  });

  it("calls onDismiss when dismiss button clicked in error state", () => {
    const state: UpdateState = { status: "error", message: "Network error" };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    const dismissButton = screen.getByRole("button");
    fireEvent.click(dismissButton);
    expect(mockOnDismiss).toHaveBeenCalledTimes(1);
  });

  it("does not call onDownload in error state", () => {
    const state: UpdateState = { status: "error", message: "Network error" };
    render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    expect(mockOnDownload).not.toHaveBeenCalled();
  });

  it("up-to-date message has success styling", () => {
    const state: UpdateState = { status: "upToDate" };
    const { container } = render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    const banner = container.querySelector(".bg-obd-success\\/10");
    expect(banner).toBeInTheDocument();
  });

  it("error state has danger styling", () => {
    const state: UpdateState = { status: "error", message: "Network error" };
    const { container } = render(
      <UpdateBanner state={state} onDownload={mockOnDownload} onDismiss={mockOnDismiss} />
    );
    const banner = container.querySelector(".bg-obd-danger\\/10");
    expect(banner).toBeInTheDocument();
  });
});
