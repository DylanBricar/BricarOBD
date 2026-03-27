import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Sidebar from "./Sidebar";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en", changeLanguage: vi.fn() },
  }),
}));

vi.mock("@/stores/theme", () => ({
  useThemeStore: () => ({
    mode: "dark" as const,
    setThemeMode: vi.fn(),
  }),
}));

const defaultProps = {
  activePage: "connection",
  onNavigate: vi.fn(),
  connectionStatus: "disconnected" as const,
  canNavigate: false,
  discoveryProgress: 0,
  hasVin: false,
};

describe("Sidebar", () => {
  it("renders navigation items", () => {
    render(<Sidebar {...defaultProps} activePage="dashboard" />);

    expect(screen.getByText("nav.connection")).toBeInTheDocument();
    expect(screen.getByText("nav.dashboard")).toBeInTheDocument();
    expect(screen.getByText("nav.liveData")).toBeInTheDocument();
  });

  it("highlights active page", () => {
    const { container } = render(
      <Sidebar {...defaultProps} activePage="dashboard" connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} />
    );

    const activeButton = Array.from(container.querySelectorAll("button")).find(
      (btn) => btn.textContent?.includes("nav.dashboard")
    );

    expect(activeButton?.className).toContain("nav-item-active");
  });

  it("navigates when nav item is clicked", async () => {
    const onNavigate = vi.fn();
    const user = userEvent.setup();

    render(
      <Sidebar {...defaultProps} activePage="dashboard" onNavigate={onNavigate} connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} />
    );

    const liveDataButton = screen.getByText("nav.liveData");
    await user.click(liveDataButton);

    expect(onNavigate).toHaveBeenCalledWith("liveData");
  });

  it("disables pages when disconnected (except connection)", () => {
    render(<Sidebar {...defaultProps} />);

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(true);
  });

  it("enables navigation when connected with VIN and discovery complete", () => {
    render(
      <Sidebar {...defaultProps} connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} />
    );

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(false);
  });

  it("shows VIN required warning when connected without VIN", () => {
    render(
      <Sidebar {...defaultProps} connectionStatus="connected" canNavigate={false} hasVin={false} />
    );

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(true);
    expect(screen.getByText("nav.tabsLocked")).toBeInTheDocument();
  });

  it("shows discovery progress when connected with VIN but discovery not complete", () => {
    render(
      <Sidebar {...defaultProps} connectionStatus="connected" canNavigate={false} hasVin={true} discoveryProgress={45} />
    );

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(true);
    expect(screen.getByText("nav.tabsLockedDiscovery")).toBeInTheDocument();
  });

  it("displays DTC badge when dtcCount is provided", () => {
    const { container } = render(
      <Sidebar {...defaultProps} activePage="dtc" connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} dtcCount={5} />
    );

    expect(container.textContent).toContain("5");
  });

  it("calls onToggleDevConsole when dev console button is clicked", async () => {
    const onToggleDevConsole = vi.fn();
    const user = userEvent.setup();

    render(
      <Sidebar {...defaultProps} activePage="dashboard" connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} onToggleDevConsole={onToggleDevConsole} />
    );

    const devConsoleButton = screen.getByText("nav.devConsole");
    await user.click(devConsoleButton);

    expect(onToggleDevConsole).toHaveBeenCalled();
  });

  it("renders logo", () => {
    const { container } = render(
      <Sidebar {...defaultProps} connectionStatus="connected" canNavigate={true} hasVin={true} discoveryProgress={100} />
    );

    expect(container.textContent).toContain("BricarOBD");
  });
});
