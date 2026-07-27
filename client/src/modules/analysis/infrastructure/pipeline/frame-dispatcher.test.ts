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
      readFrame: () => pixels,
    };
    const dispatcher = new FrameDispatcher({
      extractor,
      maxPendingBitmaps: 4,
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

  test("uses synchronous extraction when the bitmap limit is reached", async () => {
    const modes: string[] = [];
    const extractor: StripFrameExtractor<FakeFrame, number> = {
      createBitmaps: (item) => item.id,
      readBitmaps: async () => {
        modes.push("async");
        return pixels;
      },
      readFrame: () => {
        modes.push("sync");
        return pixels;
      },
    };
    const dispatcher = new FrameDispatcher({
      extractor,
      maxPendingBitmaps: 1,
      sendFrame: async () => {},
      onError: (error) => {
        throw error;
      },
      now: () => 0,
    });

    dispatcher.dispatch(frame(1), 0);
    dispatcher.dispatch(frame(2), 1);
    await dispatcher.drain();

    expect(modes).toEqual(["sync", "async"]);
  });
});
