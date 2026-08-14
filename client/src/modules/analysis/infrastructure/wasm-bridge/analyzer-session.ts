import init, {
  Analyzer,
  SpatialWindowAnalyzer,
  wasm_memory,
} from "../../../../../../crates/wasm-bridge/pkg/wasm_bridge.js";
import type { AnalysisContext } from "../../domain/context.js";
import type { SpatialFrameHints } from "../../domain/result.js";
import { buildHpDebugSnapshot } from "./hp-debug-snapshot.js";

export interface WasmResultFrameBuffers {
  readonly hud: ArrayBuffer;
  readonly super: ArrayBuffer;
  readonly input: ArrayBuffer;
}

export interface WasmResultFrameTiming {
  readonly tCopy: number;
  readonly tHud: number;
}

export interface WasmMeterFrameTiming {
  readonly tCopy: number;
  readonly tMeter: number;
}

export interface WasmAttackFrameTiming {
  readonly tCopy: number;
  readonly tAttack: number;
}

export interface WasmFirstPassPayload {
  readonly report: string;
  readonly timeline: string;
  readonly features: string;
  readonly trackedInputs: string;
  readonly fightMarkers: string;
  readonly attackInfo: string;
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
  readonly superPtr: number;
  readonly superLen: number;
  readonly inputPtr: number;
  readonly inputLen: number;
  readonly spatialPtr: number;
  readonly spatialLen: number;
}

export class AnalyzerWasmSession {
  #state: AnalyzerWasmState | null = null;
  #hudFromGpu = false;

  async initialize(options: {
    readonly ownSide: string;
    readonly analysisContext: AnalysisContext;
    readonly spatialWidth: number;
    readonly spatialHeight: number;
    /** HUD の画素読み取りを主スレッドの GPU が担うか。 */
    readonly hudFromGpu: boolean;
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
      superPtr: analyzer.super_buf_ptr() as number,
      superLen: analyzer.super_buf_len() as number,
      inputPtr: analyzer.input_buf_ptr() as number,
      inputLen: analyzer.input_buf_len() as number,
      spatialPtr: spatialAnalyzer.rgba_buf_ptr() as number,
      spatialLen: spatialAnalyzer.rgba_buf_len() as number,
    };
    this.#hudFromGpu = options.hudFromGpu;
    if (options.hudFromGpu) {
      analyzer.use_gpu_hp_scores();
      analyzer.use_gpu_hp_columns();
      analyzer.use_gpu_drive();
    }
  }

  async analyzeResultFrame(
    frameIndex: number,
    buffers: WasmResultFrameBuffers,
    dimensions: { readonly width: number; readonly height: number },
  ): Promise<WasmResultFrameTiming> {
    const state = this.#requireState();
    const t0 = performance.now();
    copyToWasm(state, state.hudPtr, state.hudLen, buffers.hud);
    copyToWasm(state, state.superPtr, state.superLen, buffers.super);
    copyToWasm(state, state.inputPtr, state.inputLen, buffers.input);
    const t1 = performance.now();

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
    const t2 = performance.now();
    return { tCopy: t1 - t0, tHud: t2 - t1 };
  }

  async finishFirstPass(
    meterTimeline: string,
    attackInfo: string,
  ): Promise<WasmFirstPassResult> {
    const { analyzer } = this.#requireState();
    analyzer.set_meter_timeline(meterTimeline);
    // タイムラインが運ぶ観測は空なので、読み取ったワーカーの結果で置き換える。
    analyzer.set_attack_info_json(attackInfo);
    const report = analyzer.finish();
    const timeline = analyzer.get_timeline();
    const features = analyzer.get_features_json();
    void fetch("/result", {
      method: "POST",
      body: `[perf] report ${report}`,
    });
    return {
      payload: {
        report,
        timeline,
        features,
        trackedInputs: analyzer.get_tracked_inputs(),
        fightMarkers: analyzer.get_fight_markers_json(),
        attackInfo: analyzer.get_attack_info_json(),
        debugHp: buildHpDebugSnapshot(features),
      },
      spatialWindows: analyzer.get_spatial_windows_json(),
    };
  }

  /** 主スレッドの GPU が読み取ったまとまりを受け取る。 */
  acceptHudGpuBatch(
    firstFrame: number,
    scores: Uint32Array,
    columns: Uint32Array,
    drive: Uint32Array,
  ): void {
    if (!this.#hudFromGpu) return;
    const { analyzer } = this.#requireState();
    analyzer.push_hp_score_counts(firstFrame, scores);
    analyzer.apply_hp_columns(firstFrame, columns);
    analyzer.apply_drive_columns(firstFrame, drive);
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
    readonly regressionEvents: string;
  } {
    const state = this.#requireState();
    const spatialObservations = state.spatialAnalyzer.get_observations_json();
    return {
      report: state.analyzer.refine_with_spatial(spatialObservations),
      spatialObservations,
      regressionEvents: state.analyzer.get_regression_events_json(),
    };
  }

  #requireState(): AnalyzerWasmState {
    if (!this.#state)
      throw new Error("Analyzer WASM session is not initialized");
    return this.#state;
  }
}

interface MeterWasmState {
  readonly analyzer: InstanceType<typeof Analyzer>;
  readonly memory: WebAssembly.Memory;
  readonly meterPtr: number;
  readonly meterLen: number;
}

export class MeterWasmSession {
  #state: MeterWasmState | null = null;

  async initialize(ownSide: string): Promise<void> {
    await init();
    const analyzer = new Analyzer(ownSide);
    this.#state = {
      analyzer,
      memory: wasm_memory() as WebAssembly.Memory,
      meterPtr: analyzer.meter_buf_ptr() as number,
      meterLen: analyzer.meter_buf_len() as number,
    };
  }

  analyzeFrame(
    frameIndex: number,
    meterBuffer: ArrayBuffer,
    dimensions: { readonly width: number; readonly height: number },
  ): WasmMeterFrameTiming {
    const state = this.#requireState();
    const t0 = performance.now();
    copyToWasm(state, state.meterPtr, state.meterLen, meterBuffer);
    const t1 = performance.now();
    state.analyzer.analyze_meter_inplace(
      dimensions.width,
      dimensions.height,
      frameIndex,
    );
    const t2 = performance.now();
    return { tCopy: t1 - t0, tMeter: t2 - t1 };
  }

  finish(): string {
    return this.#requireState().analyzer.finish_meter_timeline();
  }

  analyzeAttackFrame(
    frameIndex: number,
    meterBuffer: ArrayBuffer,
    dimensions: { readonly width: number; readonly height: number },
  ): WasmAttackFrameTiming {
    const state = this.#requireState();
    const t0 = performance.now();
    copyToWasm(state, state.meterPtr, state.meterLen, meterBuffer);
    const t1 = performance.now();
    state.analyzer.analyze_attack_info_inplace(dimensions.width, frameIndex);
    const t2 = performance.now();
    return { tCopy: t1 - t0, tAttack: t2 - t1 };
  }

  finishAttackInfo(): string {
    return this.#requireState().analyzer.get_attack_info_json();
  }

  #requireState(): MeterWasmState {
    if (!this.#state) throw new Error("Meter WASM session is not initialized");
    return this.#state;
  }
}

function copyToWasm(
  state: { readonly memory: WebAssembly.Memory },
  pointer: number,
  length: number,
  source: ArrayBuffer,
): void {
  new Uint8Array(state.memory.buffer, pointer, length).set(
    new Uint8Array(source),
  );
}
