import { describe, expect, test } from "bun:test";
import type { AnalysisContext } from "../../domain/context.js";
import {
  EMPTY_SPATIAL_DECODE_STATS,
  SPATIAL_WORKER_PENDING_WATERMARKS,
} from "../spatial-analysis/backpressure.js";
import { AnalyzerWorkerSession, MeterWorkerSession } from "./client.js";
import type { AnalyzerWorkerResponse } from "./protocol.js";

const analysisContext: AnalysisContext = {
  ownSide: "p1",
  p1: {},
  p2: {},
};

const rgbaBuffer = () => new ArrayBuffer(1);

class FakeWorker {
  onerror: ((event: ErrorEvent) => unknown) | null = null;
  onmessage: ((event: MessageEvent<AnalyzerWorkerResponse>) => unknown) | null =
    null;
  readonly messages: unknown[] = [];
  terminateCount = 0;

  postMessage(message: unknown): void {
    this.messages.push(message);
  }

  terminate(): void {
    this.terminateCount += 1;
  }

  receive(message: AnalyzerWorkerResponse): void {
    this.onmessage?.({ data: message } as MessageEvent<AnalyzerWorkerResponse>);
  }

  asWorker(): Worker {
    return this as unknown as Worker;
  }
}

describe("AnalyzerWorkerSession", () => {
  test("termination rejects every lifecycle and drain wait with the cancellation reason", async () => {
    const worker = new FakeWorker();
    const callbackErrors: unknown[] = [];
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: (error) => callbackErrors.push(error),
    });
    workerSession.initialize("p1", analysisContext);
    worker.receive({ type: "ready" });
    await workerSession.sendFrame({
      slot: 0,
      frameIndex: 0,
      hudBuf: new ArrayBuffer(1),
      inputBuf: new ArrayBuffer(1),
    });
    const frameDrain = workerSession.drainFrames();
    const firstPass = workerSession.firstPass();
    const spatialReset = workerSession.resetSpatialWindow();
    const spatialSend = workerSession.sendSpatialFrame(0, rgbaBuffer, {
      p1Teleport: false,
      p2Teleport: false,
      p1Airborne: false,
      p2Airborne: false,
    });
    const spatialDrain = workerSession.drainSpatialFrames();
    const result = workerSession.result();
    const reason = new Error("cancelled");

    workerSession.terminate(reason);
    workerSession.terminate(new Error("late termination"));

    await expectRejectedWith(
      [frameDrain, firstPass, spatialReset, spatialSend, spatialDrain, result],
      reason,
    );
    expect(worker.terminateCount).toBe(1);
    expect(callbackErrors).toEqual([]);
    expect(() => workerSession.finishFirstPass("{}")).toThrow("cancelled");
    await expect(workerSession.drainFrames()).rejects.toBe(reason);
  });

  test("termination before ready settles later consumers without unhandled rejections", async () => {
    const worker = new FakeWorker();
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    const reason = new Error("cancelled before ready");

    workerSession.terminate(reason);
    await Promise.resolve();

    await expect(workerSession.firstPass()).rejects.toBe(reason);
    await expect(workerSession.result()).rejects.toBe(reason);
    await expect(
      workerSession.sendFrame({
        slot: 0,
        frameIndex: 0,
        hudBuf: new ArrayBuffer(1),
        inputBuf: new ArrayBuffer(1),
      }),
    ).rejects.toBe(reason);
  });

  test("malformed first-pass results fail and settle the entire session", async () => {
    const worker = new FakeWorker();
    const callbackErrors: unknown[] = [];
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: (error) => callbackErrors.push(error),
    });
    worker.receive({ type: "ready" });
    await workerSession.sendFrame({
      slot: 0,
      frameIndex: 0,
      hudBuf: new ArrayBuffer(1),
      inputBuf: new ArrayBuffer(1),
    });
    const frameDrain = workerSession.drainFrames();
    const firstPass = workerSession.firstPass();
    const spatialReset = workerSession.resetSpatialWindow();
    const spatialSend = workerSession.sendSpatialFrame(0, rgbaBuffer, {
      p1Teleport: false,
      p2Teleport: false,
      p1Airborne: false,
      p2Airborne: false,
    });
    const spatialDrain = workerSession.drainSpatialFrames();
    const result = workerSession.result();

    expect(() =>
      worker.receive({ type: "firstPass", spatialWindows: "{" }),
    ).not.toThrow();

    expect(callbackErrors).toHaveLength(1);
    const reason = callbackErrors[0];
    expect(reason).toBeInstanceOf(SyntaxError);
    await expectRejectedWith(
      [frameDrain, firstPass, spatialReset, spatialSend, spatialDrain, result],
      reason,
    );
    expect(worker.terminateCount).toBe(1);
    await expect(workerSession.drainFrames()).rejects.toBe(reason);
    await expect(workerSession.drainSpatialFrames()).rejects.toBe(reason);
  });

  test("1,001 concurrent spatial sends never exceed the reserved high watermark", async () => {
    const worker = new FakeWorker();
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    const frameCount = 1_001;
    const sends = Array.from({ length: frameCount }, (_, frameIndex) =>
      workerSession.sendSpatialFrame(frameIndex, rgbaBuffer, {
        p1Teleport: false,
        p2Teleport: false,
        p1Airborne: false,
        p2Airborne: false,
      }),
    );
    let acknowledged = 0;
    let peakPending = 0;
    const drain = workerSession.drainSpatialFrames();

    while (acknowledged < frameCount) {
      await waitUntil(() => spatialMessages(worker).length > acknowledged);
      const sent = spatialMessages(worker).length;
      peakPending = Math.max(peakPending, sent - acknowledged);
      expect(sent - acknowledged).toBeLessThanOrEqual(
        SPATIAL_WORKER_PENDING_WATERMARKS.high,
      );
      worker.receive({ type: "spatialFrameResult" });
      acknowledged += 1;
    }

    await Promise.all(sends);
    await drain;
    workerSession.finishSpatialPass(EMPTY_SPATIAL_DECODE_STATS);
    const finish = worker.messages.find(
      (message) => messageType(message) === "spatialFinish",
    ) as {
      readonly spatialPerformance: {
        readonly frameCount: number;
        readonly peakWorkerPendingFrames: number;
      };
    };

    expect(peakPending).toBe(SPATIAL_WORKER_PENDING_WATERMARKS.high);
    expect(finish.spatialPerformance).toMatchObject({
      frameCount,
      peakWorkerPendingFrames: SPATIAL_WORKER_PENDING_WATERMARKS.high,
    });
  });

  test("does not allocate another RGBA buffer until pending work reaches the low watermark", async () => {
    const worker = new FakeWorker();
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    let bufferBuilds = 0;
    const buildBuffer = () => {
      bufferBuilds += 1;
      return new ArrayBuffer(1);
    };
    const high = SPATIAL_WORKER_PENDING_WATERMARKS.high;
    const low = SPATIAL_WORKER_PENDING_WATERMARKS.low;
    const initial = Array.from({ length: high }, (_, frameIndex) =>
      workerSession.sendSpatialFrame(frameIndex, buildBuffer, {
        p1Teleport: false,
        p2Teleport: false,
        p1Airborne: false,
        p2Airborne: false,
      }),
    );
    await Promise.all(initial);
    const queued = workerSession.sendSpatialFrame(high, buildBuffer, {
      p1Teleport: false,
      p2Teleport: false,
      p1Airborne: false,
      p2Airborne: false,
    });
    const drained = workerSession.drainSpatialFrames();

    expect(bufferBuilds).toBe(high);
    expect(spatialMessages(worker)).toHaveLength(high);
    for (let pending = high - 1; pending > low; pending -= 1) {
      worker.receive({ type: "spatialFrameResult" });
      await Promise.resolve();
      expect(bufferBuilds).toBe(high);
    }

    worker.receive({ type: "spatialFrameResult" });
    await waitUntil(() => spatialMessages(worker).length === high + 1);
    expect(bufferBuilds).toBe(high + 1);

    for (let pending = low + 1; pending > 0; pending -= 1) {
      worker.receive({ type: "spatialFrameResult" });
    }
    await queued;
    await drained;
  });

  test("an aborted spatial admission is removed without posting its buffer", async () => {
    const worker = new FakeWorker();
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    const initial = Array.from(
      { length: SPATIAL_WORKER_PENDING_WATERMARKS.high },
      (_, frameIndex) =>
        workerSession.sendSpatialFrame(frameIndex, rgbaBuffer, {
          p1Teleport: false,
          p2Teleport: false,
          p1Airborne: false,
          p2Airborne: false,
        }),
    );
    await Promise.all(initial);
    const controller = new AbortController();
    const reason = new Error("cancel queued spatial send");
    let queuedBufferBuilt = false;
    const queued = workerSession.sendSpatialFrame(
      initial.length,
      () => {
        queuedBufferBuilt = true;
        return new ArrayBuffer(1);
      },
      {
        p1Teleport: false,
        p2Teleport: false,
        p1Airborne: false,
        p2Airborne: false,
      },
      controller.signal,
    );

    controller.abort(reason);

    await expect(queued).rejects.toBe(reason);
    expect(queuedBufferBuilt).toBeFalse();
    expect(spatialMessages(worker)).toHaveLength(initial.length);
    workerSession.terminate(reason);
  });

  test("releases a reserved admission when RGBA creation fails", async () => {
    const worker = new FakeWorker();
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    const reason = new Error("getImageData failed");

    await expect(
      workerSession.sendSpatialFrame(
        0,
        () => {
          throw reason;
        },
        {
          p1Teleport: false,
          p2Teleport: false,
          p1Airborne: false,
          p2Airborne: false,
        },
      ),
    ).rejects.toBe(reason);
    await expect(workerSession.drainSpatialFrames()).resolves.toBeUndefined();
    expect(spatialMessages(worker)).toHaveLength(0);

    await workerSession.sendSpatialFrame(1, rgbaBuffer, {
      p1Teleport: false,
      p2Teleport: false,
      p1Airborne: false,
      p2Airborne: false,
    });
    worker.receive({ type: "spatialFrameResult" });
    await expect(workerSession.drainSpatialFrames()).resolves.toBeUndefined();
  });

  test("Worker errors reject queued spatial admission and drain waiters", async () => {
    const worker = new FakeWorker();
    const callbackErrors: unknown[] = [];
    const workerSession = new AnalyzerWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: (error) => callbackErrors.push(error),
    });
    const initial = Array.from(
      { length: SPATIAL_WORKER_PENDING_WATERMARKS.high },
      (_, frameIndex) =>
        workerSession.sendSpatialFrame(frameIndex, rgbaBuffer, {
          p1Teleport: false,
          p2Teleport: false,
          p1Airborne: false,
          p2Airborne: false,
        }),
    );
    await Promise.all(initial);
    const queued = workerSession.sendSpatialFrame(initial.length, rgbaBuffer, {
      p1Teleport: false,
      p2Teleport: false,
      p1Airborne: false,
      p2Airborne: false,
    });
    const drain = workerSession.drainSpatialFrames();

    worker.receive({ type: "error", message: "spatial worker failed" });

    expect(callbackErrors).toHaveLength(1);
    await expectRejectedWith([queued, drain], callbackErrors[0]);
    expect(worker.terminateCount).toBe(1);
  });
});

describe("MeterWorkerSession", () => {
  test("termination rejects ready, frame-drain, and timeline waits", async () => {
    const readyWorker = new FakeWorker();
    const readySession = new MeterWorkerSession(readyWorker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    const beforeReady = readySession.sendFrame({
      slot: 0,
      frameIndex: 0,
      meterBuf: new ArrayBuffer(1),
    });
    const reason = new Error("meter cancelled");
    readySession.terminate(reason);

    await expect(beforeReady).rejects.toBe(reason);

    const worker = new FakeWorker();
    const workerSession = new MeterWorkerSession(worker.asWorker(), {
      onFrameResult: () => {},
      onError: () => {},
    });
    worker.receive({ type: "ready" });
    await workerSession.sendFrame({
      slot: 0,
      frameIndex: 0,
      meterBuf: new ArrayBuffer(1),
    });
    const frameDrain = workerSession.drainFrames();
    const timeline = workerSession.finish();

    workerSession.terminate(reason);

    await expectRejectedWith([frameDrain, timeline], reason);
    expect(worker.terminateCount).toBe(1);
    await expect(workerSession.finish()).rejects.toBe(reason);
  });
});

async function expectRejectedWith(
  promises: readonly Promise<unknown>[],
  reason: unknown,
): Promise<void> {
  const results = await Promise.allSettled(promises);
  expect(results).toHaveLength(promises.length);
  for (const result of results) {
    expect(result.status).toBe("rejected");
    if (result.status === "rejected") expect(result.reason).toBe(reason);
  }
}

function spatialMessages(worker: FakeWorker): unknown[] {
  return worker.messages.filter(
    (message) => messageType(message) === "spatialFrame",
  );
}

function messageType(message: unknown): unknown {
  return typeof message === "object" && message !== null
    ? Reflect.get(message, "type")
    : undefined;
}

async function waitUntil(predicate: () => boolean): Promise<void> {
  for (let attempt = 0; attempt < 100; attempt += 1) {
    if (predicate()) return;
    await Promise.resolve();
  }
  throw new Error("timed out waiting for a fake Worker message");
}
