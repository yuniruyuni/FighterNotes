import { describe, expect, test } from "bun:test";
import type { AnalyzerWorkerDone } from "../worker-bridge/protocol.js";
import {
  type AnalysisCompletionSession,
  completeAnalysis,
} from "./complete-analysis.js";

function session(
  windows: Awaited<ReturnType<AnalysisCompletionSession["firstPass"]>> = [],
): AnalysisCompletionSession & { finishCount: number } {
  const value = {
    finishCount: 0,
    firstPass: async () => windows,
    resetSpatialWindow: async () => {},
    sendSpatialFrame: async () => {},
    drainSpatialFrames: async () => {},
    finishSpatialPass: () => {
      value.finishCount += 1;
    },
    result: async (): Promise<AnalyzerWorkerDone> => ({
      type: "done",
      report: "{}",
      timeline: "{}",
      features: "[]",
      regressionEvents: "{}",
    }),
  };
  return value;
}

function options(workerSession: AnalysisCompletionSession) {
  return {
    session: workerSession,
    analysisContext: { ownSide: "p1" as const, p1: {}, p2: {} },
    videoFile: new Blob(),
    sampleData: [],
    frameToSampleIdx: [],
    frameTimestamps: [0],
    getCodecConfig: () => null,
    onProgress: () => {},
    signal: new AbortController().signal,
  };
}

describe("completeAnalysis", () => {
  test("finishes the spatial pass and builds the result without candidate windows", async () => {
    const workerSession = session();
    const result = await completeAnalysis(options(workerSession));

    expect(workerSession.finishCount).toBe(1);
    expect(result.frameCount).toBe(1);
    expect(result.hpFeatures).toEqual([]);
  });

  test("requires a codec configuration before a spatial re-decode", async () => {
    const workerSession = session([
      {
        start_frame: 1,
        end_frame: 2,
        teleport_hints: [],
        airborne_hints: [],
        contact_hints: [],
        certain_side_hints: [],
      },
    ]);

    expect(completeAnalysis(options(workerSession))).rejects.toThrow(
      "codec設定がありません",
    );
    expect(workerSession.finishCount).toBe(0);
  });

  test("does not finish after cancellation", async () => {
    const workerSession = session();
    const controller = new AbortController();
    controller.abort(new Error("cancelled"));

    expect(
      completeAnalysis({
        ...options(workerSession),
        signal: controller.signal,
      }),
    ).rejects.toThrow("cancelled");
    expect(workerSession.finishCount).toBe(0);
  });

  test("settles when cancellation interrupts the first-pass wait", async () => {
    const workerSession = {
      ...session(),
      firstPass: () => new Promise<never>(() => {}),
    };
    const controller = new AbortController();
    const reason = new Error("cancelled during first pass");
    const completion = completeAnalysis({
      ...options(workerSession),
      signal: controller.signal,
    });

    controller.abort(reason);

    expect(await completion.catch((error) => error)).toBe(reason);
    expect(workerSession.finishCount).toBe(0);
  });

  test("settles when cancellation interrupts the final-result wait", async () => {
    let resultStarted!: () => void;
    const waitingForResult = new Promise<void>((resolve) => {
      resultStarted = resolve;
    });
    const baseSession = session();
    const workerSession = {
      ...baseSession,
      result: () => {
        resultStarted();
        return new Promise<never>(() => {});
      },
    };
    const controller = new AbortController();
    const reason = new Error("cancelled during result");
    const completion = completeAnalysis({
      ...options(workerSession),
      signal: controller.signal,
    });
    await waitingForResult;

    controller.abort(reason);

    expect(await completion.catch((error) => error)).toBe(reason);
    expect(baseSession.finishCount).toBe(1);
  });
});
