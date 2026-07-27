import type { ResultsServices } from "./application/ports.js";
import { browserDebugFrameSourceFactory } from "./infrastructure/frame-access/browser-debug-frame-source.js";
import { wasmDebugFrameInspector } from "./infrastructure/frame-inspection/wasm-debug-frame-inspector.js";
import { browserAnalysisHistoryRepository } from "./infrastructure/history-persistence/indexeddb-analysis-history-repository.js";

export const browserResultsServices: ResultsServices = {
  debugFrameInspector: wasmDebugFrameInspector,
  debugFrameSourceFactory: browserDebugFrameSourceFactory,
  history: browserAnalysisHistoryRepository,
};
