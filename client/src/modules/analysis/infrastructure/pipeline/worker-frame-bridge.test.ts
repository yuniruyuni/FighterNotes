import { describe, expect, test } from "bun:test";
import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  SUPER_BAND_HEIGHT,
} from "../frame-extraction/layout.js";
import type { StripPixels } from "../frame-extraction/strip-extractor.js";
import { WorkerFrameBridge } from "./worker-frame-bridge.js";

const pixels: StripPixels = {
  hud: new Uint8ClampedArray(ANALYSIS_STRIPS.hud.byteLength),
  super: new Uint8ClampedArray(ANALYSIS_WIDTH * SUPER_BAND_HEIGHT * 4),
  meter: new Uint8ClampedArray(ANALYSIS_STRIPS.meter.byteLength),
  input: new Uint8ClampedArray(ANALYSIS_STRIPS.input.byteLength),
};

describe("WorkerFrameBridge", () => {
  test("completes a frame only after every worker returns its buffers", async () => {
    const meterMessages: Array<{
      readonly slot: number;
      readonly meterBuf: ArrayBuffer;
    }> = [];
    const attackMessages: Array<{
      readonly slot: number;
      readonly meterBuf: ArrayBuffer;
    }> = [];
    const resultMessages: Array<{
      readonly slot: number;
      readonly hudBuf: ArrayBuffer;
      readonly superBuf: ArrayBuffer;
      readonly inputBuf: ArrayBuffer;
    }> = [];
    let completed = 0;
    const bridge = new WorkerFrameBridge({
      sendMeter: async (message) => {
        meterMessages.push(message);
      },
      sendAttack: async (message) => {
        attackMessages.push(message);
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
    const attack = attackMessages[0];
    const result = resultMessages[0];
    if (!meter || !attack || !result) {
      throw new Error("worker message was not sent");
    }
    expect(attack.meterBuf).not.toBe(meter.meterBuf);

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
    expect(bridge.completedFrames).toBe(0);

    bridge.acceptAttack({
      ...attack,
      tCopy: 1,
      tAttack: 11,
    });
    expect(bridge.completedFrames).toBe(1);
    expect(completed).toBe(1);
    expect(bridge.timing).toEqual({
      tCopy: 8,
      tMeter: 7,
      tAttack: 11,
      tHud: 3,
    });
  });

  test("aborts a frame waiting for a transfer buffer", async () => {
    const controller = new AbortController();
    const bridge = new WorkerFrameBridge({
      sendMeter: async () => {},
      sendAttack: async () => {},
      sendResult: async () => {},
      totalSamples: () => 3,
      drawTime: () => 0,
      onProgress: () => {},
      onFrameCompleted: () => {},
      signal: controller.signal,
    });

    await bridge.send(0, pixels);
    await bridge.send(1, pixels);
    const waiting = bridge.send(2, pixels);
    controller.abort(new Error("利用者が中止"));

    await expect(waiting).rejects.toThrow("利用者が中止");
  });
});
