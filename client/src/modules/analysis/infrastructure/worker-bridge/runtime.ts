import { ANALYSIS_HEIGHT, ANALYSIS_WIDTH } from "../frame-extraction/layout.js";
import { SPATIAL_HEIGHT, SPATIAL_WIDTH } from "../spatial-analysis/layout.js";
import {
  AnalyzerWasmSession,
  type WasmFirstPassPayload,
} from "../wasm-bridge/analyzer-session.js";
import type {
  AnalyzerWorkerRequest,
  AnalyzerWorkerResponse,
} from "./protocol.js";

interface AnalyzerWorkerState {
  readonly wasm: AnalyzerWasmSession;
  firstPassPayload: WasmFirstPassPayload | null;
}

export function installAnalyzerWorker(scope: DedicatedWorkerGlobalScope): void {
  const state: AnalyzerWorkerState = {
    wasm: new AnalyzerWasmSession(),
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
    case "init":
      await state.wasm.initialize({
        ownSide: message.ownSide,
        analysisContext: message.analysisContext,
        spatialWidth: SPATIAL_WIDTH,
        spatialHeight: SPATIAL_HEIGHT,
      });
      respond(scope, { type: "ready" });
      break;
    case "frame": {
      const timing = state.wasm.analyzeFrame(
        message.frameIndex,
        {
          hud: message.hudBuf,
          meter: message.meterBuf,
          input: message.inputBuf,
        },
        { width: ANALYSIS_WIDTH, height: ANALYSIS_HEIGHT },
      );
      respond(
        scope,
        {
          type: "frameResult",
          slot: message.slot,
          ...timing,
          hudBuf: message.hudBuf,
          meterBuf: message.meterBuf,
          inputBuf: message.inputBuf,
        },
        [message.hudBuf, message.meterBuf, message.inputBuf],
      );
      break;
    }
    case "finish": {
      const result = state.wasm.finishFirstPass();
      state.firstPassPayload = result.payload;
      respond(scope, {
        type: "firstPass",
        spatialWindows: result.spatialWindows,
      });
      break;
    }
    case "spatialReset":
      state.wasm.resetSpatialWindow();
      respond(scope, { type: "spatialResetReady" });
      break;
    case "spatialFrame":
      state.wasm.analyzeSpatialFrame(
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
      const spatial = state.wasm.finishSpatialPass();
      respond(scope, {
        type: "done",
        ...state.firstPassPayload,
        ...spatial,
      });
      break;
    }
  }
}

function respond(
  scope: DedicatedWorkerGlobalScope,
  message: AnalyzerWorkerResponse,
  transfer: Transferable[] = [],
): void {
  scope.postMessage(message, { transfer });
}
