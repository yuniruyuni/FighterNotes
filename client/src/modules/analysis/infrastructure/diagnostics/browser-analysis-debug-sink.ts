import type { AnalysisDebugSink } from "../../application/ports.js";

export const browserAnalysisDebugSink: AnalysisDebugSink = {
  capture(result) {
    Object.assign(window, {
      __hpFeatures: result.hpFeatures,
      __spatialObservations: result.spatialObservations,
    });
  },
};
