import type { AnalysisContext } from "../../domain/context.js";
import type {
  SpatialCandidateWindow,
  SpatialFrameHints,
} from "../../domain/result.js";
import type { AnalyzerWorkerDone, AnalyzerWorkerResponse } from "./protocol.js";
import { postAnalyzerWorkerMessage } from "./protocol.js";

export interface ResultFrameResult {
  readonly slot: number;
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

interface WorkerSessionCallbacks {
  readonly onFrameResult: (result: ResultFrameResult) => void;
  readonly onError: (error: unknown) => void;
}

interface MeterWorkerSessionCallbacks {
  readonly onFrameResult: (result: MeterFrameResult) => void;
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
  #spatialReset: ReturnType<typeof deferred<void>> | null = null;
  #pendingFrames = 0;
  #pendingSpatialFrames = 0;
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

  initialize(ownSide: string, analysisContext: AnalysisContext): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "init",
      role: "result",
      ownSide,
      analysisContext,
    });
  }

  async sendFrame(options: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly hudBuf: ArrayBuffer;
    readonly inputBuf: ArrayBuffer;
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

  finishFirstPass(meterTimeline: string): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, {
      type: "finish",
      meterTimeline,
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

  sendSpatialFrame(
    frameIndex: number,
    rgbaBuf: ArrayBuffer,
    hints: SpatialFrameHints,
  ): void {
    this.#throwIfTerminated();
    this.#pendingSpatialFrames += 1;
    postAnalyzerWorkerMessage(
      this.#worker,
      { type: "spatialFrame", frameIndex, rgbaBuf, hints },
      [rgbaBuf],
    );
  }

  drainSpatialFrames(): Promise<void> {
    if (this.#terminated) return rejected(this.#terminationReason);
    if (this.#pendingSpatialFrames === 0) return Promise.resolve();
    const waiter = deferred<void>();
    this.#spatialDrainWaiters.push(waiter);
    return waiter.promise;
  }

  finishSpatialPass(): void {
    this.#throwIfTerminated();
    postAnalyzerWorkerMessage(this.#worker, { type: "spatialFinish" });
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
    this.#pendingFrames = 0;
    this.#pendingSpatialFrames = 0;
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
        this.#pendingSpatialFrames -= 1;
        if (this.#pendingSpatialFrames === 0) {
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

function drain(waiters: Array<Deferred<void>>): void {
  for (const waiter of waiters.splice(0)) waiter.resolve();
}

function rejectWaiters(waiters: Array<Deferred<void>>, reason: unknown): void {
  for (const waiter of waiters.splice(0)) waiter.reject(reason);
}
