import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, it, expect, vi, beforeEach } from "vitest";
import DtcDetailPanel from "./DtcDetailPanel";

interface DtcCode {
  code: string;
  status: "active" | "pending" | "permanent";
  description: string;
  source: string;
  causes?: string[];
  repairTips?: string;
  difficulty?: number;
  quickCheck?: string;
  ecuContext?: string;
}

describe("DtcDetailPanel", () => {
  const mockOnOpenExternal = vi.fn();
  const mockOnBuildSearchQuery = vi.fn((code: string, platform: "google" | "youtube") =>
    `https://search.example.com?q=${code}&platform=${platform}`
  );
  const mockT = (key: string) => key;

  beforeEach(() => {
    mockOnOpenExternal.mockClear();
    mockOnBuildSearchQuery.mockClear();
  });

  it("returns null when selectedData is null", () => {
    const { container } = render(
      <DtcDetailPanel
        selectedData={null}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(container.firstChild).toBeNull();
  });

  it("renders code in font-mono", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Mass Airflow Sensor Range/Performance",
      source: "Engine Control Module",
    };
    const { container } = render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    const monoCode = container.querySelector(".font-mono");
    expect(monoCode?.textContent).toBe("P0101");
  });

  it("renders status badge with i18n key", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("dtc.active")).toBeInTheDocument();
  });

  it("renders description", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Mass Airflow Sensor Range/Performance",
      source: "Engine Control Module",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("Mass Airflow Sensor Range/Performance")).toBeInTheDocument();
  });

  it("renders source", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Engine Control Module",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("Engine Control Module")).toBeInTheDocument();
  });

  it("renders causes list with bullet points", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
      causes: ["Dirty MAF sensor", "Vacuum leak", "Faulty sensor"],
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("Dirty MAF sensor")).toBeInTheDocument();
    expect(screen.getByText("Vacuum leak")).toBeInTheDocument();
    expect(screen.getByText("Faulty sensor")).toBeInTheDocument();
  });

  it("renders repair tips", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
      repairTips: "Clean the MAF sensor with appropriate cleaner",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("Clean the MAF sensor with appropriate cleaner")).toBeInTheDocument();
  });

  it("shows 'no tips' message when no tips data", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("dtc.noTips")).toBeInTheDocument();
  });

  it("calls onOpenExternal with Google search URL when web search button clicked", async () => {
    const user = userEvent.setup();
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
    };
    mockOnBuildSearchQuery.mockReturnValue("https://google.com/search?q=P0101");
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    const webSearchButton = screen.getByText("dtc.webSearch");
    await user.click(webSearchButton);
    expect(mockOnBuildSearchQuery).toHaveBeenCalledWith("P0101", "google");
    expect(mockOnOpenExternal).toHaveBeenCalledWith("https://google.com/search?q=P0101");
  });

  it("calls onOpenExternal with YouTube search URL when YouTube button clicked", async () => {
    const user = userEvent.setup();
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
    };
    mockOnBuildSearchQuery.mockReturnValue("https://youtube.com/results?search_query=P0101");
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    const youtubeButton = screen.getByText("dtc.youtubeSearch");
    await user.click(youtubeButton);
    expect(mockOnBuildSearchQuery).toHaveBeenCalledWith("P0101", "youtube");
    expect(mockOnOpenExternal).toHaveBeenCalledWith("https://youtube.com/results?search_query=P0101");
  });

  it("renders difficulty badge when difficulty is present", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
      difficulty: 3,
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("dtc.difficulty3")).toBeInTheDocument();
  });

  it("renders ECU context when present", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
      ecuContext: "ECM / PCM",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("ECM / PCM")).toBeInTheDocument();
  });

  it("renders quick check when present", () => {
    const dtc: DtcCode = {
      code: "P0101",
      status: "active",
      description: "Test Description",
      source: "Test Source",
      quickCheck: "Check for air intake leaks",
    };
    render(
      <DtcDetailPanel
        selectedData={dtc}
        t={mockT}
        onOpenExternal={mockOnOpenExternal}
        onBuildSearchQuery={mockOnBuildSearchQuery}
      />
    );
    expect(screen.getByText("Check for air intake leaks")).toBeInTheDocument();
  });
});
