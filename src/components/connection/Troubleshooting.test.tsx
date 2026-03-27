import { describe, it, expect, vi } from "vitest";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import Troubleshooting from "./Troubleshooting";

vi.mock("react-i18next", () => ({
  useTranslation: () => ({
    t: (key: string) => {
      const keys: Record<string, string> = {
        "connection.troubleshoot.title": "Connection Issues?",
        "connection.troubleshoot.tip1": "Tip 1",
        "connection.troubleshoot.tip2": "Tip 2",
        "connection.troubleshoot.tip3": "Tip 3",
        "connection.troubleshoot.tip4": "Tip 4",
        "connection.troubleshoot.tip5": "Tip 5",
        "connection.troubleshoot.tip6": "Tip 6",
        "connection.troubleshoot.tip7": "Tip 7",
        "connection.troubleshoot.tip8": "Tip 8",
        "connection.troubleshoot.tip9": "Tip 9",
        "connection.troubleshoot.tip10": "Tip 10",
        "connection.troubleshoot.tip11": "Tip 11",
      };
      return keys[key] || key;
    },
    i18n: { language: "en" },
  }),
}));

describe("Troubleshooting", () => {
  it("renders title", () => {
    render(<Troubleshooting onClose={vi.fn()} />);

    expect(screen.getByText("Connection Issues?")).toBeInTheDocument();
  });

  it("renders all tips", () => {
    render(<Troubleshooting onClose={vi.fn()} />);

    for (let i = 1; i <= 11; i++) {
      expect(screen.getByText(`Tip ${i}`)).toBeInTheDocument();
    }
  });

  it("renders tips as ordered list", () => {
    const { container } = render(<Troubleshooting onClose={vi.fn()} />);

    const ol = container.querySelector("ol");
    expect(ol).toBeInTheDocument();
    expect(ol?.className).toContain("list-decimal");
  });

  it("calls onClose when clicked", async () => {
    const onClose = vi.fn();
    const user = userEvent.setup();

    render(<Troubleshooting onClose={onClose} />);

    const button = screen.getByRole("button");
    await user.click(button);

    expect(onClose).toHaveBeenCalled();
  });

  it("renders as glass card", () => {
    const { container } = render(<Troubleshooting onClose={vi.fn()} />);

    expect(container.querySelector(".glass-card")).toBeInTheDocument();
  });

  it("displays warning icon", () => {
    const { container } = render(<Troubleshooting onClose={vi.fn()} />);

    const svgs = container.querySelectorAll("svg");
    expect(svgs.length).toBeGreaterThan(0);
  });

  it("renders list items with proper styling", () => {
    const { container } = render(<Troubleshooting onClose={vi.fn()} />);

    const listItems = container.querySelectorAll("li");
    expect(listItems.length).toBe(11);

    listItems.forEach((li) => {
      expect(li.className).toContain("text-xs");
      expect(li.className).toContain("text-obd-text-muted");
    });
  });
});
