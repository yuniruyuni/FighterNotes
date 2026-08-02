import { describe, expect, test } from "bun:test";
import type { StripPixels } from "../frame-extraction/strip-extractor.js";
import {
  FrameDispatcher,
  type StripFrameExtractor,
} from "./frame-dispatcher.js";

interface FakeFrame {
  readonly id: number;
  closes: number;
  close(): void;
}

const pixels = {} as StripPixels;

function frame(id: number): FakeFrame {
  return { id, closes: 0, close: () => {} };
}

describe("FrameDispatcher", () => {
  test("keeps asynchronous frame delivery ordered and closes every frame", async () => {
    const sent: number[] = [];
    const frames = [frame(1), frame(2)];
    for (const item of frames) {
      item.close = () => {
        item.closes += 1;
      };
    }
    const extractor: StripFrameExtractor<FakeFrame, number> = {
      createBitmaps: (item) => item.id,
      readBitmaps: async () => pixels,
    };
    const dispatcher = new FrameDispatcher({
      extractor,
      sendFrame: async (index) => {
        sent.push(index);
      },
      onError: (error) => {
        throw error;
      },
      now: () => 0,
    });

    dispatcher.dispatch(frames[0], 0);
    dispatcher.dispatch(frames[1], 1);
    await dispatcher.drain();

    expect(sent).toEqual([0, 1]);
    expect(frames.map((item) => item.closes)).toEqual([1, 1]);
  });

  test("starts asynchronous extraction for every frame in a burst", async () => {
    const created: Array<[number, number]> = [];
    const read: number[] = [];
    const extractor: StripFrameExtractor<FakeFrame, number> = {
      createBitmaps: (item, frameIndex) => {
        created.push([item.id, frameIndex]);
        return item.id;
      },
      readBitmaps: async (pending) => {
        read.push(pending);
        return pixels;
      },
    };
    const dispatcher = new FrameDispatcher({
      extractor,
      sendFrame: async () => {},
      onError: (error) => {
        throw error;
      },
      now: () => 0,
    });

    for (let id = 1; id <= 6; id += 1) {
      dispatcher.dispatch(frame(id), id - 1);
    }
    expect(created).toEqual([
      [1, 0],
      [2, 1],
      [3, 2],
      [4, 3],
      [5, 4],
      [6, 5],
    ]);

    await dispatcher.drain();

    expect(read).toEqual([1, 2, 3, 4, 5, 6]);
  });

  test("closes queued frames once and reports the first failure in order", async () => {
    const abort = new DOMException("cancelled", "AbortError");
    const laterFailure = new Error("late bitmap failure");
    const errors: unknown[] = [];
    const events: string[] = [];
    const frames = [frame(1), frame(2), frame(3)];
    for (const item of frames) {
      item.close = () => {
        item.closes += 1;
        events.push(`close:${item.id}`);
      };
    }
    const extractor: StripFrameExtractor<FakeFrame, number> = {
      createBitmaps: (item) => item.id,
      readBitmaps: async (pending) => {
        events.push(`read:${pending}`);
        if (pending === 2) throw laterFailure;
        return pixels;
      },
    };
    const dispatcher = new FrameDispatcher({
      extractor,
      sendFrame: async (index) => {
        events.push(`send:${index}`);
        throw abort;
      },
      onError: (error) => {
        errors.push(error);
        events.push("error");
      },
      now: () => 0,
    });

    dispatcher.dispatch(frames[0], 0);
    dispatcher.dispatch(frames[1], 1);
    dispatcher.dispatch(frames[2], 2);

    await expect(dispatcher.drain()).rejects.toBe(abort);
    expect(frames.map((item) => item.closes)).toEqual([1, 1, 1]);
    expect(errors).toEqual([abort]);
    expect(events).toEqual([
      "read:1",
      "send:0",
      "close:1",
      "error",
      "read:2",
      "close:2",
      "read:3",
      "close:3",
    ]);
  });
});
