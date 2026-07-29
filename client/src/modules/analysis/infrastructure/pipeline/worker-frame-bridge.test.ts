import { describe, expect, test } from "bun:test";
import { ANALYSIS_STRIPS } from "../frame-extraction/layout.js";
import type { StripPixels } from "../frame-extraction/strip-extractor.js";
import { WorkerFrameBridge } from "./worker-frame-bridge.js";

const pixels: StripPixels = {
  hud: new Uint8ClampedArray(ANALYSIS_STRIPS.hud.byteLength),
  meter: new Uint8ClampedArray(ANALYSIS_STRIPS.meter.byteLength),
  input: new Uint8ClampedArray(ANALYSIS_STRIPS.input.byteLength),
};

describe("WorkerFrameBridge", () => {
  test("completes a frame only after both workers return their buffers", async () => {
    const meterMessages: Array<{
      readonly slot: number;
      readonly meterBuf: ArrayBuffer;
    }> = [];
    const resultMessages: Array<{
      readonly slot: number;
      readonly hudBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }> = [];
    let completed = 0;
    const bridge = new WorkerFrameBridge({
      sendMeter: async (message) => {
        meterMessages.push(message);
      },
      sendResult: async (message) => {
        resultMessages.push(message);
      },
      totalSamples: () => 10,
      drawTime: () => 0,
      onProgress: () => {},
      onFrameCompleted: () => {
        completed += 1;
      },
      signal: new AbortController().signal,
    });

    await bridge.send(0, pixels);
    const meter = meterMessages[0];
    const result = resultMessages[0];
    if (!meter || !result) throw new Error("worker message was not sent");

    bridge.acceptResult({
      ...result,
      tCopy: 2,
      tHud: 3,
    });
    expect(bridge.completedFrames).toBe(0);
    expect(completed).toBe(0);

    bridge.acceptMeter({
      ...meter,
      tCopy: 5,
      tMeter: 7,
    });
    expect(bridge.completedFrames).toBe(1);
    expect(completed).toBe(1);
    expect(bridge.timing).toEqual({
      tCopy: 7,
      tMeter: 7,
      tHud: 3,
    });
  });
});
