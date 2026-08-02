import { expect, mock, test } from "bun:test";
import type { ISOFile, Movie } from "mp4box";
import type { InspectedVideoTrack } from "../../domain/video-preflight.js";
import { Mp4VideoSource } from "./mp4-video-source.js";

test("Mp4VideoSource starts extraction before asynchronous decoder setup finishes", async () => {
  const calls: string[] = [];
  let resolveTrack!: () => void;
  const trackReady = new Promise<void>((resolve) => {
    resolveTrack = resolve;
  });
  const movie = {
    videoTracks: [
      {
        id: 1,
        nb_samples: 0,
        codec: "avc1.42c028",
        video: { width: 1920, height: 1080 },
      },
    ],
  } as unknown as Movie;
  const file = fakeFile(movie, calls);
  const onSamples = mock(() => calls.push("samples"));
  const source = new Mp4VideoSource(
    new ArrayBuffer(0),
    {
      onTrack: async () => {
        calls.push("track");
        await trackReady;
      },
      onSamples,
      onError: (error) => {
        throw error;
      },
    },
    undefined,
    file,
  );

  source.start();

  expect(calls).toEqual(["track", "options", "start", "flush"]);
  expect(onSamples).not.toHaveBeenCalled();

  resolveTrack();
  await trackReady;
  await Promise.resolve();

  expect(calls).toEqual(["track", "options", "start", "flush", "samples"]);
  expect(onSamples).toHaveBeenCalledTimes(1);
});

test("Mp4VideoSource stops extraction and drops samples while decoder setup is pending", async () => {
  const calls: string[] = [];
  let resolveTrack!: () => void;
  const trackReady = new Promise<void>((resolve) => {
    resolveTrack = resolve;
  });
  const movie = {
    videoTracks: [
      {
        id: 1,
        nb_samples: 1,
        codec: "avc1.42c028",
        video: { width: 1920, height: 1080 },
      },
    ],
  } as unknown as Movie;
  const onSamples = mock(() => calls.push("samples"));
  const source = new Mp4VideoSource(
    new ArrayBuffer(0),
    {
      onTrack: async () => {
        calls.push("track");
        await trackReady;
      },
      onSamples,
      onError: (error) => {
        throw error;
      },
    },
    undefined,
    fakeFile(movie, calls),
  );

  source.start();
  source.stop();
  source.stop();
  resolveTrack();
  await trackReady;
  await Promise.resolve();

  expect(calls).toContain("stop");
  expect(calls.filter((call) => call === "stop")).toHaveLength(1);
  expect(onSamples).not.toHaveBeenCalled();
});

test("Mp4VideoSource reuses the validated track and rejects a changed demux identity", async () => {
  const calls: string[] = [];
  const description = new Uint8Array([1, 2, 3]);
  const validated: InspectedVideoTrack = {
    trackId: 1,
    codec: "avc1.42c028",
    codedWidth: 1920,
    codedHeight: 1080,
    displayWidth: 1920,
    displayHeight: 1080,
    rotation: 0,
    framesPerSecond: 60,
    constantFrameRate: true,
    totalSamples: 1,
    timescale: 60_000,
    duration: 1000,
    decoderConfig: {
      codec: "avc1.42c028",
      codedWidth: 1920,
      codedHeight: 1080,
      description,
    },
    codecConfig: {
      codec: "avc1.42c028",
      width: 1920,
      height: 1080,
      description,
    },
  };
  const baseTrack = {
    id: 1,
    nb_samples: 1,
    codec: "avc1.42c028",
    video: { width: 1920, height: 1080 },
    timescale: 60_000,
    duration: 1000,
  };
  const movie = { videoTracks: [baseTrack] } as unknown as Movie;
  const onTrack = mock(async () => undefined);
  const source = new Mp4VideoSource(
    new ArrayBuffer(0),
    {
      onTrack,
      onSamples: () => undefined,
      onError: (error) => {
        throw error;
      },
    },
    validated,
    fakeFile(movie, calls),
  );

  source.start();
  await Promise.resolve();
  expect(onTrack).toHaveBeenCalledWith({
    totalSamples: 1,
    decoderConfig: validated.decoderConfig,
    codecConfig: validated.codecConfig,
  });

  const errors: unknown[] = [];
  const changed = new Mp4VideoSource(
    new ArrayBuffer(0),
    {
      onTrack: async () => undefined,
      onSamples: () => undefined,
      onError: (error) => errors.push(error),
    },
    validated,
    fakeFile(
      {
        videoTracks: [{ ...baseTrack, codec: "hvc1.changed" }],
      } as unknown as Movie,
      [],
    ),
  );
  changed.start();
  await Promise.resolve();
  expect(errors).toHaveLength(1);
  expect(errors[0]).toBeInstanceOf(Error);
  expect(String(errors[0])).toContain("事前確認時と一致しません");
});

function fakeFile(movie: Movie, calls: string[]): ISOFile {
  const file = {
    onReady: undefined as ((info: Movie) => void) | undefined,
    onSamples: undefined as
      | ((id: number, user: unknown, samples: never[]) => void)
      | undefined,
    onError: undefined,
    appendBuffer() {
      this.onReady?.(movie);
    },
    flush() {
      calls.push("flush");
    },
    setExtractionOptions() {
      calls.push("options");
    },
    start() {
      calls.push("start");
      this.onSamples?.(1, undefined, []);
    },
    stop() {
      calls.push("stop");
    },
    getTrackById() {
      return undefined;
    },
  };
  return file as unknown as ISOFile;
}
