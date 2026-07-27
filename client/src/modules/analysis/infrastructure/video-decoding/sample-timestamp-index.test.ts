import { describe, expect, test } from "bun:test";
import { mp4TimestampUs, SampleTimestampIndex } from "./sample-timestamp-index";

describe("mp4TimestampUs", () => {
  test("WebCodecsへ渡す前に小数マイクロ秒を切り捨てる", () => {
    expect(mp4TimestampUs(1001, 30_000)).toBe(33_366);
    expect(mp4TimestampUs(2002, 30_000)).toBe(66_733);
  });

  test("不正なtimescaleを拒否する", () => {
    expect(() => mp4TimestampUs(1, 0)).toThrow(RangeError);
  });
});

describe("SampleTimestampIndex", () => {
  test("Bフレームの表示順をデコード順sampleへ対応付ける", () => {
    const index = new SampleTimestampIndex();
    const decodeOrder = [
      mp4TimestampUs(0, 30_000),
      mp4TimestampUs(2002, 30_000),
      mp4TimestampUs(1001, 30_000),
    ];
    decodeOrder.forEach((timestampUs, sampleIndex) => {
      index.add(timestampUs, sampleIndex);
    });

    const presentationOrder = [decodeOrder[0], decodeOrder[2], decodeOrder[1]];
    expect(
      presentationOrder.map((timestampUs) => index.resolve(timestampUs)),
    ).toEqual([0, 2, 1]);
    expect(index.resolve(999_999)).toBe(-1);
  });
});
