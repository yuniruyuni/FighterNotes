import init, {
  Analyzer,
  SpatialWindowAnalyzer,
  wasm_memory,
} from "../../../../../../crates/wasm-bridge/pkg/wasm_bridge.js";
import type { AnalysisContext } from "../../domain/context.js";
import type { SpatialFrameHints } from "../../domain/result.js";
import { buildHpDebugSnapshot } from "./hp-debug-snapshot.js";

export interface WasmFrameBuffers {
  readonly hud: ArrayBuffer;
  readonly meter: ArrayBuffer;
  readonly input: ArrayBuffer;
}

export interface WasmFrameTiming {
  readonly tCopy: number;
  readonly tMeter: number;
  readonly tHud: number;
}

export interface WasmFirstPassPayload {
  readonly report: string;
  readonly timeline: string;
  readonly features: string;
  readonly trackedInputs: string;
  readonly debugHp: unknown[];
}

export interface WasmFirstPassResult {
  readonly payload: WasmFirstPassPayload;
  readonly spatialWindows: string;
}

interface AnalyzerWasmState {
  readonly analyzer: InstanceType<typeof Analyzer>;
  readonly spatialAnalyzer: InstanceType<typeof SpatialWindowAnalyzer>;
  readonly memory: WebAssembly.Memory;
  readonly hudPtr: number;
  readonly hudLen: number;
  readonly meterPtr: number;
  readonly meterLen: number;
  readonly inputPtr: number;
  readonly inputLen: number;
  readonly spatialPtr: number;
  readonly spatialLen: number;
}

export class AnalyzerWasmSession {
  #state: AnalyzerWasmState | null = null;

  async initialize(options: {
    readonly ownSide: string;
    readonly analysisContext: AnalysisContext;
    readonly spatialWidth: number;
    readonly spatialHeight: number;
  }): Promise<void> {
    await init();
    const analyzer = new Analyzer(options.ownSide);
    analyzer.set_analysis_context(JSON.stringify(options.analysisContext));
    const spatialAnalyzer = new SpatialWindowAnalyzer(
      options.spatialWidth,
      options.spatialHeight,
      true,
    );
    this.#state = {
      analyzer,
      spatialAnalyzer,
      memory: wasm_memory() as WebAssembly.Memory,
      hudPtr: analyzer.hud_buf_ptr() as number,
      hudLen: analyzer.hud_buf_len() as number,
      meterPtr: analyzer.meter_buf_ptr() as number,
      meterLen: analyzer.meter_buf_len() as number,
      inputPtr: analyzer.input_buf_ptr() as number,
      inputLen: analyzer.input_buf_len() as number,
      spatialPtr: spatialAnalyzer.rgba_buf_ptr() as number,
      spatialLen: spatialAnalyzer.rgba_buf_len() as number,
    };
  }

  analyzeFrame(
    frameIndex: number,
    buffers: WasmFrameBuffers,
    dimensions: { readonly width: number; readonly height: number },
  ): WasmFrameTiming {
    const state = this.#requireState();
    const t0 = performance.now();
    copyToWasm(state, state.hudPtr, state.hudLen, buffers.hud);
    copyToWasm(state, state.meterPtr, state.meterLen, buffers.meter);
    copyToWasm(state, state.inputPtr, state.inputLen, buffers.input);
    const t1 = performance.now();

    state.analyzer.analyze_meter_inplace(
      dimensions.width,
      dimensions.height,
      frameIndex,
    );
    const t2 = performance.now();
    state.analyzer.push_hud_features_inplace(
      dimensions.width,
      dimensions.height,
      frameIndex,
    );
    state.analyzer.analyze_input_inplace(
      dimensions.width,
      dimensions.height,
      frameIndex,
    );
    const t3 = performance.now();
    return { tCopy: t1 - t0, tMeter: t2 - t1, tHud: t3 - t2 };
  }

  finishFirstPass(): WasmFirstPassResult {
    const { analyzer } = this.#requireState();
    const report = analyzer.finish();
    const timeline = analyzer.get_timeline();
    const features = analyzer.get_features_json();
    return {
      payload: {
        report,
        timeline,
        features,
        trackedInputs: analyzer.get_tracked_inputs(),
        debugHp: buildHpDebugSnapshot(features),
      },
      spatialWindows: analyzer.get_spatial_windows_json(),
    };
  }

  resetSpatialWindow(): void {
    this.#requireState().spatialAnalyzer.reset_window();
  }

  analyzeSpatialFrame(
    frameIndex: number,
    rgbaBuffer: ArrayBuffer,
    hints: SpatialFrameHints,
  ): void {
    const state = this.#requireState();
    if (rgbaBuffer.byteLength !== state.spatialLen) {
      throw new Error(
        `spatial RGBA length mismatch: expected ${state.spatialLen}, got ${rgbaBuffer.byteLength}`,
      );
    }
    copyToWasm(state, state.spatialPtr, state.spatialLen, rgbaBuffer);
    state.spatialAnalyzer.observe_inplace(
      frameIndex,
      hints.p1Teleport,
      hints.p2Teleport,
      hints.p1Airborne,
      hints.p2Airborne,
    );
  }

  finishSpatialPass(): {
    readonly report: string;
    readonly spatialObservations: string;
  } {
    const state = this.#requireState();
    const spatialObservations = state.spatialAnalyzer.get_observations_json();
    return {
      report: state.analyzer.refine_with_spatial(spatialObservations),
      spatialObservations,
    };
  }

  #requireState(): AnalyzerWasmState {
    if (!this.#state)
      throw new Error("Analyzer WASM session is not initialized");
    return this.#state;
  }
}

function copyToWasm(
  state: AnalyzerWasmState,
  pointer: number,
  length: number,
  source: ArrayBuffer,
): void {
  new Uint8Array(state.memory.buffer, pointer, length).set(
    new Uint8Array(source),
  );
}
