import {
  type AnalysisContextInput,
  resolveAnalysisContext,
} from "../../domain/context.js";
import type { AnalysisProgress, AnalysisResult } from "../../domain/result.js";
import {
  AnalysisRuntime,
  type AnalysisRuntimeCapabilities,
  type AnalysisRuntimeReadiness,
} from "../../domain/runtime.js";
import {
  matchesValidatedVideoFile,
  type ValidatedVideoInput,
  type VideoPreflightResult,
} from "../../domain/video-preflight.js";
import { preflightBrowserVideo } from "../video-decoding/browser-video-preflight.js";
import { LinkedAbortController } from "./linked-abort-controller.js";
import {
  AnalysisProgressWatchdog,
  browserAnalysisWatchdogHost,
} from "./progress-watchdog.js";
import { analyzeWithWebCodecs } from "./webcodecs-analysis-pipeline.js";

const ANALYSIS_STALL_TIMEOUT_MS = 30_000;
const ANALYSIS_STALL_MESSAGE =
  "動画解析の進捗が30秒以上停止したため中断しました。タブへ戻って再試行してください。";

export function analysisRuntimeReadiness(): AnalysisRuntimeReadiness {
  return AnalysisRuntime.evaluate(browserRuntimeCapabilities());
}

export async function preflightVideo(
  file: File,
  signal: AbortSignal,
): Promise<VideoPreflightResult> {
  const runtime = AnalysisRuntime.evaluate(browserRuntimeCapabilities());
  if (!runtime.available) {
    return {
      status: "invalid",
      code: "frame_extraction",
      message: runtime.reason,
    };
  }
  return preflightBrowserVideo(file, signal);
}

export async function analyzeVideo(
  file: File,
  validatedVideo: ValidatedVideoInput,
  ownSide: string,
  onProgress: AnalysisProgress,
  ownCharOrContext: string | AnalysisContextInput = "",
  signal: AbortSignal = new AbortController().signal,
): Promise<AnalysisResult> {
  const capabilities = browserRuntimeCapabilities();
  const runtime = AnalysisRuntime.evaluate(capabilities);
  if (!runtime.available) throw new Error(runtime.reason);
  if (!matchesValidatedVideoFile(file, validatedVideo)) {
    throw new Error(
      "選択中の動画と事前確認済みの動画が一致しません。動画を選択し直してください。",
    );
  }

  console.info("[analysis] runtime", {
    origin: capabilities.origin,
    secureContext: capabilities.secureContext,
    visibilityState: document.visibilityState,
  });

  const abortController = new LinkedAbortController(signal);
  const watchdog = new AnalysisProgressWatchdog(
    browserAnalysisWatchdogHost(),
    ANALYSIS_STALL_TIMEOUT_MS,
    () => {
      console.warn("[analysis] stalled", {
        visibilityState: document.visibilityState,
      });
      abortController.abort(new Error(ANALYSIS_STALL_MESSAGE));
    },
  );
  const reportProgress: AnalysisProgress = (progress, message) => {
    watchdog.pulse();
    onProgress(progress, message);
  };

  reportProgress(0, "解析中…");
  const analysisContext = resolveAnalysisContext(ownSide, ownCharOrContext);

  try {
    return await analyzeWithWebCodecs(
      file,
      validatedVideo,
      ownSide,
      reportProgress,
      analysisContext,
      abortController.signal,
    );
  } finally {
    watchdog.dispose();
    abortController.dispose();
  }
}

function browserRuntimeCapabilities(): AnalysisRuntimeCapabilities {
  return {
    // Test DOMs and older engines may omit the flag; VideoDecoder is checked
    // independently below.
    secureContext: globalThis.isSecureContext !== false,
    hasWorker: typeof Worker === "function",
    hasOffscreenCanvas2d: supportsOffscreenCanvas2d(),
    hasVideoFrameBitmap:
      typeof VideoFrame === "function" &&
      typeof createImageBitmap === "function",
    hasVideoDecoder:
      typeof VideoDecoder !== "undefined" &&
      typeof VideoDecoder.isConfigSupported === "function",
    origin: globalThis.location?.origin ?? "unknown",
  };
}

function supportsOffscreenCanvas2d(): boolean {
  if (typeof OffscreenCanvas !== "function") return false;
  try {
    return Boolean(new OffscreenCanvas(1, 1).getContext("2d"));
  } catch {
    return false;
  }
}
