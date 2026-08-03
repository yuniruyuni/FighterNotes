import { describe, expect, test } from "bun:test";
import type { ISOFile, Movie, MP4BoxBuffer, Sample } from "mp4box";
import { validateInspectedVideo } from "../../domain/video-preflight.js";
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
      cts.map((timestamp) => ({ cts: timestamp, size: 1024 })) as Sample[],
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
        maxSampleBytes: 1024,
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

  test("B-frame CTSを表示順に並べ、59.94fpsの一貫した量子化だけをCFRにする", () => {
    expect(
      summarizeFrameTiming(
        [{ cts: 0 }, { cts: 2002 }, { cts: 1001 }, { cts: 3003 }],
        60_000,
        4,
      ),
    ).toMatchObject({
      constantFrameRate: true,
      framesPerSecond: 59.94005994005994,
    });
    expect(
      summarizeFrameTiming(
        [{ cts: 0 }, { cts: 3003 }, { cts: 1501 }, { cts: 4504 }],
        90_000,
        4,
      ),
    ).toMatchObject({ constantFrameRate: true });
    const oneDroppedFrame = Array.from({ length: 120 }, (_, cts) => ({ cts }));
    oneDroppedFrame.splice(60, 1);
    expect(summarizeFrameTiming(oneDroppedFrame, 60, 119)).toMatchObject({
      constantFrameRate: false,
      framesPerSecond: 59.49579831932773,
    });
    const ntscLookalikeDrop = Array.from({ length: 1200 }, (_, cts) => ({
      cts,
    }));
    ntscLookalikeDrop.splice(500, 1);
    expect(
      summarizeFrameTiming(ntscLookalikeDrop, 60, ntscLookalikeDrop.length),
    ).toMatchObject({ constantFrameRate: false });
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

  test("130k超のsamplesを引数spreadやdelta配列なしで集計する", () => {
    const frameCount = 130_001;
    const samples = Array.from({ length: frameCount }, (_, cts) => ({ cts }));
    expect(summarizeFrameTiming(samples, 60, frameCount)).toEqual({
      framesPerSecond: 60,
      constantFrameRate: true,
    });
  });

  test("先頭のmajor brandだけをMP4 allowlistと照合する", () => {
    const file = {} as ISOFile;
    for (const majorBrand of ["isom", "mp42", "M4V "]) {
      expect(
        inspectMovie(
          file,
          {
            brands: [majorBrand],
            isFragmented: false,
            videoTracks: [],
          } as unknown as Movie,
          0,
        ).container,
      ).toBe("mp4");
    }
    for (const brands of [
      ["mif1", "heic", "isom"],
      ["avif", "isom"],
      ["3gp6", "isom"],
      ["qt  ", "mp42"],
    ]) {
      expect(
        inspectMovie(
          file,
          {
            brands,
            isFragmented: false,
            videoTracks: [],
          } as unknown as Movie,
          0,
        ).container,
      ).toBe("other");
    }
  });

  test("CMAF/DASH major brandをfragmented MP4の理由で拒否する", () => {
    const source = new File(["video"], "replay.mp4", { type: "video/mp4" });
    for (const majorBrand of ["cmfc", "cmfs", "cmfl", "cmff", "dash"]) {
      const inspected = inspectMovie(
        isoFile([0, 1001, 2002, 3003]),
        {
          ...movie(),
          brands: [majorBrand],
          isFragmented: true,
        } as Movie,
        4096,
      );
      expect(inspected).toMatchObject({
        container: "mp4",
        fragmented: true,
      });
      expect(validateInspectedVideo(source, inspected)).toMatchObject({
        status: "invalid",
        code: "fragmented_mp4",
      });
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

  test("metadata budgetを使い切ったMP4を専用理由で拒否する", async () => {
    let stopped = 0;
    const parser = {
      appendBuffer(buffer: MP4BoxBuffer) {
        return buffer.fileStart + buffer.byteLength;
      },
      flush() {},
      stop() {
        stopped += 1;
      },
    } as unknown as ISOFile;

    await expect(
      inspectMp4VideoFile(
        new File([Uint8Array.of(0, 0, 0)], "replay.mp4", {
          type: "video/mp4",
        }),
        new AbortController().signal,
        {
          createIsoFile: () => parser,
          chunkBytes: 1,
          maxMetadataBytes: 2,
        },
      ),
    ).rejects.toMatchObject({ code: "metadata_size" });
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
