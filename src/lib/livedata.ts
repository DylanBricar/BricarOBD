import type { PidValue } from "@/stores/vehicle";
import { escapeCSV } from "@/lib/utils";

export function getTheoreticalMax(unit: string): number {
  const map: Record<string, number> = {
    "RPM": 8000,
    "km/h": 250,
    "°C": 120,
    "%": 100,
    "bar": 5,
    "V": 14,
    "A": 100,
    "kPa": 200,
    "ms": 50,
    "g": 5,
  };
  return map[unit] || 100;
}

export function generateCSV(
  pidData: Map<number, PidValue>,
  header: string,
  buffer?: Array<{ timestamp: Date; snapshot: Record<number, number> }>
): string {
  const rows: string[] = [header];

  if (buffer && buffer.length > 0) {
    // Export recording buffer
    buffer.forEach((record) => {
      pidData.forEach((pid) => {
        const value = record.snapshot[pid.pid];
        if (value !== undefined) {
          rows.push(
            `${record.timestamp.toISOString()},0x${pid.pid
              .toString(16)
              .toUpperCase()
              .padStart(2, "0")},${escapeCSV(pid.name)},${value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`
          );
        }
      });
    });
  } else {
    // Export current snapshot
    const now = new Date().toISOString();
    pidData.forEach((pid) => {
      rows.push(
        `${now},0x${pid.pid
          .toString(16)
          .toUpperCase()
          .padStart(2, "0")},${escapeCSV(pid.name)},${pid.value.toFixed(2)},${pid.unit},${pid.min.toFixed(2)},${pid.max.toFixed(2)}`
      );
    });
  }

  return rows.join("\n");
}
