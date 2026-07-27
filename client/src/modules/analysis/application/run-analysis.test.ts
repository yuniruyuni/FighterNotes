import { describe, expect, mock, test } from "bun:test";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import type { AnalysisServices } from "./ports.js";
import { runAnalysis } from "./run-analysis.js";

describe("runAnalysis", () => {
  test("requestを解析contextへ変換し、debug用bufferを外した完了値を返す", async () => {
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const rawResult = syntheticAnalysisResult();
    const onProgress = mock(() => undefined);
    const analyze = mock(async () => rawResult);
    const capture = mock(() => undefined);
    const services: AnalysisServices = {
      engine: { readiness: () => ({ available: true }), analyze },
      debugSink: { capture },
    };

    const completed = await runAnalysis(
      {
        file,
        side: "p2",
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
      },
      onProgress,
      services,
    );

    const context = {
      ownSide: "p2" as const,
      p1: { character: "KEN" },
      p2: { character: "JURI" },
    };
    expect(analyze).toHaveBeenCalledWith(file, "p2", onProgress, context);
    expect(completed).toMatchObject({
      file,
      report: rawResult.report,
      context,
      result: { videoArrayBuffer: null },
    });
    expect(rawResult.videoArrayBuffer).toBeInstanceOf(ArrayBuffer);
    expect(capture).toHaveBeenCalledWith(completed.result);
  });
});
