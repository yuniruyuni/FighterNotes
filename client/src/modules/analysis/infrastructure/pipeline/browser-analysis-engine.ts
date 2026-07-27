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

export async function analyzeVideo(
  file: File,
  ownSide: string,
  onProgress: AnalysisProgress,
  ownCharOrContext: string | AnalysisContextInput = "",
): Promise<AnalysisResult> {
  const capabilities = browserRuntimeCapabilities();
  const runtime = AnalysisRuntime.evaluate(capabilities);
  if (!runtime.available) throw new Error(runtime.reason);

  console.info("[analysis] runtime", {
    origin: capabilities.origin,
    secureContext: capabilities.secureContext,
    visibilityState: document.visibilityState,
  });

  const abortController = new AbortController();
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
      ownSide,
      reportProgress,
      analysisContext,
      abortController.signal,
    );
  } finally {
    watchdog.dispose();
  }
}

function browserRuntimeCapabilities(): AnalysisRuntimeCapabilities {
  return {
    // Test DOMs and older engines may omit the flag; VideoDecoder is checked
    // independently below.
    secureContext: globalThis.isSecureContext !== false,
    hasVideoDecoder:
      typeof VideoDecoder !== "undefined" &&
      typeof VideoDecoder.isConfigSupported === "function",
    origin: globalThis.location?.origin ?? "unknown",
  };
}
