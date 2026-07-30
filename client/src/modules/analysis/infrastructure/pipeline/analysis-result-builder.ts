import type { AnalysisContext } from "../../domain/context.js";
import type {
  AnalysisResult,
  FrameSample,
  HpFrameData,
  VideoCodecConfig,
} from "../../domain/result.js";
import type { AnalyzerWorkerDone } from "../worker-bridge/protocol.js";

interface AnalysisResultArtifacts {
  readonly analysisContext: AnalysisContext;
  readonly frameTimestamps: number[];
  readonly sampleData: FrameSample[];
  readonly videoArrayBuffer: ArrayBuffer;
  readonly codecConfig: VideoCodecConfig | null;
  readonly frameToSampleIdx: number[];
}

export function buildAnalysisResult(
  message: AnalyzerWorkerDone,
  artifacts: AnalysisResultArtifacts,
): AnalysisResult {
  return {
    ...artifacts,
    report: JSON.parse(message.report),
    timeline: JSON.parse(message.timeline),
    trackedInputs: message.trackedInputs
      ? JSON.parse(message.trackedInputs)
      : null,
    attackInfo: message.attackInfo ? JSON.parse(message.attackInfo) : [],
    hpFeatures: message.features
      ? (JSON.parse(message.features) as HpFrameData[])
      : [],
    frameCount: artifacts.frameTimestamps.length,
    spatialObservations: message.spatialObservations
      ? JSON.parse(message.spatialObservations)
      : [],
  };
}
