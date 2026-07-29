import { ANALYSIS_HEIGHT, ANALYSIS_WIDTH } from "../frame-extraction/layout.js";
import { SPATIAL_HEIGHT, SPATIAL_WIDTH } from "../spatial-analysis/layout.js";
import {
  AnalyzerWasmSession,
  MeterWasmSession,
  type WasmFirstPassPayload,
} from "../wasm-bridge/analyzer-session.js";
import type {
  AnalyzerWorkerRequest,
  AnalyzerWorkerResponse,
  AnalyzerWorkerRole,
} from "./protocol.js";

interface AnalyzerWorkerState {
  readonly resultWasm: AnalyzerWasmSession;
  readonly meterWasm: MeterWasmSession;
  role: AnalyzerWorkerRole | null;
  firstPassPayload: WasmFirstPassPayload | null;
}

export function installAnalyzerWorker(scope: DedicatedWorkerGlobalScope): void {
  const state: AnalyzerWorkerState = {
    resultWasm: new AnalyzerWasmSession(),
    meterWasm: new MeterWasmSession(),
    role: null,
    firstPassPayload: null,
  };
  scope.onmessage = (event: MessageEvent<AnalyzerWorkerRequest>) => {
    void handleMessage(scope, state, event.data);
  };
}

async function handleMessage(
  scope: DedicatedWorkerGlobalScope,
  state: AnalyzerWorkerState,
  message: AnalyzerWorkerRequest,
): Promise<void> {
  switch (message.type) {
    case "init": {
      if (state.role) throw new Error("Analyzer worker is already initialized");
      state.role = message.role;
      if (message.role === "meter") {
        await state.meterWasm.initialize(message.ownSide);
      } else {
        await state.resultWasm.initialize({
          ownSide: message.ownSide,
          analysisContext: message.analysisContext,
          spatialWidth: SPATIAL_WIDTH,
          spatialHeight: SPATIAL_HEIGHT,
        });
      }
      respond(scope, { type: "ready" });
      break;
    }
    case "meterFrame": {
      requireRole(state, "meter");
      const timing = state.meterWasm.analyzeFrame(
        message.frameIndex,
        message.meterBuf,
        { width: ANALYSIS_WIDTH, height: ANALYSIS_HEIGHT },
      );
      respond(
        scope,
        {
          type: "meterFrameResult",
          slot: message.slot,
          ...timing,
          meterBuf: message.meterBuf,
        },
        [message.meterBuf],
      );
      break;
    }
    case "resultFrame": {
      requireRole(state, "result");
      const timing = state.resultWasm.analyzeResultFrame(
        message.frameIndex,
        {
          hud: message.hudBuf,
          input: message.inputBuf,
        },
        { width: ANALYSIS_WIDTH, height: ANALYSIS_HEIGHT },
      );
      respond(
        scope,
        {
          type: "resultFrameResult",
          slot: message.slot,
          ...timing,
          hudBuf: message.hudBuf,
          inputBuf: message.inputBuf,
        },
        [message.hudBuf, message.inputBuf],
      );
      break;
    }
    case "finishMeter":
      requireRole(state, "meter");
      respond(scope, {
        type: "meterDone",
        timeline: state.meterWasm.finish(),
      });
      break;
    case "finish": {
      requireRole(state, "result");
      const result = state.resultWasm.finishFirstPass(message.meterTimeline);
      state.firstPassPayload = result.payload;
      respond(scope, {
        type: "firstPass",
        spatialWindows: result.spatialWindows,
      });
      break;
    }
    case "spatialReset":
      requireRole(state, "result");
      state.resultWasm.resetSpatialWindow();
      respond(scope, { type: "spatialResetReady" });
      break;
    case "spatialFrame":
      requireRole(state, "result");
      state.resultWasm.analyzeSpatialFrame(
        message.frameIndex,
        message.rgbaBuf,
        message.hints,
      );
      respond(scope, { type: "spatialFrameResult" });
      break;
    case "spatialFinish": {
      if (!state.firstPassPayload) {
        throw new Error("spatial finish before first pass");
      }
      requireRole(state, "result");
      const spatial = state.resultWasm.finishSpatialPass();
      respond(scope, {
        type: "done",
        ...state.firstPassPayload,
        ...spatial,
      });
      break;
    }
  }
}

function requireRole(
  state: AnalyzerWorkerState,
  expected: AnalyzerWorkerRole,
): void {
  if (state.role !== expected) {
    throw new Error(
      `Expected ${expected} worker, received ${state.role ?? "uninitialized"} worker`,
    );
  }
}

function respond(
  scope: DedicatedWorkerGlobalScope,
  message: AnalyzerWorkerResponse,
  transfer: Transferable[] = [],
): void {
  scope.postMessage(message, { transfer });
}
