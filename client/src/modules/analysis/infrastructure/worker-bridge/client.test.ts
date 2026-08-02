import { describe, expect, test } from "bun:test";
import type { AnalysisContext } from "../../domain/context.js";
import { AnalyzerWorkerSession, MeterWorkerSession } from "./client.js";
import type { AnalyzerWorkerResponse } from "./protocol.js";

const analysisContext: AnalysisContext = {
  ownSide: "p1",
  p1: {},
  p2: {},
};

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
    workerSession.sendSpatialFrame(0, new ArrayBuffer(1), {
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
      [frameDrain, firstPass, spatialReset, spatialDrain, result],
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
    workerSession.sendSpatialFrame(0, new ArrayBuffer(1), {
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
      [frameDrain, firstPass, spatialReset, spatialDrain, result],
      reason,
    );
    expect(worker.terminateCount).toBe(1);
    await expect(workerSession.drainFrames()).rejects.toBe(reason);
    await expect(workerSession.drainSpatialFrames()).rejects.toBe(reason);
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
