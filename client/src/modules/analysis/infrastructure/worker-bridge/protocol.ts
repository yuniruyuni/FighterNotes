import type { AnalysisContext } from "../../domain/context.js";
import type { SpatialFrameHints } from "../../domain/result.js";

export type AnalyzerWorkerRole = "meter" | "result";

export type AnalyzerWorkerRequest =
  | {
      readonly type: "init";
      readonly role: AnalyzerWorkerRole;
      readonly ownSide: string;
      readonly analysisContext: AnalysisContext;
    }
  | {
      readonly type: "meterFrame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly meterBuf: ArrayBuffer;
    }
  | {
      readonly type: "resultFrame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly hudBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }
  | { readonly type: "finishMeter" }
  | { readonly type: "finish"; readonly meterTimeline: string }
  | { readonly type: "spatialReset" }
  | {
      readonly type: "spatialFrame";
      readonly frameIndex: number;
      readonly rgbaBuf: ArrayBuffer;
      readonly hints: SpatialFrameHints;
    }
  | { readonly type: "spatialFinish" };

export interface AnalyzerWorkerDone {
  readonly type: "done";
  readonly report: string;
  readonly timeline: string;
  readonly features: string;
  readonly trackedInputs?: string;
  readonly fightMarkers?: string;
  readonly attackInfo?: string;
  readonly debugHp?: unknown[];
  readonly spatialObservations?: string;
}

export type AnalyzerWorkerResponse =
  | { readonly type: "ready" }
  | { readonly type: "error"; readonly message: string }
  | {
      readonly type: "meterFrameResult";
      readonly slot: number;
      readonly tCopy: number;
      readonly tMeter: number;
      readonly meterBuf: ArrayBuffer;
    }
  | {
      readonly type: "resultFrameResult";
      readonly slot: number;
      readonly tCopy: number;
      readonly tHud: number;
      readonly hudBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }
  | { readonly type: "meterDone"; readonly timeline: string }
  | { readonly type: "spatialResetReady" }
  | { readonly type: "spatialFrameResult" }
  | {
      readonly type: "firstPass";
      readonly spatialWindows: string;
    }
  | AnalyzerWorkerDone;

export function postAnalyzerWorkerMessage(
  target: Pick<Worker, "postMessage">,
  message: AnalyzerWorkerRequest,
  transfer: Transferable[] = [],
): void {
  target.postMessage(message, transfer);
}
