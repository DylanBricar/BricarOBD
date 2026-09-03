import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import CatalogGate from "./CatalogGate";
import { formatCatalogSize } from "@/lib/catalog";

const { invokeMock } = vi.hoisted(() => ({ invokeMock: vi.fn() }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: invokeMock,
  Channel: class MockChannel {
    onmessage = vi.fn();
  },
}));

describe("CatalogGate", () => {
  beforeEach(() => {
    invokeMock.mockReset();
  });

  it("renders the application when the catalog is ready", async () => {
    invokeMock.mockResolvedValue({
      ready: true,
      downloading: false,
      expectedBytes: 527_691_776,
    });

    render(
      <CatalogGate>
        <div>diagnostic-ready</div>
      </CatalogGate>,
    );

    expect(await screen.findByText("diagnostic-ready")).toBeInTheDocument();
    expect(invokeMock).toHaveBeenCalledWith("get_catalog_status");
  });

  it("downloads the verified catalog before rendering the application", async () => {
    invokeMock
      .mockResolvedValueOnce({
        ready: false,
        downloading: false,
        expectedBytes: 527_691_776,
      })
      .mockResolvedValueOnce({
        ready: true,
        downloading: false,
        expectedBytes: 527_691_776,
      });

    render(
      <CatalogGate>
        <div>diagnostic-ready</div>
      </CatalogGate>,
    );
    fireEvent.click(await screen.findByRole("button", { name: "catalog.download" }));

    await waitFor(() => expect(screen.getByText("diagnostic-ready")).toBeInTheDocument());
    expect(invokeMock).toHaveBeenLastCalledWith(
      "download_catalog",
      expect.objectContaining({ onEvent: expect.anything() }),
    );
  });

  it("formats the published catalog size in mebibytes", () => {
    expect(formatCatalogSize(527_691_776)).toBe("503 Mo");
  });
});
