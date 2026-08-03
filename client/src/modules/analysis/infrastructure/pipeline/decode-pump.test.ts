import { describe, expect, test } from "bun:test";
import {
  MAX_ENCODED_QUEUE_BYTES,
  MAX_ENCODED_SAMPLE_BYTES,
} from "../../domain/encoded-video-limits.js";
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
  const events = { flushes: 0, queueLow: 0, errors: [] as unknown[] };
  const pump = new DecodePump<number>({
    maxDecodeQueue: 2,
    maxOutstandingFrames: 3,
    maxQueuedSamples: 4,
    queuedSampleLowWatermark: 1,
    maxQueuedBytes: 400,
    queuedByteLowWatermark: 100,
    onQueueLow: () => {
      events.queueLow += 1;
    },
    onReadyToFlush: () => {
      events.flushes += 1;
    },
    onError: (error) => events.errors.push(error),
  });
  return { pump, events };
}

describe("DecodePump", () => {
  test("bounds both the decoder queue and total outstanding work", () => {
    const { pump } = setup();
    const decoder = new FakeDecoder();
    pump.setTotalSamples(4);
    for (const sample of [1, 2, 3, 4]) pump.enqueue(sample, 0);

    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2]);

    decoder.decodeQueueSize = 0;
    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2, 3]);

    decoder.decodeQueueSize = 0;
    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2, 3]);

    pump.pump(decoder, 1);
    expect(decoder.decoded).toEqual([1, 2, 3, 4]);
  });

  test("starts flushing exactly once after every sample is fed", () => {
    const { pump, events } = setup();
    const decoder = new FakeDecoder();
    pump.setTotalSamples(1);
    pump.enqueue(1, 0);

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
    pump.enqueue(1, 0);

    pump.pump(decoder, 0);

    expect(events.errors).toEqual([decoder.error]);
    expect(events.flushes).toBe(0);
  });

  test("bounds and observes the encoded queue before requesting another batch", () => {
    const { pump, events } = setup();
    const decoder = new FakeDecoder();
    decoder.state = "unconfigured";
    for (const sample of [1, 2, 3, 4]) pump.enqueue(sample, sample * 10);

    expect(() => pump.enqueue(5, 50)).toThrow("exceeded 4");
    expect(pump.statistics).toEqual({
      maxQueuedSamples: 4,
      queuedSampleLowWatermark: 1,
      maxQueuedBytes: 400,
      queuedByteLowWatermark: 100,
      peakQueuedSamples: 4,
      peakQueuedBytes: 100,
    });
    expect(events.queueLow).toBe(0);

    decoder.state = "configured";
    pump.pump(decoder, 0);
    expect(decoder.decoded).toEqual([1, 2]);
    expect(events.queueLow).toBe(0);
    decoder.decodeQueueSize = 0;
    pump.pump(decoder, 2);
    expect(decoder.decoded).toEqual([1, 2, 3, 4]);
    expect(events.queueLow).toBe(1);
  });

  test("drops queued chunks and refuses further admission after stop", () => {
    const { pump } = setup();
    const decoder = new FakeDecoder();
    pump.enqueue(1, 10);
    pump.stop();
    pump.enqueue(2, 20);
    pump.pump(decoder, 0);

    expect(decoder.decoded).toEqual([]);
    expect(pump.statistics.peakQueuedSamples).toBe(1);
  });

  test("issues one edge-triggered pull per admitted batch", () => {
    const { pump, events } = setup();
    const decoder = new FakeDecoder();

    // Frame/dequeue callbacks can race repeatedly while the previous pull is
    // reading. They must not accumulate credit without new sample admission.
    pump.pump(decoder, 0);
    pump.pump(decoder, 1);
    expect(events.queueLow).toBe(0);

    pump.enqueue(1, 0);
    pump.pump(decoder, 0);
    pump.pump(decoder, 1);
    pump.pump(decoder, 1);
    expect(events.queueLow).toBe(1);

    decoder.decodeQueueSize = 0;
    pump.enqueue(2, 0);
    pump.pump(decoder, 1);
    expect(events.queueLow).toBe(2);
  });

  test("rejects a queue of huge samples by bytes before the 16-sample limit", () => {
    const pump = new DecodePump<number>({
      maxDecodeQueue: 12,
      maxOutstandingFrames: 12,
      maxQueuedSamples: 16,
      queuedSampleLowWatermark: 8,
      maxQueuedBytes: MAX_ENCODED_QUEUE_BYTES,
      queuedByteLowWatermark: MAX_ENCODED_QUEUE_BYTES / 2,
      onQueueLow() {},
      onReadyToFlush() {},
      onError(error) {
        throw error;
      },
    });

    pump.enqueue(1, MAX_ENCODED_SAMPLE_BYTES);
    pump.enqueue(2, MAX_ENCODED_SAMPLE_BYTES);
    expect(() => pump.enqueue(3, MAX_ENCODED_SAMPLE_BYTES)).toThrow(
      `exceeded ${MAX_ENCODED_QUEUE_BYTES} bytes`,
    );
    expect(pump.statistics).toMatchObject({
      maxQueuedSamples: 16,
      queuedSampleLowWatermark: 8,
      maxQueuedBytes: MAX_ENCODED_QUEUE_BYTES,
      queuedByteLowWatermark: MAX_ENCODED_QUEUE_BYTES / 2,
      peakQueuedSamples: 2,
      peakQueuedBytes: MAX_ENCODED_QUEUE_BYTES,
    });
  });
});
