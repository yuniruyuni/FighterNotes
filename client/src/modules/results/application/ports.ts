import type { AnalysisHistoryRecord } from "../domain/history.js";
import type { DebugFrameInspector } from "./debug-frame-inspection.js";
import type { DebugFrameSourceFactory } from "./debug-frame-source.js";

export interface AnalysisHistoryRepository {
  save(record: AnalysisHistoryRecord): Promise<void>;
  load(): Promise<AnalysisHistoryRecord[]>;
}

export interface ResultsServices {
  debugFrameInspector: DebugFrameInspector;
  debugFrameSourceFactory: DebugFrameSourceFactory;
  history: AnalysisHistoryRepository;
}
