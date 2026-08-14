import type { AnalysisContext } from "../../domain/context.js";
import type { SpatialFrameHints } from "../../domain/result.js";
import type { SpatialPerformanceStats } from "../spatial-analysis/backpressure.js";

export type AnalyzerWorkerRole = "meter" | "attackInfo" | "result";

export type AnalyzerWorkerRequest =
  | {
      readonly type: "init";
      readonly role: AnalyzerWorkerRole;
      readonly ownSide: string;
      readonly analysisContext: AnalysisContext;
      /** HUD の画素読み取りを主スレッドの GPU が担うか。 */
      readonly hudFromGpu?: boolean;
    }
  | {
      readonly type: "meterFrame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly meterBuf: ArrayBuffer;
    }
  | {
      readonly type: "attackFrame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly meterBuf: ArrayBuffer;
    }
  | {
      readonly type: "resultFrame";
      readonly slot: number;
      readonly frameIndex: number;
      readonly hudBuf: ArrayBuffer;
      readonly superBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
      /** GPU 側で strip を切り出すための復号フレーム。 */
      readonly frame?: VideoFrame;
    }
  | {
      readonly type: "hudGpuBatch";
      readonly firstFrame: number;
      readonly scores: Uint32Array;
      readonly columns: Uint32Array;
      readonly drive: Uint32Array;
    }
  | { readonly type: "finishMeter" }
  | { readonly type: "finishAttack" }
  | {
      readonly type: "finish";
      readonly meterTimeline: string;
      readonly attackInfo: string;
    }
  | { readonly type: "spatialReset" }
  | {
      readonly type: "spatialFrame";
      readonly frameIndex: number;
      readonly rgbaBuf: ArrayBuffer;
      readonly hints: SpatialFrameHints;
    }
  | {
      readonly type: "spatialFinish";
      readonly spatialPerformance: SpatialPerformanceStats;
    };

export interface AnalyzerWorkerDone {
  readonly type: "done";
  readonly report: string;
  readonly timeline: string;
  readonly features: string;
  readonly trackedInputs?: string;
  readonly fightMarkers?: string;
  readonly attackInfo?: string;
  readonly regressionEvents: string;
  readonly debugHp?: unknown[];
  readonly spatialObservations?: string;
  readonly spatialPerformance?: SpatialPerformanceStats;
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
      readonly type: "attackFrameResult";
      readonly slot: number;
      readonly tCopy: number;
      readonly tAttack: number;
      readonly meterBuf: ArrayBuffer;
    }
  | {
      readonly type: "resultFrameResult";
      readonly slot: number;
      readonly tCopy: number;
      readonly tHud: number;
      readonly hudBuf: ArrayBuffer;
      readonly superBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }
  | { readonly type: "meterDone"; readonly timeline: string }
  | { readonly type: "attackDone"; readonly attackInfo: string }
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
