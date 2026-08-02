import type { AnalysisHistoryRecord } from "../domain/history.js";
import type { DebugFrameInspector } from "./debug-frame-inspection.js";
import type { DebugFrameSourceFactory } from "./debug-frame-source.js";

export interface AnalysisHistorySavingPreference {
  enabled: boolean;
  persistent: boolean;
}

export interface AnalysisHistoryRepository {
  save(record: AnalysisHistoryRecord): Promise<void>;
  load(): Promise<AnalysisHistoryRecord[]>;
  delete(id: string): Promise<void>;
  clear(): Promise<void>;
  getSavingPreference(): Promise<AnalysisHistorySavingPreference>;
  setSavingEnabled(enabled: boolean): Promise<void>;
}

export interface ResultsServices {
  debugFrameInspector: DebugFrameInspector;
  debugFrameSourceFactory: DebugFrameSourceFactory;
  history: AnalysisHistoryRepository;
}
