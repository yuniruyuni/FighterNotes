import type { AnalysisProgress } from "../../domain/result.js";
import { TransferBufferPool } from "../frame-extraction/strip-buffer-pool.js";
import {
  copyStripPixels,
  type StripPixels,
} from "../frame-extraction/strip-extractor.js";
import type {
  MeterFrameResult,
  ResultFrameResult,
} from "../worker-bridge/client.js";
import { throwIfAborted } from "./abort.js";

interface PartialWorkerFrameResult {
  meter?: MeterFrameResult;
  result?: ResultFrameResult;
}

interface WorkerFrameBridgeOptions {
  readonly sendMeter: (message: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly meterBuf: ArrayBuffer;
  }) => Promise<void>;
  readonly sendResult: (message: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly hudBuf: ArrayBuffer;
    readonly inputBuf: ArrayBuffer;
  }) => Promise<void>;
  readonly totalSamples: () => number;
  readonly drawTime: () => number;
  readonly onProgress: AnalysisProgress;
  readonly onFrameCompleted: () => void;
  readonly signal: AbortSignal;
}

export class WorkerFrameBridge {
  readonly #options: WorkerFrameBridgeOptions;
  readonly #bufferPool = new TransferBufferPool(2);
  readonly #partialResults = new Map<number, PartialWorkerFrameResult>();
  #completedFrames = 0;
  #copyTime = 0;
  #meterTime = 0;
  #hudTime = 0;

  constructor(options: WorkerFrameBridgeOptions) {
    this.#options = options;
  }

  get completedFrames(): number {
    return this.#completedFrames;
  }

  get timing() {
    return {
      tCopy: this.#copyTime,
      tMeter: this.#meterTime,
      tHud: this.#hudTime,
    };
  }

  acceptMeter(result: MeterFrameResult): void {
    this.#copyTime += result.tCopy;
    this.#meterTime += result.tMeter;
    const partial = this.#partialResult(result.slot);
    if (partial.meter) {
      throw new Error(`Duplicate meter frame result for slot ${result.slot}`);
    }
    partial.meter = result;
    this.#completeIfReady(result.slot, partial);
  }

  acceptResult(result: ResultFrameResult): void {
    this.#copyTime += result.tCopy;
    this.#hudTime += result.tHud;
    const partial = this.#partialResult(result.slot);
    if (partial.result) {
      throw new Error(`Duplicate result frame result for slot ${result.slot}`);
    }
    partial.result = result;
    this.#completeIfReady(result.slot, partial);
  }

  async send(frameIndex: number, pixels: StripPixels): Promise<void> {
    throwIfAborted(this.#options.signal);
    const slot = await this.#bufferPool.acquire(this.#options.signal);
    throwIfAborted(this.#options.signal);
    const buffers = this.#bufferPool.get(slot);
    copyStripPixels(pixels, buffers);
    await Promise.all([
      this.#options.sendMeter({
        slot,
        frameIndex,
        meterBuf: buffers.meter,
      }),
      this.#options.sendResult({
        slot,
        frameIndex,
        hudBuf: buffers.hud,
        inputBuf: buffers.input,
      }),
    ]);

    if ((frameIndex + 1) % 300 === 0) this.#logProgress(frameIndex + 1);
    this.#options.onProgress(
      ((frameIndex + 1) / this.#options.totalSamples()) * 0.9,
      `フレーム ${frameIndex + 1} / ${this.#options.totalSamples()}`,
    );
  }

  #partialResult(slot: number): PartialWorkerFrameResult {
    const partial = this.#partialResults.get(slot) ?? {};
    this.#partialResults.set(slot, partial);
    return partial;
  }

  #completeIfReady(slot: number, partial: PartialWorkerFrameResult): void {
    if (!partial.meter || !partial.result) return;
    this.#partialResults.delete(slot);
    this.#bufferPool.release(slot, {
      hud: partial.result.hudBuf,
      meter: partial.meter.meterBuf,
      input: partial.result.inputBuf,
    });
    this.#completedFrames += 1;
    this.#options.onFrameCompleted();
  }

  #logProgress(frameCount: number): void {
    const ms = (value: number) => `${value.toFixed(0)}ms`;
    console.log(
      `[perf] ${frameCount}f 累計: draw+get=${ms(this.#options.drawTime())} worker_copy=${ms(this.#copyTime)} meter=${ms(this.#meterTime)} hud=${ms(this.#hudTime)}`,
    );
  }
}
