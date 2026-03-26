import { invoke } from "@tauri-apps/api/core";

/**
 * Generate a timestamped filename for CSV exports.
 */
export function makeCSVFilename(prefix: string): string {
  const now = new Date();
  return `${prefix}_${now.getFullYear()}${String(now.getMonth() + 1).padStart(2, "0")}${String(now.getDate()).padStart(2, "0")}_${String(now.getHours()).padStart(2, "0")}${String(now.getMinutes()).padStart(2, "0")}${String(now.getSeconds()).padStart(2, "0")}.csv`;
}

/**
 * Save CSV content via Tauri backend to Desktop/BricarOBD_Exports/.
 */
export async function saveCSVFile(content: string, filename: string): Promise<string> {
  return invoke<string>("save_csv_file", { filename, content });
}
