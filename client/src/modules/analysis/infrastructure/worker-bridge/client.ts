import type { AnalysisContext } from "../../domain/context.js";
import type {
  SpatialCandidateWindow,
  SpatialFrameHints,
} from "../../domain/result.js";
import {
  SPATIAL_DECODER_OUTSTANDING_WATERMARKS,
  SPATIAL_DECODER_QUEUE_WATERMARKS,
  SPATIAL_WORKER_PENDING_WATERMARKS,
  type SpatialDecodeStats,
} from "../spatial-analysis/backpressure.js";
import { HighLowWatermarkGate } from "./high-low-watermark.js";
import type { AnalyzerWorkerDone, AnalyzerWorkerResponse } from "./protocol.js";
import { postAnalyzerWorkerMessage } from "./protocol.js";

export interface ResultFrameResult {
  readonly slot: number;
  readonly superBuf: ArrayBuffer;
  readonly tCopy: number;
  readonly tHud: number;
  readonly hudBuf: ArrayBuffer;
  readonly inputBuf: ArrayBuffer;
}

export interface MeterFrameResult {
  readonly slot: number;
  readonly tCopy: number;
  readonly tMeter: number;
  readonly meterBuf: ArrayBuffer;
}

export interface AttackFrameResult {
  readonly slot: number;
  readonly tCopy: number;
  readonly tAttack: number;
  readonly meterBuf: ArrayBuffer;
}

interface WorkerSessionCallbacks {
  readonly onFrameResult: (result: ResultFrameResult) => void;
  readonly onError: (error: unknown) => void;
}

interface MeterWorkerSessionCallbacks {
  readonly onFrameResult: (result: MeterFrameResult) => void;
  readonly onError: (error: unknown) => void;
}

interface AttackWorkerSessionCallbacks {
  readonly onFrameResult: (result: AttackFrameResult) => void;
  readonly onError: (error: unknown) => void;
}

export class AnalyzerWorkerSession {
  readonly #worker: Worker;
  readonly #callbacks: WorkerSessionCallbacks;
  readonly #ready = deferred<void>();
  readonly #firstPass = deferred<SpatialCandidateWindow[]>();
  readonly #done = deferred<AnalyzerWorkerDone>();
  readonly #frameDrainWaiters: Array<Deferred<void>> = [];
  readonly #spatialDrainWaiters: Array<Deferred<void>> = [];
  readonly #spatialGate = new HighLowWatermarkGate(
    SPATIAL_WORKER_PENDING_WATERMARKS.high,
    SPATIAL_WORKER_PENDING_WATERMARKS.low,
  );
  #spatialReset: ReturnType<typeof deferred<void>> | null = null;
  #pendingFrames = 0;
  #spatialFrameCount = 0;
  #terminated = false;
  #terminationReason: unknown;

  constructor(worker: Worker, callbacks: WorkerSessionCallbacks) {
    this.#worker = worker;
    this.#callbacks = callbacks;
    worker.onerror = (event) => this.#fail(event);
    worker.onmessage = (event: MessageEvent<AnalyzerWorkerResponse>) => {
      try {
        this.#receive(event.data);
      } catch (error) {
        this.#fail(error);
      }
    };
  }

  initialize(
    ownSide: string,
    analysisContext: AnalysisContext,
    hudFromGpu = false,
  ): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "init",
      role: "result",
      ownSide,
      analysisContext,
      hudFromGpu,
    });
  }

  /** 主スレッドの GPU が読み取ったまとまりを送る。 */
  sendHudGpuBatch(batch: {
    readonly firstFrame: number;
    readonly scores: Uint32Array;
    readonly columns: Uint32Array;
    readonly drive: Uint32Array;
  }): void {
    if (this.#terminated) return;
    postAnalyzerWorkerMessage(this.#worker, { type: "hudGpuBatch", ...batch });
  }

  async sendFrame(options: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly hudBuf: ArrayBuffer;
    readonly superBuf: ArrayBuffer;
    readonly inputBuf: ArrayBuffer;
    readonly frame?: VideoFrame;
  }): Promise<void> {
    this.#throwIfTerminated();
    await this.#ready.promise;
    this.#throwIfTerminated();
    this.#pendingFrames += 1;
    postAnalyzerWorkerMessage(
      this.#worker,
      { type: "resultFrame", ...options },
      [options.hudBuf, options.inputBuf],
    );
  }

  drainFrames(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#pendingFrames === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#frameDrainWaiters.push(waiter);
    return waiter.promise;
  }

  finishFirstPass(meterTimeline: string, attackInfo: string): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "finish",
      meterTimeline,
      attackInfo,
    });
  }

  firstPass(): Promise<SpatialCandidateWindow[]> {
    return this.#firstPass.promise;
  }

  resetSpatialWindow(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#spatialReset) {
      throw new Error("Spatial reset is already pending");
    }
    this.#spatialReset = deferred<void>();
    postAnalyzerWorkerMessage(this.#worker, { type: "spatialReset" });
    return this.#spatialReset.promise;
  }

  async sendSpatialFrame(
    frameIndex: number,
    createRgbaBuffer: () => ArrayBuffer,
    hints: SpatialFrameHints,
    signal?: AbortSignal,
  ): Promise<void> {
    this.#throwIfTerminated();
    await this.#spatialGate.acquire(signal);
    try {
      this.#throwIfTerminated();
      throwIfAborted(signal);
      const rgbaBuf = createRgbaBuffer();
      this.#throwIfTerminated();
      throwIfAborted(signal);
      postAnalyzerWorkerMessage(
        this.#worker,
        { type: "spatialFrame", frameIndex, rgbaBuf, hints },
        [rgbaBuf],
      );
      this.#spatialFrameCount += 1;
    } catch (error) {
      this.#spatialGate.release();
      if (this.#spatialGate.active === 0) {
        drain(this.#spatialDrainWaiters);
      }
      throw error;
    }
  }

  drainSpatialFrames(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#spatialGate.active === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#spatialDrainWaiters.push(waiter);
    return waiter.promise;
  }

  finishSpatialPass(decodeStats: SpatialDecodeStats): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "spatialFinish",
      spatialPerformance: {
        frameCount: this.#spatialFrameCount,
        decoderQueueHighWatermark: SPATIAL_DECODER_QUEUE_WATERMARKS.high,
        decoderQueueLowWatermark: SPATIAL_DECODER_QUEUE_WATERMARKS.low,
        decoderOutstandingHighWatermark:
          SPATIAL_DECODER_OUTSTANDING_WATERMARKS.high,
        decoderOutstandingLowWatermark:
          SPATIAL_DECODER_OUTSTANDING_WATERMARKS.low,
        workerPendingHighWatermark: SPATIAL_WORKER_PENDING_WATERMARKS.high,
        workerPendingLowWatermark: SPATIAL_WORKER_PENDING_WATERMARKS.low,
        peakWorkerPendingFrames: this.#spatialGate.peak,
        ...decodeStats,
      },
    });
  }

  result(): Promise<AnalyzerWorkerDone> {
    return this.#done.promise;
  }

  terminate(reason: unknown = new Error("解析Workerを終了しました")): void {
    if (this.#terminated) return;
    this.#terminated = true;
    this.#terminationReason = reason;
    this.#worker.onerror = null;
    this.#worker.onmessage = null;
    this.#worker.terminate();
    this.#ready.reject(reason);
    this.#firstPass.reject(reason);
    this.#done.reject(reason);
    this.#spatialReset?.reject(reason);
    this.#spatialReset = null;
    rejectWaiters(this.#frameDrainWaiters, reason);
    rejectWaiters(this.#spatialDrainWaiters, reason);
    this.#spatialGate.close(reason);
    this.#pendingFrames = 0;
  }

  #receive(message: AnalyzerWorkerResponse): void {
    if (this.#terminated) return;
    switch (message.type) {
      case "error":
        this.#fail(new Error(message.message));
        break;
      case "ready":
        this.#ready.resolve();
        break;
      case "resultFrameResult":
        this.#pendingFrames -= 1;
        this.#callbacks.onFrameResult(message);
        if (this.#pendingFrames === 0) drain(this.#frameDrainWaiters);
        break;
      case "spatialResetReady":
        this.#spatialReset?.resolve();
        this.#spatialReset = null;
        break;
      case "spatialFrameResult":
        this.#spatialGate.release();
        if (this.#spatialGate.active === 0) {
          drain(this.#spatialDrainWaiters);
        }
        break;
      case "firstPass":
        this.#firstPass.resolve(JSON.parse(message.spatialWindows));
        break;
      case "done":
        this.#done.resolve(message);
        break;
    }
  }

  #fail(error: unknown): void {
    this.terminate(error);
    this.#callbacks.onError(error);
  }

  #throwIfTerminated(): void {
    if (this.#terminated) throw this.#terminationReason;
  }
}

export class MeterWorkerSession {
  readonly #worker: Worker;
  readonly #callbacks: MeterWorkerSessionCallbacks;
  readonly #ready = deferred<void>();
  readonly #timeline = deferred<string>();
  readonly #frameDrainWaiters: Array<Deferred<void>> = [];
  #pendingFrames = 0;
  #terminated = false;
  #terminationReason: unknown;

  constructor(worker: Worker, callbacks: MeterWorkerSessionCallbacks) {
    this.#worker = worker;
    this.#callbacks = callbacks;
    worker.onerror = (event) => this.#fail(event);
    worker.onmessage = (event: MessageEvent<AnalyzerWorkerResponse>) => {
      this.#receive(event.data);
    };
  }

  initialize(ownSide: string, analysisContext: AnalysisContext): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "init",
      role: "meter",
      ownSide,
      analysisContext,
    });
  }

  async sendFrame(options: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly meterBuf: ArrayBuffer;
  }): Promise<void> {
    this.#throwIfTerminated();
    await this.#ready.promise;
    this.#throwIfTerminated();
    this.#pendingFrames += 1;
    postAnalyzerWorkerMessage(
      this.#worker,
      { type: "meterFrame", ...options },
      [options.meterBuf],
    );
  }

  drainFrames(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#pendingFrames === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#frameDrainWaiters.push(waiter);
    return waiter.promise;
  }

  finish(): Promise<string> {
    if (this.#terminated) return rejected(this.#terminationReason);
    postAnalyzerWorkerMessage(this.#worker, { type: "finishMeter" });
    return this.#timeline.promise;
  }

  terminate(reason: unknown = new Error("解析Workerを終了しました")): void {
    if (this.#terminated) return;
    this.#terminated = true;
    this.#terminationReason = reason;
    this.#worker.onerror = null;
    this.#worker.onmessage = null;
    this.#worker.terminate();
    this.#ready.reject(reason);
    this.#timeline.reject(reason);
    rejectWaiters(this.#frameDrainWaiters, reason);
    this.#pendingFrames = 0;
  }

  #receive(message: AnalyzerWorkerResponse): void {
    if (this.#terminated) return;
    switch (message.type) {
      case "error":
        this.#fail(new Error(message.message));
        break;
      case "ready":
        this.#ready.resolve();
        break;
      case "meterFrameResult":
        this.#pendingFrames -= 1;
        this.#callbacks.onFrameResult(message);
        if (this.#pendingFrames === 0) drain(this.#frameDrainWaiters);
        break;
      case "meterDone":
        this.#timeline.resolve(message.timeline);
        break;
    }
  }

  #fail(error: unknown): void {
    this.terminate(error);
    this.#callbacks.onError(error);
  }

  #throwIfTerminated(): void {
    if (this.#terminated) throw this.#terminationReason;
  }
}

/**
 * 攻撃情報だけを読むワーカー。
 *
 * 読み取りは meter strip だけで決まる純粋な処理で、実機計測では meter
 * ワーカーの費用の半分（1 フレーム 1.1ms）を占めていた。別のワーカーへ
 * 出しても結果は変わらない。
 */
export class AttackWorkerSession {
  readonly #worker: Worker;
  readonly #callbacks: AttackWorkerSessionCallbacks;
  readonly #ready = deferred<void>();
  readonly #attackInfo = deferred<string>();
  readonly #frameDrainWaiters: Array<Deferred<void>> = [];
  #pendingFrames = 0;
  #terminated = false;
  #terminationReason: unknown;

  constructor(worker: Worker, callbacks: AttackWorkerSessionCallbacks) {
    this.#worker = worker;
    this.#callbacks = callbacks;
    worker.onerror = (event) => this.#fail(event);
    worker.onmessage = (event: MessageEvent<AnalyzerWorkerResponse>) => {
      this.#receive(event.data);
    };
  }

  initialize(ownSide: string, analysisContext: AnalysisContext): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "init",
      role: "attackInfo",
      ownSide,
      analysisContext,
    });
  }

  async sendFrame(options: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly meterBuf: ArrayBuffer;
  }): Promise<void> {
    this.#throwIfTerminated();
    await this.#ready.promise;
    this.#throwIfTerminated();
    this.#pendingFrames += 1;
    postAnalyzerWorkerMessage(
      this.#worker,
      { type: "attackFrame", ...options },
      [options.meterBuf],
    );
  }

  drainFrames(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#pendingFrames === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#frameDrainWaiters.push(waiter);
    return waiter.promise;
  }

  finish(): Promise<string> {
    if (this.#terminated) return rejected(this.#terminationReason);
    postAnalyzerWorkerMessage(this.#worker, { type: "finishAttack" });
    return this.#attackInfo.promise;
  }

  terminate(reason: unknown = new Error("解析Workerを終了しました")): void {
    if (this.#terminated) return;
    this.#terminated = true;
    this.#terminationReason = reason;
    this.#worker.onerror = null;
    this.#worker.onmessage = null;
    this.#worker.terminate();
    this.#ready.reject(reason);
    this.#attackInfo.reject(reason);
    rejectWaiters(this.#frameDrainWaiters, reason);
    this.#pendingFrames = 0;
  }

  #receive(message: AnalyzerWorkerResponse): void {
    if (this.#terminated) return;
    switch (message.type) {
      case "error":
        this.#fail(new Error(message.message));
        break;
      case "ready":
        this.#ready.resolve();
        break;
      case "attackFrameResult":
        this.#pendingFrames -= 1;
        this.#callbacks.onFrameResult(message);
        if (this.#pendingFrames === 0) drain(this.#frameDrainWaiters);
        break;
      case "attackDone":
        this.#attackInfo.resolve(message.attackInfo);
        break;
    }
  }

  #fail(error: unknown): void {
    this.terminate(error);
    this.#callbacks.onError(error);
  }

  #throwIfTerminated(): void {
    if (this.#terminated) throw this.#terminationReason;
  }
}

interface Deferred<T> {
  readonly promise: Promise<T>;
  readonly resolve: (value: T | PromiseLike<T>) => void;
  readonly reject: (reason?: unknown) => void;
}

function deferred<T>(): Deferred<T> {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  // A worker may fail before a later lifecycle phase starts waiting. Observe
  // shutdown rejections immediately while preserving rejection for callers.
  void promise.catch(() => {});
  return { promise, resolve, reject };
}

function rejected<T>(reason: unknown): Promise<T> {
  const value = deferred<T>();
  value.reject(reason);
  return value.promise;
}

function throwIfAborted(signal?: AbortSignal): void {
  if (!signal?.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new Error("空間フレーム送信を中断しました");
}

function drain(waiters: Array<Deferred<void>>): void {
  for (const waiter of waiters.splice(0)) waiter.resolve();
}

function rejectWaiters(waiters: Array<Deferred<void>>, reason: unknown): void {
  for (const waiter of waiters.splice(0)) waiter.reject(reason);
}
