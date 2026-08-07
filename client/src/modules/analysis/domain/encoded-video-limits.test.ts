import { describe, expect, test } from "bun:test";
import {
  DEMUX_METADATA_CHUNK_BYTES,
  ENCODED_QUEUE_BYTE_LOW_WATERMARK,
  ENCODED_QUEUE_SAMPLE_LOW_WATERMARK,
  MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES,
  MAX_DEMUX_METADATA_BYTES,
  MAX_DEMUX_METADATA_MP4_BUFFER_BYTES,
  MAX_DEMUX_MP4_SAMPLE_BYTES,
  MAX_DEMUX_RETAINED_BYTES,
  MAX_ENCODED_BATCH_BYTES,
  MAX_ENCODED_BATCH_SAMPLES,
  MAX_ENCODED_QUEUE_BYTES,
  MAX_ENCODED_QUEUE_SAMPLES,
  MAX_ENCODED_SAMPLE_BYTES,
} from "./encoded-video-limits.js";

const MiB = 1024 * 1024;

/**
 * これらの上限は docs/architecture.md が「demuxが論理的に所有できるbuffer量」
 * として公開している契約そのもので、実装の都合で動かしてよい値ではない。
 * 算術式のまま置いておくと桁を1つ間違えても誰も気付かないため、
 * 期待値を明示して固定する。変更するときは文書も同時に更新する。
 */
describe("encoded video limits", () => {
  test("公開しているbyte budgetを固定する", () => {
    expect(DEMUX_METADATA_CHUNK_BYTES).toBe(1 * MiB);
    expect(MAX_ENCODED_SAMPLE_BYTES).toBe(16 * MiB);
    expect(MAX_ENCODED_BATCH_BYTES).toBe(16 * MiB);
    expect(MAX_ENCODED_QUEUE_BYTES).toBe(32 * MiB);
    expect(ENCODED_QUEUE_BYTE_LOW_WATERMARK).toBe(16 * MiB);
    expect(MAX_DEMUX_METADATA_BYTES).toBe(32 * MiB);
    expect(MAX_DEMUX_METADATA_MP4_BUFFER_BYTES).toBe(32 * MiB);
    expect(MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES).toBe(48 * MiB);
    expect(MAX_DEMUX_MP4_SAMPLE_BYTES).toBe(16 * MiB);
    expect(MAX_DEMUX_RETAINED_BYTES).toBe(96 * MiB);
  });

  test("公開しているsample件数の上限を固定する", () => {
    expect(MAX_ENCODED_BATCH_SAMPLES).toBe(8);
    expect(MAX_ENCODED_QUEUE_SAMPLES).toBe(16);
    expect(ENCODED_QUEUE_SAMPLE_LOW_WATERMARK).toBe(8);
  });

  /**
   * low watermark は供給を再開する閾値なので、上限と等しいか超えると
   * backpressure が意味を失い、上限を超えて滞留し続ける。
   */
  test("low watermarkが上限を下回る", () => {
    expect(ENCODED_QUEUE_SAMPLE_LOW_WATERMARK).toBeLessThan(
      MAX_ENCODED_QUEUE_SAMPLES,
    );
    expect(ENCODED_QUEUE_BYTE_LOW_WATERMARK).toBeLessThan(
      MAX_ENCODED_QUEUE_BYTES,
    );
  });

  /**
   * 1個のsampleがbatch上限を超えると、そのsampleだけで batch を組めず
   * 供給が永久に止まる。
   */
  test("1 sampleの上限がbatch上限を超えない", () => {
    expect(MAX_ENCODED_SAMPLE_BYTES).toBeLessThanOrEqual(
      MAX_ENCODED_BATCH_BYTES,
    );
    expect(MAX_ENCODED_BATCH_BYTES).toBeLessThanOrEqual(
      MAX_ENCODED_QUEUE_BYTES,
    );
  });

  /**
   * 合計上限は、同時に所有しうる各内訳の合計を下回ってはならない。
   * 下回ると、正常な最大構成のまま停止条件に当たる。
   */
  test("合計上限が内訳の同時所有量を下回らない", () => {
    expect(MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES).toBe(
      MAX_DEMUX_METADATA_BYTES + MAX_ENCODED_BATCH_BYTES,
    );
    expect(MAX_DEMUX_RETAINED_BYTES).toBeGreaterThanOrEqual(
      MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES + MAX_DEMUX_MP4_SAMPLE_BYTES,
    );
  });
});
