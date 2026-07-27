import type { CompletedAnalysis } from "../domain/analysis-session.js";
import type { AnalysisSide } from "../domain/context.js";
import { createAnalysisContext } from "../domain/context.js";
import type { AnalysisProgress } from "../domain/result.js";
import type { AnalysisServices } from "./ports.js";

export interface AnalysisRequest {
  file: File;
  side: AnalysisSide;
  ownCharacter: string;
  opponentCharacter: string;
}

export async function runAnalysis(
  request: AnalysisRequest,
  onProgress: AnalysisProgress,
  services: AnalysisServices,
): Promise<CompletedAnalysis> {
  const context = createAnalysisContext(
    request.side,
    request.ownCharacter,
    request.opponentCharacter,
  );
  const rawResult = await services.engine.analyze(
    request.file,
    request.side,
    onProgress,
    context,
  );
  const result = { ...rawResult, videoArrayBuffer: null };
  services.debugSink.capture(result);
  return {
    file: request.file,
    result,
    report: result.report,
    context,
  };
}
