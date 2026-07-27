import { expect, mock, test } from "bun:test";
import type { ISOFile, Movie } from "mp4box";
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
    getTrackById() {
      return undefined;
    },
  };
  return file as unknown as ISOFile;
}
