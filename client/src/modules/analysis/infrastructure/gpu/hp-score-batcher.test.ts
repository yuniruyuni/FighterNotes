import { describe, expect, test } from "bun:test";
import {
  HP_SCORE_BATCH,
  type HpScoreBackend,
  HpScoreBatcher,
  type HudGpuBatch,
  type HudGpuResult,
} from "./hp-score-batcher.js";

/** 置かれた枚数をそのまま数え返す、順番だけを見る偽の GPU。 */
class RecordingBackend implements HpScoreBackend {
  readonly layers: Array<{ readonly layer: number; readonly mark: number }> =
    [];
  readonly batches: number[] = [];
  #resolvers: Array<() => void> = [];

  writeLayer(pixels: ArrayBuffer, layer: number): void {
    this.layers.push({ layer, mark: new Uint8Array(pixels)[0] ?? 0 });
  }

  extractLayer(): void {
    throw new Error("この試験は画素を直接置く経路だけを見る");
  }

  count(frames: number): Promise<HudGpuResult> {
    this.batches.push(frames);
    const marks = this.layers.slice(-frames).map((entry) => entry.mark);
    const values = Uint32Array.from(
      marks.flatMap((mark) => [mark, 100, mark * 2, 200]),
    );
    return new Promise((resolve) => {
      this.#resolvers.push(() =>
        resolve({ scores: values, columns: Uint32Array.from(marks) }),
      );
    });
  }

  settleAll(): void {
    const resolvers = this.#resolvers;
    this.#resolvers = [];
    for (const resolve of resolvers) resolve();
  }
}

function frame(mark: number): ArrayBuffer {
  const buffer = new ArrayBuffer(4);
  new Uint8Array(buffer)[0] = mark;
  return buffer;
}

describe("HpScoreBatcher", () => {
  test("hands each batch over with the frame it starts at", async () => {
    const backend = new RecordingBackend();
    const batches: HudGpuBatch[] = [];
    const batcher = new HpScoreBatcher(backend, (batch) => {
      batches.push(batch);
    });

    const pushes: Array<Promise<void>> = [];
    for (let index = 0; index < HP_SCORE_BATCH + 3; index += 1) {
      pushes.push(batcher.push(frame(index + 1), index));
    }
    backend.settleAll();
    await Promise.all(pushes);
    const finished = batcher.finish();
    backend.settleAll();
    await finished;

    expect(backend.batches).toEqual([HP_SCORE_BATCH, 3]);
    expect(batches.map((batch) => batch.firstFrame)).toEqual([
      0,
      HP_SCORE_BATCH,
    ]);
    // 3 枚目のフレームの値が、最初のまとまりの 3 番目にある。
    expect([...(batches[0]?.scores.slice(8, 12) ?? [])]).toEqual([
      3, 100, 6, 200,
    ]);
    // 後のまとまりは 9 枚目から始まる。
    expect([...(batches[1]?.scores.slice(0, 4) ?? [])]).toEqual([
      9, 100, 18, 200,
    ]);
  });

  test("refuses frames that arrive out of order", async () => {
    const backend = new RecordingBackend();
    const batcher = new HpScoreBatcher(backend);

    await batcher.push(frame(1), 0);

    expect(batcher.push(frame(2), 5)).rejects.toThrow("in order");
  });

  test("has nothing to count when no frame arrived", async () => {
    const backend = new RecordingBackend();
    const batches: HudGpuBatch[] = [];

    await new HpScoreBatcher(backend, (batch) => batches.push(batch)).finish();

    expect(batches).toEqual([]);
    expect(backend.batches).toEqual([]);
  });
});
