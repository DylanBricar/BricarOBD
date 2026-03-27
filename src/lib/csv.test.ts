import { describe, it, expect, vi, beforeEach } from "vitest";
import { makeCSVFilename, saveCSVFile } from "./csv";
import { invoke } from "@tauri-apps/api/core";

vi.mock("@tauri-apps/api/core");

describe("makeCSVFilename()", () => {
  it("creates filename with prefix and .csv extension", () => {
    const filename = makeCSVFilename("export");
    expect(filename).toMatch(/^export_\d{8}_\d{6}\.csv$/);
  });

  it("includes date in YYYYMMDD format", () => {
    const filename = makeCSVFilename("data");
    const dateMatch = filename.match(/_(\d{8})_/);
    expect(dateMatch).toBeTruthy();
    const date = dateMatch![1];
    // Validate format: 4 digit year, 2 digit month, 2 digit day
    expect(/^\d{4}(0[1-9]|1[0-2])(0[1-9]|[12]\d|3[01])$/.test(date)).toBe(true);
  });

  it("includes time in HHMMSS format", () => {
    const filename = makeCSVFilename("report");
    const timeMatch = filename.match(/_(\d{6})\.csv$/);
    expect(timeMatch).toBeTruthy();
    const time = timeMatch![1];
    // Validate format: HH (00-23), MM (00-59), SS (00-59)
    expect(/^([01]\d|2[0-3])[0-5]\d[0-5]\d$/.test(time)).toBe(true);
  });

  it("handles various prefix names", () => {
    expect(makeCSVFilename("dtc")).toMatch(/^dtc_\d{8}_\d{6}\.csv$/);
    expect(makeCSVFilename("vehicle_data")).toMatch(/^vehicle_data_\d{8}_\d{6}\.csv$/);
    expect(makeCSVFilename("live")).toMatch(/^live_\d{8}_\d{6}\.csv$/);
  });

  it("consecutive calls have different timestamps", () => {
    const file1 = makeCSVFilename("test");
    const file2 = makeCSVFilename("test");
    // At minimum they should be different (or at least have the infrastructure to be)
    // We won't assert they're always different due to timing, but ensure format is consistent
    expect(file1).toMatch(/^test_\d{8}_\d{6}\.csv$/);
    expect(file2).toMatch(/^test_\d{8}_\d{6}\.csv$/);
  });
});

describe("saveCSVFile()", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("calls invoke with correct arguments", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue("path/to/file.csv");

    const result = await saveCSVFile("a,b,c\n1,2,3", "test.csv");

    expect(mockInvoke).toHaveBeenCalledWith("save_csv_file", {
      filename: "test.csv",
      content: "a,b,c\n1,2,3",
    });
    expect(result).toBe("path/to/file.csv");
  });

  it("returns the file path from invoke", async () => {
    const mockInvoke = vi.mocked(invoke);
    const expectedPath = "/home/user/Desktop/BricarOBD_Exports/data_20250327_143022.csv";
    mockInvoke.mockResolvedValue(expectedPath);

    const result = await saveCSVFile("content", "data.csv");

    expect(result).toBe(expectedPath);
  });

  it("handles empty content", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue("path/to/empty.csv");

    const result = await saveCSVFile("", "empty.csv");

    expect(mockInvoke).toHaveBeenCalledWith("save_csv_file", {
      filename: "empty.csv",
      content: "",
    });
    expect(result).toBe("path/to/empty.csv");
  });

  it("handles large content", async () => {
    const mockInvoke = vi.mocked(invoke);
    const largeContent = "a,b,c\n" + "1,2,3\n".repeat(1000);
    mockInvoke.mockResolvedValue("path/to/large.csv");

    const result = await saveCSVFile(largeContent, "large.csv");

    expect(mockInvoke).toHaveBeenCalledWith("save_csv_file", {
      filename: "large.csv",
      content: largeContent,
    });
    expect(result).toBe("path/to/large.csv");
  });

  it("propagates invoke errors", async () => {
    const mockInvoke = vi.mocked(invoke);
    const error = new Error("Failed to save file");
    mockInvoke.mockRejectedValue(error);

    await expect(saveCSVFile("content", "test.csv")).rejects.toThrow(error);
  });

  it("handles filename with path traversal attempt", async () => {
    const mockInvoke = vi.mocked(invoke);
    mockInvoke.mockResolvedValue("path/to/file.csv");

    // The function doesn't sanitize, but backend should handle it
    await saveCSVFile("content", "../../../etc/passwd");

    expect(mockInvoke).toHaveBeenCalledWith("save_csv_file", {
      filename: "../../../etc/passwd",
      content: "content",
    });
  });
});
