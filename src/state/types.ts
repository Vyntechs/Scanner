export type AppPhase =
  | "disconnected"
  | "connecting"
  | "identifying"
  | "discovering"
  | "scanningDtc"
  | "ready"
  | "error";

export type TransportMode = "simulation" | "j2534";

export type ModuleStatus = "ok" | "noResponse" | "error";

export interface ModuleInfo {
  id: string;
  name: string;
  bus: string;
  category: string;
  txId: number;
  rxId: number;
  status: ModuleStatus;
  dtcCount: number;
}

export interface DtcInfo {
  code: string;
  description: string;
  status: string;
}

export interface BusInfo {
  name: string;
  modules: string[];
}

export interface TopologyGraph {
  buses: BusInfo[];
}

export interface ProgressInfo {
  stage: string;
  percent: number;
  message: string;
}

export interface ErrorInfo {
  summary: string;
  details: string;
}

export interface SessionSummary {
  sessionId: string;
  timestamp: string;
  vin?: string | null;
  moduleCount: number;
  dtcCount: number;
}

export interface AppSnapshot {
  phase: AppPhase;
  transport: TransportMode;
  adapterConnected: boolean;
  vin?: string | null;
  modules: ModuleInfo[];
  dtcs: Record<string, DtcInfo[]>;
  topology: TopologyGraph;
  progress?: ProgressInfo | null;
  lastError?: ErrorInfo | null;
  sessionId?: string | null;
  logsPath?: string | null;
  lastSession?: SessionSummary | null;
}

export interface AdapterStatus {
  available: boolean;
  message: string;
  dllPath?: string | null;
}
