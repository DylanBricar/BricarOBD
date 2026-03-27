import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen } from "@testing-library/react";
import ErrorBoundary from "./ErrorBoundary";

vi.mock("i18next", () => ({
  default: {
    t: (key: string) => key,
    language: "en",
  },
}));

const ThrowError = () => {
  throw new Error("Test error");
};

describe("ErrorBoundary", () => {
  beforeEach(() => {
    vi.spyOn(console, "error").mockImplementation(() => {});
  });

  it("renders children when there is no error", () => {
    render(
      <ErrorBoundary>
        <div>Test content</div>
      </ErrorBoundary>
    );

    expect(screen.getByText("Test content")).toBeInTheDocument();
  });

  it("renders error UI when error is caught", () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText("common.errorBoundaryTitle")).toBeInTheDocument();
    expect(screen.getByText("common.errorBoundaryMessage")).toBeInTheDocument();
  });

  it("displays error message in error UI", () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(screen.getByText("Test error")).toBeInTheDocument();
  });

  it("displays reload button", () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    const button = screen.getByRole("button");
    expect(button.textContent).toContain("common.errorBoundaryReload");
  });

  it("reload button is present and functional", async () => {
    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    const button = screen.getByRole("button");
    expect(button).toBeInTheDocument();
    expect(button.textContent).toContain("common.errorBoundaryReload");
  });

  it("logs error to console", () => {
    const consoleErrorSpy = vi.spyOn(console, "error");

    render(
      <ErrorBoundary>
        <ThrowError />
      </ErrorBoundary>
    );

    expect(consoleErrorSpy).toHaveBeenCalledWith(
      "[BricarOBD] Error caught by boundary:",
      expect.any(Error)
    );
  });
});
