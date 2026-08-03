import { describe, expect, test } from "bun:test";
import { BlobRangeReader } from "./blob-range-reader.js";

describe("BlobRangeReader", () => {
  test("reads only the requested range and reports bounded peak bytes", async () => {
    const source = new Blob([new Uint8Array([0, 1, 2, 3, 4, 5])]);
    const reader = new BlobRangeReader(source);

    expect(new Uint8Array(await reader.read(2, 3))).toEqual(
      new Uint8Array([2, 3, 4]),
    );
    expect(new Uint8Array(await reader.read(0, 1))).toEqual(
      new Uint8Array([0]),
    );
    expect(reader.statistics).toEqual({
      readCount: 2,
      totalBytesRead: 4,
      peakReadBytes: 3,
    });
  });

  test("does not publish a completed read or begin another read after abort", async () => {
    const controller = new AbortController();
    let finishRead!: (buffer: ArrayBuffer) => void;
    let calls = 0;
    const reader = new BlobRangeReader(
      new Blob([new Uint8Array(8)]),
      controller.signal,
      {
        readSlice: async () => {
          calls += 1;
          return new Promise<ArrayBuffer>((resolve) => {
            finishRead = resolve;
          });
        },
      },
    );
    const reason = new Error("cancelled");
    const reading = reader.read(0, 4);

    controller.abort(reason);
    finishRead(new ArrayBuffer(4));

    expect(await reading.catch((error) => error)).toBe(reason);
    await expect(reader.read(4, 4)).rejects.toBe(reason);
    expect(calls).toBe(1);
    expect(reader.statistics).toEqual({
      readCount: 0,
      totalBytesRead: 0,
      peakReadBytes: 4,
    });
  });

  test("rejects overlapping reads and out-of-bounds ranges", async () => {
    let finishRead!: (buffer: ArrayBuffer) => void;
    const reader = new BlobRangeReader(
      new Blob([new Uint8Array(4)]),
      undefined,
      {
        readSlice: () =>
          new Promise<ArrayBuffer>((resolve) => {
            finishRead = resolve;
          }),
      },
    );
    const first = reader.read(0, 2);

    await expect(reader.read(2, 2)).rejects.toThrow("concurrent");
    finishRead(new ArrayBuffer(2));
    await first;
    await expect(reader.read(3, 2)).rejects.toThrow("outside");
  });
});
