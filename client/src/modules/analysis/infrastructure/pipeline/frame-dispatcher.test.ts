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
    const created: number[] = [];
    const read: number[] = [];
    const extractor: StripFrameExtractor<FakeFrame, number> = {
      createBitmaps: (item) => {
        created.push(item.id);
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
    expect(created).toEqual([1, 2, 3, 4, 5, 6]);

    await dispatcher.drain();

    expect(read).toEqual([1, 2, 3, 4, 5, 6]);
  });
});
