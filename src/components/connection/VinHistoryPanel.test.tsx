import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import { Mock } from "vitest";
import VinHistoryPanel from "./VinHistoryPanel";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(undefined),
}));

import { invoke } from "@tauri-apps/api/core";

describe("VinHistoryPanel", () => {
  const mockOnSelectVin = vi.fn();
  const mockOnRemoveFromHistory = vi.fn();
  const mockT = (key: string) => key;

  const vinHistory = [
    {
      vin: "1HGCM82633A123456",
      make: "Honda",
      model: "Accord",
      year: 2015,
      lastSeen: 1704067200000,
    },
    {
      vin: "2T1BURHE0JC048026",
      make: "Toyota",
      model: "Corolla",
      year: 2018,
      lastSeen: 1704153600000,
    },
  ];

  beforeEach(() => {
    mockOnSelectVin.mockClear();
    mockOnRemoveFromHistory.mockClear();
    (invoke as Mock).mockResolvedValue(undefined);
  });

  it("renders title with i18n key", () => {
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    expect(screen.getByText("connection.vinHistory")).toBeInTheDocument();
  });

  it("renders glass-card styling", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    expect(container.querySelector(".glass-card")).toBeInTheDocument();
  });

  it("renders Clock icon", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const icon = container.querySelector("svg");
    expect(icon).toBeInTheDocument();
  });

  it("displays all vehicles in history", () => {
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    expect(screen.getByText(/Honda Accord/)).toBeInTheDocument();
    expect(screen.getByText(/2015/)).toBeInTheDocument();
    expect(screen.getByText(/Toyota Corolla/)).toBeInTheDocument();
    expect(screen.getByText(/2018/)).toBeInTheDocument();
  });

  it("shows VIN in font-mono", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const monoElements = container.querySelectorAll(".font-mono");
    const vins = Array.from(monoElements).map((el) => el.textContent);
    expect(vins).toContain("1HGCM82633A123456");
    expect(vins).toContain("2T1BURHE0JC048026");
  });

  it("calls onSelectVin when vehicle entry is clicked", async () => {
    const user = userEvent.setup();
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const hondaText = screen.getByText(/Honda Accord/);
    await user.click(hondaText);
    expect(mockOnSelectVin).toHaveBeenCalledWith("1HGCM82633A123456");
  });

  it("has group class on entries for hover effects", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const groupElements = container.querySelectorAll(".group");
    expect(groupElements.length).toBeGreaterThan(0);
  });

  it("shows delete button with Trash2 icon for each entry", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const deleteButtons = container.querySelectorAll("button");
    expect(deleteButtons.length).toBeGreaterThanOrEqual(vinHistory.length);
  });

  it("calls invoke with clear_vin_cache when cache button clicked", async () => {
    const user = userEvent.setup();
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const cacheButtons = screen.getAllByTitle("connection.clearCache");
    await user.click(cacheButtons[0]);
    expect(invoke).toHaveBeenCalledWith("clear_vin_cache", { vin: "1HGCM82633A123456" });
  });

  it("calls onRemoveFromHistory when remove button clicked", async () => {
    const user = userEvent.setup();
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const removeButtons = screen.getAllByTitle("connection.removeFromHistory");
    await user.click(removeButtons[0]);
    expect(mockOnRemoveFromHistory).toHaveBeenCalledWith("1HGCM82633A123456");
  });

  it("displays date for each entry", () => {
    render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const dateElements = screen.getAllByText(/\d{1,2}\/\d{1,2}\/\d{4}/);
    expect(dateElements.length).toBeGreaterThan(0);
  });

  it("renders with space-y-3 spacing class", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    const card = container.querySelector(".glass-card.space-y-3");
    expect(card).toBeInTheDocument();
  });

  it("handles empty vin history", () => {
    const { container } = render(
      <VinHistoryPanel
        vinHistory={[]}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    expect(container.querySelector(".glass-card")).toBeInTheDocument();
  });

  it("uses correct language for date formatting", () => {
    const { rerender } = render(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="fr"
      />
    );
    expect(screen.getAllByText(/\d{1,2}\/\d{1,2}\/\d{4}/).length).toBeGreaterThan(0);

    rerender(
      <VinHistoryPanel
        vinHistory={vinHistory}
        onSelectVin={mockOnSelectVin}
        onRemoveFromHistory={mockOnRemoveFromHistory}
        isClearing={false}
        t={mockT}
        language="en"
      />
    );
    expect(screen.getAllByText(/\d{1,2}\/\d{1,2}\/\d{4}/).length).toBeGreaterThan(0);
  });
});
