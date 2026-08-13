/** GPU へ渡す 1 まとまりの枚数。1 枚ずつ投げると往復の遅延だけで 3ms かかる。 */
export const HP_SCORE_BATCH = 8;

/** 同時に走らせる読み戻しの本数。遅延はこれで隠れる。 */
export const HP_SCORE_IN_FLIGHT = 4;

/** 1 フレームあたりの結果 (p1 一致, p1 全体, p2 一致, p2 全体)。 */
export const HP_SCORE_VALUES_PER_FRAME = 4;

/**
 * GPU へ画素を渡す先。WebGPU そのものを差し替えられるようにしてある。
 */
export interface HpScoreBackend {
  /** まとめの中の `layer` 枚目として画素を置く。 */
  writeLayer(pixels: ArrayBuffer, layer: number): void;
  /** 置いた `frames` 枚を数え、フレームごとの値を順に返す。 */
  count(frames: number): Promise<Uint32Array>;
}

/**
 * HP スコアの画素数えを、まとめて GPU へ流す。
 *
 * 数えた結果はフレームの順番に依存しないので、解析の最後にまとめて渡せる。
 * GPU の往復の遅延を解析の途中に持ち込まずに済む。
 */
export class HpScoreBatcher {
  readonly #backend: HpScoreBackend;
  readonly #results: number[] = [];
  readonly #pending: Array<Promise<void>> = [];
  #filled = 0;
  #firstFrameOfBatch = 0;
  #nextFrame = 0;

  constructor(backend: HpScoreBackend) {
    this.#backend = backend;
  }

  async push(pixels: ArrayBuffer, frameIndex: number): Promise<void> {
    if (frameIndex !== this.#nextFrame) {
      throw new Error(
        `HP score frames must arrive in order: expected ${this.#nextFrame}, got ${frameIndex}`,
      );
    }
    this.#nextFrame += 1;
    if (this.#filled === 0) this.#firstFrameOfBatch = frameIndex;
    this.#backend.writeLayer(pixels, this.#filled);
    this.#filled += 1;
    if (this.#filled === HP_SCORE_BATCH) await this.#flush();
  }

  /** 残りを流し、全フレーム分の値を並べて返す。 */
  async finish(): Promise<Uint32Array> {
    await this.#flush();
    await Promise.all(this.#pending);
    return Uint32Array.from(this.#results);
  }

  async #flush(): Promise<void> {
    if (this.#filled === 0) return;
    const frames = this.#filled;
    const from = this.#firstFrameOfBatch;
    this.#filled = 0;
    const counted = this.#backend.count(frames).then((values) => {
      this.#store(from, values);
    });
    this.#pending.push(counted);
    // 走らせすぎると読み戻し先が足りなくなる。古いものから待つ。
    if (this.#pending.length >= HP_SCORE_IN_FLIGHT) {
      await this.#pending.shift();
    }
  }

  #store(firstFrame: number, values: Uint32Array): void {
    const at = firstFrame * HP_SCORE_VALUES_PER_FRAME;
    for (let index = 0; index < values.length; index += 1) {
      this.#results[at + index] = values[index] ?? 0;
    }
  }
}
