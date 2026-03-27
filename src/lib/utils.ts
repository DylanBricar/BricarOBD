import { clsx, type ClassValue } from "clsx";
import { twMerge } from "tailwind-merge";

export function cn(...inputs: ClassValue[]) {
  return twMerge(clsx(inputs));
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}

export function escapeCSV(value: string | number): string {
  const str = String(value);
  const escaped = str.replace(/"/g, '""');
  if (/[,"\n\r]/.test(str) || /^[=+\-@\t\r]/.test(str)) {
    return `"${escaped}"`;
  }
  return escaped;
}
