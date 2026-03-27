import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ConnectionTypeSelector from "./ConnectionTypeSelector";

describe("ConnectionTypeSelector", () => {
  const mockT = (key: string) => {
    const translations: Record<string, string> = {
      "connection.usb": "USB",
      "connection.wifi": "WiFi",
    };
    return translations[key] || key;
  };

  it("renders USB and WiFi buttons", () => {
    render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    expect(screen.getByText("USB")).toBeInTheDocument();
    expect(screen.getByText("WiFi")).toBeInTheDocument();
  });

  it("highlights active USB tab with bg-obd-accent", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    const usbButton = buttons[0];
    expect(usbButton.className).toContain("bg-obd-accent");
  });

  it("highlights active WiFi tab with bg-obd-accent", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="wifi"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    const wifiButton = buttons[1];
    expect(wifiButton.className).toContain("bg-obd-accent");
  });

  it("calls onTypeChange when USB is clicked", async () => {
    const onTypeChange = vi.fn();
    const user = userEvent.setup();

    const { container } = render(
      <ConnectionTypeSelector
        connectionType="wifi"
        onTypeChange={onTypeChange}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    await user.click(buttons[0]);

    expect(onTypeChange).toHaveBeenCalledWith("usb");
  });

  it("calls onTypeChange when WiFi is clicked", async () => {
    const onTypeChange = vi.fn();
    const user = userEvent.setup();

    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={onTypeChange}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    await user.click(buttons[1]);

    expect(onTypeChange).toHaveBeenCalledWith("wifi");
  });

  it("disables buttons when isConnected is true", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={true}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    buttons.forEach((button) => {
      expect(button).toBeDisabled();
    });
  });

  it("does not disable buttons when isConnected is false", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    buttons.forEach((button) => {
      expect(button).not.toBeDisabled();
    });
  });

  it("shows Android USB button when isAndroid is true", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={true}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(4);
  });

  it("hides Android USB button when isAndroid is false", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    expect(buttons.length).toBe(2);
  });

  it("calls onTypeChange with usb_android when Android button is clicked", async () => {
    const onTypeChange = vi.fn();
    const user = userEvent.setup();

    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={onTypeChange}
        isConnected={false}
        isAndroid={true}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    await user.click(buttons[2]);

    expect(onTypeChange).toHaveBeenCalledWith("usb_android");
  });

  it("applies opacity-50 when disabled", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={true}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    buttons.forEach((button) => {
      expect(button.className).toContain("opacity-50");
    });
  });

  it("has border styling", () => {
    const { container } = render(
      <ConnectionTypeSelector
        connectionType="usb"
        onTypeChange={vi.fn()}
        isConnected={false}
        isAndroid={false}
        t={mockT}
      />
    );

    const buttons = container.querySelectorAll("button");
    buttons.forEach((button) => {
      expect(button.className).toContain("border");
    });
  });
});
