import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import ManualVinInput from "./ManualVinInput";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(async (cmd: string, opts?: any) => {
    if (cmd === "set_manual_vin") {
      return {
        vin: opts.vin,
        make: "Peugeot",
        model: "308",
        year: 2016,
        protocol: "CAN 500k",
      };
    }
    return null;
  }),
}));

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string, opts?: any) => {
      const keys: Record<string, string> = {
        "connection.manualVin": "Manual VIN",
        "connection.vinLength": `Length: ${opts?.current || 0}/17`,
        "connection.vinInvalidChars": "VIN contains invalid characters (I, O, Q)",
        "connection.vinInvalid": `Invalid VIN (${opts?.count || 0} chars)`,
        "connection.vin": "VIN",
        "common.ok": "OK",
        "common.error": "Error",
      };
      return keys[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

describe("ManualVinInput", () => {
  it("renders input field with label", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value=""
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    expect(screen.getByText("Manual VIN")).toBeInTheDocument();
    expect(screen.getByPlaceholderText("VF3LCBHZ6JS123456")).toBeInTheDocument();
  });

  it("converts input to uppercase", async () => {
    const onChange = vi.fn();
    const user = userEvent.setup();

    render(
      <ManualVinInput
        value=""
        onChange={onChange}
        showToast={vi.fn()}
      />
    );

    const input = screen.getByPlaceholderText("VF3LCBHZ6JS123456") as HTMLInputElement;
    await user.type(input, "abc");

    expect(onChange).toHaveBeenCalledWith(expect.stringContaining("A"));
  });

  it("shows character count", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value="VF3LCBHZ6JS123456"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    expect(screen.getByText("Length: 17/17")).toBeInTheDocument();
  });

  it("validates VIN length", () => {
    const showToast = vi.fn();
    const { container } = render(
      <ManualVinInput
        value="ABC"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const input = container.querySelector("input") as HTMLInputElement;
    expect(input.className).toContain("border-obd-danger");
  });

  it("shows warning for invalid characters I, O, Q", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value="VF3LCBHZ6JSIQ3456"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    expect(screen.getByText("VIN contains invalid characters (I, O, Q)")).toBeInTheDocument();
  });

  it("disables OK button for invalid VIN", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value="SHORT"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const okButton = screen.getByRole("button", { name: "OK" }) as HTMLButtonElement;
    expect(okButton.disabled).toBe(true);
  });

  it("enables OK button for valid VIN", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value="VF3LCBHZ6JS123456"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const okButton = screen.getByRole("button", { name: "OK" }) as HTMLButtonElement;
    expect(okButton.disabled).toBe(false);
  });

  it("calls showToast with success message on valid VIN submit", async () => {
    const showToast = vi.fn();
    const user = userEvent.setup();

    render(
      <ManualVinInput
        value="VF3LCBHZ6JS123456"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const okButton = screen.getByRole("button", { name: "OK" });
    await user.click(okButton);

    expect(showToast).toHaveBeenCalled();
  });

  it("calls onVehicleUpdate callback when VIN is valid", async () => {
    const onVehicleUpdate = vi.fn();
    const user = userEvent.setup();

    render(
      <ManualVinInput
        value="VF3LCBHZ6JS123456"
        onChange={vi.fn()}
        onVehicleUpdate={onVehicleUpdate}
        showToast={vi.fn()}
      />
    );

    const okButton = screen.getByRole("button", { name: "OK" });
    await user.click(okButton);

    expect(onVehicleUpdate).toHaveBeenCalled();
  });

  it("shows error toast for empty VIN on submit", async () => {
    const showToast = vi.fn();

    render(
      <ManualVinInput
        value=""
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const okButton = screen.getByRole("button", { name: "OK" });
    expect((okButton as HTMLButtonElement).disabled).toBe(true);
  });

  it("enforces maxLength of 17", () => {
    const showToast = vi.fn();
    render(
      <ManualVinInput
        value="VF3LCBHZ6JS123456"
        onChange={vi.fn()}
        showToast={showToast}
      />
    );

    const input = screen.getByPlaceholderText("VF3LCBHZ6JS123456") as HTMLInputElement;
    expect(input.maxLength).toBe(17);
  });
});
