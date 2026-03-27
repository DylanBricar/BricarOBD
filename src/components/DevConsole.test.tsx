import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import DevConsole from "@/components/DevConsole";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("DevConsole", () => {
  const mockOnClose = vi.fn();

  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("renders the console container", () => {
    const { container } = render(<DevConsole onClose={mockOnClose} />);
    expect(container.firstChild).toBeTruthy();
  });

  it("shows title", () => {
    render(<DevConsole onClose={mockOnClose} />);
    expect(screen.getByText("devConsole.title")).toBeInTheDocument();
  });

  it("shows filter input", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const input = screen.getByPlaceholderText("devConsole.filter");
    expect(input).toBeInTheDocument();
  });

  it("shows waiting message when no logs", () => {
    render(<DevConsole onClose={mockOnClose} />);
    expect(screen.getByText("devConsole.waitingForLogs")).toBeInTheDocument();
  });

  it("has pause/resume button", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const pauseBtn = screen.getByLabelText("devConsole.pause");
    expect(pauseBtn).toBeInTheDocument();
  });

  it("has auto-scroll button", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const scrollBtn = screen.getByLabelText("devConsole.autoScroll");
    expect(scrollBtn).toBeInTheDocument();
  });

  it("has copy button", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const copyBtn = screen.getByLabelText("devConsole.copyLogs");
    expect(copyBtn).toBeInTheDocument();
  });

  it("has clear button", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const clearBtn = screen.getByLabelText("devConsole.clearLogs");
    expect(clearBtn).toBeInTheDocument();
  });

  it("shows close button when not standalone", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const closeBtn = screen.getByLabelText("common.close");
    expect(closeBtn).toBeInTheDocument();
  });

  it("hides close button when standalone", () => {
    render(<DevConsole isStandalone onClose={mockOnClose} />);
    expect(screen.queryByLabelText("common.close")).not.toBeInTheDocument();
  });

  it("calls onClose when close button clicked", () => {
    render(<DevConsole onClose={mockOnClose} />);
    fireEvent.click(screen.getByLabelText("common.close"));
    expect(mockOnClose).toHaveBeenCalledOnce();
  });

  it("toggles pause state on button click", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const pauseBtn = screen.getByLabelText("devConsole.pause");
    fireEvent.click(pauseBtn);
    // After click, label changes to "resume"
    expect(screen.getByLabelText("devConsole.resume")).toBeInTheDocument();
  });

  it("calls invoke on clear", () => {
    render(<DevConsole onClose={mockOnClose} />);
    fireEvent.click(screen.getByLabelText("devConsole.clearLogs"));
    expect(invoke).toHaveBeenCalledWith("clear_dev_logs");
  });

  it("accepts filter text input", () => {
    render(<DevConsole onClose={mockOnClose} />);
    const input = screen.getByPlaceholderText("devConsole.filter") as HTMLInputElement;
    fireEvent.change(input, { target: { value: "ERROR" } });
    expect(input.value).toBe("ERROR");
  });

  it("shows status bar with live/paused info", () => {
    render(<DevConsole onClose={mockOnClose} />);
    expect(screen.getByText("devConsole.live")).toBeInTheDocument();
  });

  it("fetches initial logs on mount", () => {
    render(<DevConsole onClose={mockOnClose} />);
    expect(invoke).toHaveBeenCalledWith("get_dev_logs");
  });
});
