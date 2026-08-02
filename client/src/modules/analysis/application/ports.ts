import type { AnalysisContext, AnalysisSide } from "../domain/context.js";
import type { AnalysisProgress, AnalysisResult } from "../domain/result.js";
import type { AnalysisRuntimeReadiness } from "../domain/runtime.js";

export interface AnalysisEngine {
  readiness(): AnalysisRuntimeReadiness;
  analyze(
    file: File,
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
