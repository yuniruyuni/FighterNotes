import type { AnalysisContext } from "../../domain/context.js";
import type {
  SpatialCandidateWindow,
  SpatialFrameHints,
} from "../../domain/result.js";
import type { AnalyzerWorkerDone, AnalyzerWorkerResponse } from "./protocol.js";
import { postAnalyzerWorkerMessage } from "./protocol.js";

interface FrameResult {
  readonly slot: number;
  readonly tCopy: number;
  readonly tMeter: number;
  readonly tHud: number;
  readonly hudBuf: ArrayBuffer;
  readonly meterBuf: ArrayBuffer;
  readonly inputBuf: ArrayBuffer;
}

interface WorkerSessionCallbacks {
  readonly onFrameResult: (result: FrameResult) => void;
  readonly onError: (error: unknown) => void;
}

export class AnalyzerWorkerSession {
  readonly #worker: Worker;
  readonly #callbacks: WorkerSessionCallbacks;
  readonly #ready = deferred<void>();
  readonly #firstPass = deferred<SpatialCandidateWindow[]>();
  readonly #done = deferred<AnalyzerWorkerDone>();
  readonly #frameDrainWaiters: Array<() => void> = [];
  readonly #spatialDrainWaiters: Array<() => void> = [];
  #spatialReset: ReturnType<typeof deferred<void>> | null = null;
  #pendingFrames = 0;
  #pendingSpatialFrames = 0;

  constructor(worker: Worker, callbacks: WorkerSessionCallbacks) {
    this.#worker = worker;
    this.#callbacks = callbacks;
    worker.onerror = (event) => this.#fail(event);
    worker.onmessage = (event: MessageEvent<AnalyzerWorkerResponse>) => {
      this.#receive(event.data);
    };
  }

  initialize(ownSide: string, analysisContext: AnalysisContext): void {
    postAnalyzerWorkerMessage(this.#worker, {
      type: "init",
      ownSide,
      analysisContext,
    });
  }

  async sendFrame(options: {
    readonly slot: number;
    readonly frameIndex: number;
    readonly hudBuf: ArrayBuffer;
    readonly meterBuf: ArrayBuffer;
    readonly inputBuf: ArrayBuffer;
  }): Promise<void> {
    await this.#ready.promise;
    this.#pendingFrames += 1;
    postAnalyzerWorkerMessage(this.#worker, { type: "frame", ...options }, [
      options.hudBuf,
      options.meterBuf,
      options.inputBuf,
    ]);
  }

  drainFrames(): Promise<void> {
    if (this.#pendingFrames === 0) return Promise.resolve();
    return new Promise((resolve) => this.#frameDrainWaiters.push(resolve));
  }

  finishFirstPass(): void {
    postAnalyzerWorkerMessage(this.#worker, { type: "finish" });
  }

  firstPass(): Promise<SpatialCandidateWindow[]> {
    return this.#firstPass.promise;
  }

  resetSpatialWindow(): Promise<void> {
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
    this.#pendingSpatialFrames += 1;
    postAnalyzerWorkerMessage(
      this.#worker,
      { type: "spatialFrame", frameIndex, rgbaBuf, hints },
      [rgbaBuf],
    );
  }

  drainSpatialFrames(): Promise<void> {
    if (this.#pendingSpatialFrames === 0) return Promise.resolve();
    return new Promise((resolve) => this.#spatialDrainWaiters.push(resolve));
  }

  finishSpatialPass(): void {
    postAnalyzerWorkerMessage(this.#worker, { type: "spatialFinish" });
  }

  result(): Promise<AnalyzerWorkerDone> {
    return this.#done.promise;
  }

  terminate(): void {
    this.#worker.terminate();
  }

  #receive(message: AnalyzerWorkerResponse): void {
    switch (message.type) {
      case "ready":
        this.#ready.resolve();
        break;
      case "frameResult":
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
    this.#ready.reject(error);
    this.#firstPass.reject(error);
    this.#done.reject(error);
    this.#spatialReset?.reject(error);
    this.#callbacks.onError(error);
  }
}

function deferred<T>() {
  let resolve!: (value: T | PromiseLike<T>) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

function drain(waiters: Array<() => void>): void {
  for (const resolve of waiters.splice(0)) resolve();
}
