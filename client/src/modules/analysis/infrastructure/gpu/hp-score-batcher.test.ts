import { describe, expect, test } from "bun:test";
import {
  HP_SCORE_BATCH,
  type HpScoreBackend,
  HpScoreBatcher,
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

  count(frames: number): Promise<Uint32Array> {
    this.batches.push(frames);
    const marks = this.layers.slice(-frames).map((entry) => entry.mark);
    const values = Uint32Array.from(
      marks.flatMap((mark) => [mark, 100, mark * 2, 200]),
    );
    return new Promise((resolve) => {
      this.#resolvers.push(() => resolve(values));
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
  test("keeps every frame's values at its own place", async () => {
    const backend = new RecordingBackend();
    const batcher = new HpScoreBatcher(backend);

    const pushes: Array<Promise<void>> = [];
    for (let index = 0; index < HP_SCORE_BATCH + 3; index += 1) {
      pushes.push(batcher.push(frame(index + 1), index));
    }
    backend.settleAll();
    await Promise.all(pushes);
    const finished = batcher.finish();
    backend.settleAll();
    const values = await finished;

    expect(backend.batches).toEqual([HP_SCORE_BATCH, 3]);
    expect(values.length).toBe((HP_SCORE_BATCH + 3) * 4);
    // 3 枚目のフレームの値が 3 枚目の位置にある。
    expect([...values.slice(8, 12)]).toEqual([3, 100, 6, 200]);
    // 最後のまとまりの先頭も同じ。
    const last = HP_SCORE_BATCH * 4;
    expect([...values.slice(last, last + 4)]).toEqual([9, 100, 18, 200]);
  });

  test("refuses frames that arrive out of order", async () => {
    const backend = new RecordingBackend();
    const batcher = new HpScoreBatcher(backend);

    await batcher.push(frame(1), 0);

    expect(batcher.push(frame(2), 5)).rejects.toThrow("in order");
  });

  test("has nothing to count when no frame arrived", async () => {
    const backend = new RecordingBackend();

    const values = await new HpScoreBatcher(backend).finish();

    expect(values.length).toBe(0);
    expect(backend.batches).toEqual([]);
  });
});
