import type { AnalysisContext } from "../../domain/context.js";
import type { SpatialFrameHints } from "../../domain/result.js";

export type AnalyzerWorkerRequest =
  | {
      readonly type: "init";
      readonly ownSide: string;
      readonly analysisContext: AnalysisContext;
    }
  | {
      readonly type: "frame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly hudBuf: ArrayBuffer;
      readonly meterBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }
  | { readonly type: "finish" }
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
  readonly debugHp?: unknown[];
  readonly spatialObservations?: string;
}

export type AnalyzerWorkerResponse =
  | { readonly type: "ready" }
  | {
      readonly type: "frameResult";
      readonly slot: number;
      readonly tCopy: number;
      readonly tMeter: number;
      readonly tHud: number;
      readonly hudBuf: ArrayBuffer;
      readonly meterBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }
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
