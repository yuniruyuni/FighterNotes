import { describe, expect, test } from "bun:test";
import { DecodePump, type DecodeQueue } from "./decode-pump.js";

class FakeDecoder implements DecodeQueue<number> {
  state = "configured";
  decodeQueueSize = 0;
  readonly decoded: number[] = [];
  error: Error | null = null;

  decode(sample: number): void {
    if (this.error) throw this.error;
    this.decoded.push(sample);
    this.decodeQueueSize += 1;
  }
}

function setup() {
  const events = { flushes: 0, errors: [] as unknown[] };
  const pump = new DecodePump<number>({
    maxDecodeQueue: 2,
    maxInflightFrames: 3,
    onReadyToFlush: () => {
      events.flushes += 1;
    },
    onError: (error) => events.errors.push(error),
  });
  return { pump, events };
}

describe("DecodePump", () => {
  test("honors decoder and worker backpressure", () => {
    const { pump } = setup();
    const decoder = new FakeDecoder();
    pump.setTotalSamples(4);
    for (const sample of [1, 2, 3, 4]) pump.enqueue(sample);

    pump.pump(decoder, 3);
    expect(decoder.decoded).toEqual([]);

    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2]);

    decoder.decodeQueueSize = 0;
    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2, 3, 4]);
  });

  test("starts flushing exactly once after every sample is fed", () => {
    const { pump, events } = setup();
    const decoder = new FakeDecoder();
    pump.setTotalSamples(1);
    pump.enqueue(1);

    pump.pump(decoder, 0);
    decoder.decodeQueueSize = 0;
    pump.pump(decoder, 0);

    expect(events.flushes).toBe(1);
  });

  test("reports decode errors without starting the flush", () => {
    const { pump, events } = setup();
    const decoder = new FakeDecoder();
    decoder.error = new Error("decode failed");
    pump.setTotalSamples(1);
    pump.enqueue(1);

    pump.pump(decoder, 0);

    expect(events.errors).toEqual([decoder.error]);
    expect(events.flushes).toBe(0);
  });
});
