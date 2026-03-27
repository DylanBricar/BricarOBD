import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { Toast } from "./Toast";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => key,
    i18n: { language: "en" },
  }),
}));

describe("Toast", () => {
  it("renders success toast with message", () => {
    const onDismiss = vi.fn();
    render(
      <Toast
        message="Operation successful"
        type="success"
        onDismiss={onDismiss}
      />
    );

    expect(screen.getByText("Operation successful")).toBeInTheDocument();
  });

  it("renders error toast with message", () => {
    const onDismiss = vi.fn();
    render(
      <Toast
        message="An error occurred"
        type="error"
        onDismiss={onDismiss}
      />
    );

    expect(screen.getByText("An error occurred")).toBeInTheDocument();
  });

  it("calls onDismiss when close button is clicked", async () => {
    const onDismiss = vi.fn();
    const user = userEvent.setup();
    render(
      <Toast
        message="Test message"
        type="success"
        onDismiss={onDismiss}
      />
    );

    const closeButton = screen.getByLabelText("common.close");
    await user.click(closeButton);

    expect(onDismiss).toHaveBeenCalledOnce();
  });

  it("applies correct styling for success type", () => {
    const { container } = render(
      <Toast
        message="Success"
        type="success"
        onDismiss={vi.fn()}
      />
    );

    const toastDiv = container.firstChild as HTMLElement;
    expect(toastDiv.className).toContain("bg-obd-success");
  });

  it("applies correct styling for error type", () => {
    const { container } = render(
      <Toast
        message="Error"
        type="error"
        onDismiss={vi.fn()}
      />
    );

    const toastDiv = container.firstChild as HTMLElement;
    expect(toastDiv.className).toContain("bg-obd-danger");
  });
});
