import type { LucideIcon } from "lucide-react";

export interface AdvancedOperation {
  id: string;
  name: string;
  description: string;
  risk_level: "low" | "medium" | "high" | "critical";
  needs_value?: boolean;
  unit?: string;
}

export interface OperationCategory {
  id: string;
  name: string;
  Icon: LucideIcon;
  operations: AdvancedOperation[];
}
