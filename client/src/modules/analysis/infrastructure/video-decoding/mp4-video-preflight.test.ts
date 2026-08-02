import { describe, expect, test } from "bun:test";
import type { ISOFile, Movie, MP4BoxBuffer, Sample } from "mp4box";
import {
  inspectMovie,
  inspectMp4VideoFile,
  Mp4InspectionError,
  rotationFromTrackMatrix,
  summarizeFrameTiming,
} from "./mp4-video-preflight.js";

const IDENTITY = [65536, 0, 0, 0, 65536, 0, 0, 0, 1073741824];

function movie(overrides: Partial<Movie["videoTracks"][number]> = {}): Movie {
  const videoTrack = {
    id: 1,
    codec: "avc1.640028",
    video: { width: 1920, height: 1080 },
    track_width: 1920,
    track_height: 1080,
    matrix: IDENTITY,
    nb_samples: 4,
    timescale: 60_000,
    duration: 4004,
    ...overrides,
  } as Movie["videoTracks"][number];
  return {
    brands: ["isom", "mp42"],
    isFragmented: false,
    videoTracks: [videoTrack],
  } as Movie;
}

function isoFile(cts: readonly number[]): ISOFile {
  return {
    getTrackSamplesInfo: () =>
      cts.map((timestamp) => ({ cts: timestamp })) as Sample[],
    getTrackById: () => undefined,
  } as unknown as ISOFile;
}

describe("MP4 video metadata parser", () => {
  test("coded/display dimensionsとcanonical rotationをtrack matrixから読む", () => {
    const inspected = inspectMovie(
      isoFile([0, 1001, 2002, 3003]),
      movie(),
      8192,
    );
    expect(inspected).toMatchObject({
      container: "mp4",
      fragmented: false,
      metadataBytesRead: 8192,
      track: {
        codedWidth: 1920,
        codedHeight: 1080,
        displayWidth: 1920,
        displayHeight: 1080,
        rotation: 0,
        framesPerSecond: 59.94005994005994,
        constantFrameRate: true,
      },
    });

    const rotated = inspectMovie(
      isoFile([0, 1001, 2002, 3003]),
      movie({
        video: { width: 1080, height: 1920 },
        track_width: 1080,
        track_height: 1920,
        matrix: [0, 65536, 0, -65536, 0, 0, 0, 0, 1073741824],
      }),
      4096,
    );
    expect(rotated.track).toMatchObject({
      codedWidth: 1080,
      codedHeight: 1920,
      displayWidth: 1920,
      displayHeight: 1080,
      rotation: 90,
    });
    expect(
      rotationFromTrackMatrix([-65536, 0, 0, 0, -65536, 0, 0, 0, 1073741824]),
    ).toBe(180);
    expect(
      rotationFromTrackMatrix([0, -65536, 0, 65536, 0, 0, 0, 0, 1073741824]),
    ).toBe(270);
    expect(
      rotationFromTrackMatrix([32768, 0, 0, 0, 65536, 0, 0, 0, 1073741824]),
    ).toBeNull();
    expect(
      rotationFromTrackMatrix([
        65536,
        0,
        1 << 20,
        0,
        65536,
        0,
        0,
        0,
        1073741824,
      ]),
    ).toBeNull();
    expect(rotationFromTrackMatrix(undefined)).toBeNull();
  });

  test("CTS差分からinteger timebaseのCFR揺れとVFRを区別する", () => {
    expect(
      summarizeFrameTiming(
        [{ cts: 0 }, { cts: 1501 }, { cts: 3003 }, { cts: 4504 }],
        90_000,
        4,
      ),
    ).toMatchObject({ constantFrameRate: true });
    expect(
      summarizeFrameTiming(
        [{ cts: 0 }, { cts: 1000 }, { cts: 2000 }, { cts: 4000 }],
        60_000,
        4,
      ),
    ).toMatchObject({ constantFrameRate: false, framesPerSecond: 45 });
    for (const [samples, timescale, count] of [
      [[{ cts: 0 }], 60_000, 1],
      [[{ cts: 0 }, { cts: 1000 }], 0, 2],
      [[{ cts: 0 }], 60_000, 2],
      [[{ cts: 0 }, { cts: 0 }], 60_000, 2],
    ] as const) {
      const result = summarizeFrameTiming(samples, timescale, count);
      expect(result.constantFrameRate).toBe(false);
      expect(result.framesPerSecond).toBeNaN();
    }
  });

  test("progressive parserの推奨offsetへ飛び、mdat全体を読まずmoovを得る", async () => {
    const offsets: number[] = [];
    let stopped = 0;
    const info = {
      brands: ["isom"],
      isFragmented: false,
      videoTracks: [],
    } as unknown as Movie;
    const parser = {
      onReady: undefined as ((movie: Movie) => void) | undefined,
      onError: undefined,
      appendBuffer(
        this: { onReady?: (movie: Movie) => void },
        buffer: MP4BoxBuffer,
      ) {
        offsets.push(buffer.fileStart);
        if (buffer.fileStart === 16) this.onReady?.(info);
        return buffer.fileStart === 0 ? 16 : 20;
      },
      flush() {},
      stop() {
        stopped += 1;
      },
    } as unknown as ISOFile;
    const result = await inspectMp4VideoFile(
      new File([new Uint8Array(24)], "replay.mp4", { type: "video/mp4" }),
      new AbortController().signal,
      { createIsoFile: () => parser, chunkBytes: 4 },
    );

    expect(offsets).toEqual([0, 16]);
    expect(result.metadataBytesRead).toBe(8);
    expect(result.track).toBeNull();
    expect(stopped).toBe(1);
  });

  test("明らかな非MP4と中止をparser/Worker起動前に拒否する", async () => {
    let created = 0;
    await expect(
      inspectMp4VideoFile(
        new File(["webm"], "replay.webm", { type: "video/webm" }),
        new AbortController().signal,
        {
          createIsoFile: () => {
            created += 1;
            return {} as ISOFile;
          },
        },
      ),
    ).rejects.toMatchObject({ code: "non_mp4" });
    expect(created).toBe(0);

    const controller = new AbortController();
    controller.abort(new Error("stale selection"));
    await expect(
      inspectMp4VideoFile(
        new File(["mp4"], "replay.mp4", { type: "video/mp4" }),
        controller.signal,
        {
          createIsoFile: () => ({ stop() {} }) as unknown as ISOFile,
          chunkBytes: 1,
        },
      ),
    ).rejects.toThrow("stale selection");
    expect(new Mp4InspectionError("invalid_mp4", "broken").name).toBe(
      "Mp4InspectionError",
    );
  });
});
