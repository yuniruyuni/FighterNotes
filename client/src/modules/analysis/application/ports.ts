import type { AnalysisContext, AnalysisSide } from "../domain/context.js";
import type { AnalysisProgress, AnalysisResult } from "../domain/result.js";
import type { AnalysisRuntimeReadiness } from "../domain/runtime.js";
import type {
  ValidatedVideoInput,
  VideoPreflightResult,
} from "../domain/video-preflight.js";

export interface AnalysisEngine {
  readiness(): AnalysisRuntimeReadiness;
  preflight(file: File, signal: AbortSignal): Promise<VideoPreflightResult>;
  analyze(
    file: File,
    validatedVideo: ValidatedVideoInput,
    side: AnalysisSide,
    onProgress: AnalysisProgress,
    context: AnalysisContext,
    signal: AbortSignal,
  ): Promise<AnalysisResult>;
}

export interface AnalysisDebugSink {
  capture(result: AnalysisResult): void;
}

export interface AnalysisServices {
  engine: AnalysisEngine;
  debugSink: AnalysisDebugSink;
}
