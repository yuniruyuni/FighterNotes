import { describe, expect, mock, test } from "bun:test";
import { syntheticAnalysisResult } from "~/test-support/analysis.js";
import type { ValidatedVideoInput } from "../domain/video-preflight.js";
import type { AnalysisServices } from "./ports.js";
import { runAnalysis } from "./run-analysis.js";

function validatedVideo(file: File): ValidatedVideoInput {
  return {
    file,
    identity: {
      name: file.name,
      size: file.size,
      lastModified: file.lastModified,
      type: file.type,
    },
    metadataBytesRead: 1024,
    track: {
      trackId: 1,
      codec: "avc1.640028",
      codedWidth: 1920,
      codedHeight: 1080,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 0,
      framesPerSecond: 60,
      constantFrameRate: true,
      totalSamples: 600,
      maxSampleBytes: 1024,
      timescale: 60_000,
      duration: 600_000,
      decoderConfig: {
        codec: "avc1.640028",
        codedWidth: 1920,
        codedHeight: 1080,
      },
      codecConfig: {
        codec: "avc1.640028",
        width: 1920,
        height: 1080,
      },
    },
  };
}

describe("runAnalysis", () => {
  test("requestを解析contextへ変換し、debug用bufferを外した完了値を返す", async () => {
    const file = new File(["video"], "replay.mp4", { type: "video/mp4" });
    const validated = validatedVideo(file);
    const rawResult = syntheticAnalysisResult();
    const onProgress = mock(() => undefined);
    const analyze = mock(async () => rawResult);
    const capture = mock(() => undefined);
    const services: AnalysisServices = {
      engine: {
        readiness: () => ({ available: true }),
        preflight: async () => ({ status: "valid", video: validated }),
        analyze,
      },
      debugSink: { capture },
    };
    const signal = new AbortController().signal;

    const completed = await runAnalysis(
      {
        file,
        validatedVideo: validated,
        side: "p2",
        ownCharacter: "JURI",
        opponentCharacter: "KEN",
      },
      onProgress,
      services,
      signal,
    );

    const context = {
      ownSide: "p2" as const,
      p1: { character: "KEN" },
      p2: { character: "JURI" },
    };
    expect(analyze).toHaveBeenCalledWith(
      file,
      validated,
      "p2",
      onProgress,
      context,
      signal,
    );
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
