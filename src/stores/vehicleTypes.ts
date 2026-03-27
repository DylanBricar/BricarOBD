export interface PidValue {
  pid: number;
  name: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  history: number[];
  timestamp: number;
}

export interface DtcCode {
  code: string;
  description: string;
  status: "active" | "pending" | "permanent";
  source: string;
  repairTips?: string;
  causes?: string[];
  quickCheck?: string;
  difficulty?: number;
  ecuContext?: string;
}

export interface DtcHistoryEntry extends DtcCode {
  seenAt: number;
}

export interface EcuInfo {
  name: string;
  address: string;
  protocol: string;
  dids: Record<string, string>;
}

export interface MonitorStatus {
  nameKey: string;
  available: boolean;
  complete: boolean;
  descriptionKey?: string;
  specificationKey?: string;
}

export interface Mode06Result {
  tid: number;
  mid: number;
  name: string;
  unit: string;
  testValue: number;
  minLimit: number;
  maxLimit: number;
  passed: boolean;
}

export interface FreezeFrameData {
  dtcCode: string;
  frameNumber: number;
  pids: PidValue[];
}

export interface VehicleOperation {
  [key: string]: any;
}

export interface WriteOperation {
  [key: string]: any;
}
