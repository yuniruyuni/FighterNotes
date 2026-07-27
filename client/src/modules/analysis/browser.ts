import type { AnalysisServices } from "./application/ports.js";
import { browserAnalysisDebugSink } from "./infrastructure/diagnostics/browser-analysis-debug-sink.js";
import {
  analysisRuntimeReadiness,
  analyzeVideo,
} from "./infrastructure/pipeline/browser-analysis-engine.js";

export const browserAnalysisServices: AnalysisServices = {
  engine: {
    readiness: analysisRuntimeReadiness,
    analyze: analyzeVideo,
  },
  debugSink: browserAnalysisDebugSink,
};

export { decodeFrameAt } from "./infrastructure/video-decoding/webcodecs-frame-decoder.js";
export { ensureAnalysisWasm } from "./infrastructure/wasm-bridge/main-thread-runtime.js";
