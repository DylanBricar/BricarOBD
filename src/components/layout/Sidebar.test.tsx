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

vi.mock("@/lib/units", () => ({
  useUnitSystem: () => ({
    system: "metric" as const,
    setUnitSystem: vi.fn(),
  }),
}));

describe("Sidebar", () => {
  it("renders navigation items", () => {
    const onNavigate = vi.fn();

    render(
      <Sidebar
        activePage="dashboard"
        onNavigate={onNavigate}
        connectionStatus="disconnected"
      />
    );

    expect(screen.getByText("nav.connection")).toBeInTheDocument();
    expect(screen.getByText("nav.dashboard")).toBeInTheDocument();
    expect(screen.getByText("nav.liveData")).toBeInTheDocument();
  });

  it("highlights active page", () => {
    const onNavigate = vi.fn();

    const { container } = render(
      <Sidebar
        activePage="dashboard"
        onNavigate={onNavigate}
        connectionStatus="connected"
      />
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
      <Sidebar
        activePage="dashboard"
        onNavigate={onNavigate}
        connectionStatus="connected"
      />
    );

    const liveDataButton = screen.getByText("nav.liveData");
    await user.click(liveDataButton);

    expect(onNavigate).toHaveBeenCalledWith("liveData");
  });

  it("disables pages when disconnected (except connection and history)", () => {
    const onNavigate = vi.fn();

    render(
      <Sidebar
        activePage="connection"
        onNavigate={onNavigate}
        connectionStatus="disconnected"
      />
    );

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(true);
  });

  it("enables navigation when connected", () => {
    const onNavigate = vi.fn();

    render(
      <Sidebar
        activePage="connection"
        onNavigate={onNavigate}
        connectionStatus="connected"
      />
    );

    const liveDataButton = Array.from(screen.getAllByRole("button")).find(
      (btn) => btn.textContent?.includes("nav.liveData")
    ) as HTMLButtonElement;

    expect(liveDataButton.disabled).toBe(false);
  });

  it("displays DTC badge when dtcCount is provided", () => {
    const onNavigate = vi.fn();

    const { container } = render(
      <Sidebar
        activePage="dtc"
        onNavigate={onNavigate}
        connectionStatus="connected"
        dtcCount={5}
      />
    );

    expect(container.textContent).toContain("5");
  });

  it("calls onToggleDevConsole when dev console button is clicked", async () => {
    const onNavigate = vi.fn();
    const onToggleDevConsole = vi.fn();
    const user = userEvent.setup();

    render(
      <Sidebar
        activePage="dashboard"
        onNavigate={onNavigate}
        connectionStatus="connected"
        onToggleDevConsole={onToggleDevConsole}
      />
    );

    const devConsoleButton = screen.getByText("nav.devConsole");
    await user.click(devConsoleButton);

    expect(onToggleDevConsole).toHaveBeenCalled();
  });

  it("renders logo", () => {
    const onNavigate = vi.fn();

    const { container } = render(
      <Sidebar
        activePage="dashboard"
        onNavigate={onNavigate}
        connectionStatus="connected"
      />
    );

    expect(container.textContent).toContain("BricarOBD");
  });
});
