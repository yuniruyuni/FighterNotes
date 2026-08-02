import type { AnalysisContext } from "../../domain/context.js";
import type {
  AnalysisProgress,
  AnalysisResult,
  FrameSample,
  SpatialCandidateWindow,
  SpatialFrameHints,
  VideoCodecConfig,
} from "../../domain/result.js";
import {
  EMPTY_SPATIAL_DECODE_STATS,
  type SpatialDecodeStats,
} from "../spatial-analysis/backpressure.js";
import { decodeSpatialWindows } from "../spatial-analysis/spatial-pass.js";
import type { AnalyzerWorkerDone } from "../worker-bridge/protocol.js";
import { throwIfAborted, waitForAbort } from "./abort.js";
import { buildAnalysisResult } from "./analysis-result-builder.js";

export interface AnalysisCompletionSession {
  firstPass(): Promise<SpatialCandidateWindow[]>;
  resetSpatialWindow(): Promise<void>;
  sendSpatialFrame(
    frameIndex: number,
    rgbaBuf: ArrayBuffer,
    hints: SpatialFrameHints,
    signal?: AbortSignal,
  ): Promise<void>;
  drainSpatialFrames(): Promise<void>;
  finishSpatialPass(decodeStats: SpatialDecodeStats): void;
  result(): Promise<AnalyzerWorkerDone>;
}

interface CompleteAnalysisOptions {
  readonly session: AnalysisCompletionSession;
  readonly analysisContext: AnalysisContext;
  readonly videoArrayBuffer: ArrayBuffer;
  readonly sampleData: FrameSample[];
  readonly frameToSampleIdx: number[];
  readonly frameTimestamps: number[];
  readonly getCodecConfig: () => VideoCodecConfig | null;
  readonly onProgress: AnalysisProgress;
  readonly signal: AbortSignal;
}

export async function completeAnalysis(
  options: CompleteAnalysisOptions,
): Promise<AnalysisResult> {
  const {
    session,
    analysisContext,
    videoArrayBuffer,
    sampleData,
    frameToSampleIdx,
    frameTimestamps,
    getCodecConfig,
    onProgress,
    signal,
  } = options;

  throwIfAborted(signal);
  const windows = await waitForAbort(session.firstPass(), signal);
  throwIfAborted(signal);
  const codecConfig = getCodecConfig();
  let spatialDecodeStats = EMPTY_SPATIAL_DECODE_STATS;
  if (windows.length > 0) {
    if (!codecConfig) {
      throw new Error("空間候補窓を再デコードするcodec設定がありません");
    }
    onProgress(0.9, "位置関係を確認中…");
    spatialDecodeStats = await decodeSpatialWindows({
      windows,
      sampleData,
      frameToSampleIdx,
      videoArrayBuffer,
      codecConfig,
      resetWindow: () => waitForAbort(session.resetSpatialWindow(), signal),
      sendFrame: (frameIndex, rgbaBuf, hints, processingSignal) =>
        session.sendSpatialFrame(frameIndex, rgbaBuf, hints, processingSignal),
      drain: () => waitForAbort(session.drainSpatialFrames(), signal),
      onProgress,
      signal,
    });
  }

  throwIfAborted(signal);
  session.finishSpatialPass(spatialDecodeStats);
  const message = await waitForAbort(session.result(), signal);
  throwIfAborted(signal);
  if (message.debugHp && message.debugHp.length > 0) {
    console.log(
      "[DEBUG HP] 500f毎サンプル（match/lscore/rscoreでis_match_screen判定を確認）",
    );
    console.table(message.debugHp);
  }
  onProgress(1, "レポート生成中…");
  return buildAnalysisResult(message, {
    analysisContext,
    frameTimestamps,
    sampleData,
    videoArrayBuffer,
    codecConfig,
    frameToSampleIdx,
  });
}
