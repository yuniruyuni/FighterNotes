import type { AnalysisDebugSink } from "../../application/ports.js";

export const browserAnalysisDebugSink: AnalysisDebugSink = {
  capture(result) {
    Object.assign(window, {
      __hpFeatures: result.hpFeatures,
      __spatialObservations: result.spatialObservations,
      // Local E2E reads these references after analysis to hash the exact
      // demux/decode timestamp mapping without copying encoded video bytes.
      __fighterNotesDecodeMapping: {
        frameTimestamps: result.frameTimestamps,
        frameToSampleIdx: result.frameToSampleIdx,
        sampleData: result.sampleData,
      },
    });
  },
};
