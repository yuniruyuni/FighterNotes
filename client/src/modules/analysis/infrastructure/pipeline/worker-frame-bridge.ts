import type { AnalysisProgress } from "../../domain/result.js";
import { TransferBufferPool } from "../frame-extraction/strip-buffer-pool.js";
import {
  copyStripPixels,
  type StripPixels,
} from "../frame-extraction/strip-extractor.js";
import { throwIfAborted } from "./abort.js";

interface WorkerFrameResult {
  readonly slot: number;
  readonly tCopy: number;
  readonly tMeter: number;
  readonly tHud: number;
  readonly hudBuf: ArrayBuffer;
  readonly meterBuf: ArrayBuffer;
  readonly inputBuf: ArrayBuffer;
}

interface WorkerFrameBridgeOptions {
  readonly send: (message: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly hudBuf: ArrayBuffer;
    readonly meterBuf: ArrayBuffer;
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

  accept(result: WorkerFrameResult): void {
    this.#copyTime += result.tCopy;
    this.#meterTime += result.tMeter;
    this.#hudTime += result.tHud;
    this.#bufferPool.release(result.slot, {
      hud: result.hudBuf,
      meter: result.meterBuf,
      input: result.inputBuf,
    });
    this.#completedFrames += 1;
    this.#options.onFrameCompleted();
  }

  async send(frameIndex: number, pixels: StripPixels): Promise<void> {
    throwIfAborted(this.#options.signal);
    const slot = await this.#bufferPool.acquire();
    throwIfAborted(this.#options.signal);
    const buffers = this.#bufferPool.get(slot);
    copyStripPixels(pixels, buffers);
    await this.#options.send({
      slot,
      frameIndex,
      hudBuf: buffers.hud,
      meterBuf: buffers.meter,
      inputBuf: buffers.input,
    });

    if ((frameIndex + 1) % 300 === 0) this.#logProgress(frameIndex + 1);
    this.#options.onProgress(
      ((frameIndex + 1) / this.#options.totalSamples()) * 0.9,
      `フレーム ${frameIndex + 1} / ${this.#options.totalSamples()}`,
    );
  }

  #logProgress(frameCount: number): void {
    const ms = (value: number) => `${value.toFixed(0)}ms`;
    console.log(
      `[perf] ${frameCount}f 累計: draw+get=${ms(this.#options.drawTime())} worker_copy=${ms(this.#copyTime)} meter=${ms(this.#meterTime)} hud=${ms(this.#hudTime)}`,
    );
  }
}
