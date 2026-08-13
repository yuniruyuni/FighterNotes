/** GPU へ渡す 1 まとまりの枚数。1 枚ずつ投げると往復の遅延だけで 3ms かかる。 */
export const HP_SCORE_BATCH = 8;

/** 同時に走らせる読み戻しの本数。遅延はこれで隠れる。 */
export const HP_SCORE_IN_FLIGHT = 4;

/** 1 フレームあたりの結果 (p1 一致, p1 全体, p2 一致, p2 全体)。 */
export const HP_SCORE_VALUES_PER_FRAME = 4;

export interface HudGpuResult {
  /** 1 フレームあたり [p1 一致, p1 全体, p2 一致, p2 全体]。 */
  readonly scores: Uint32Array;
  /** 1 フレームあたり p1・p2 の順に並ぶ列の色。 */
  readonly columns: Uint32Array;
  /** 1 フレームあたり左・右の順に並ぶドライブゲージの列。 */
  readonly drive: Uint32Array;
}

/**
 * GPU へ画素を渡す先。WebGPU そのものを差し替えられるようにしてある。
 */
export interface HpScoreBackend {
  /** 切り出した strip を持つテクスチャ。読み戻しに使う。 */
  readonly texture: GPUTexture;
  readonly device: GPUDevice;
  /** まとめの中の `layer` 枚目として画素を置く。 */
  writeLayer(pixels: ArrayBuffer, layer: number): void;
  /**
   * 復号フレームから直接切り出して置く。
   *
   * 呼び出しの中で GPU へ積みきる。`importExternalTexture` はフレームの
   * 資源を押さえるので、呼び出し側は戻り次第フレームを閉じてよい。
   */
  extractLayer(frame: VideoFrame, layer: number): void;
  /** 置いた `frames` 枚を読み、フレームごとの値を順に返す。 */
  count(frames: number): Promise<HudGpuResult>;
}

/** 読み取れたまとまり。先頭のフレーム番号から順に並ぶ。 */
export interface HudGpuBatch extends HudGpuResult {
  readonly firstFrame: number;
}

/**
 * HUD の読み取りを、まとめて GPU へ流す。
 *
 * 読み取った値はフレームの順番に依存しないので、届いた順に渡してよい。
 * 受け取る側が先頭のフレーム番号で置き場所を決める。
 */
export class HpScoreBatcher {
  readonly #backend: HpScoreBackend;
  readonly #onBatch: (batch: HudGpuBatch) => void;
  readonly #pending: Array<Promise<void>> = [];
  #filled = 0;
  #firstFrameOfBatch = 0;
  #nextFrame = 0;

  constructor(
    backend: HpScoreBackend,
    onBatch: (batch: HudGpuBatch) => void = () => {},
  ) {
    this.#backend = backend;
    this.#onBatch = onBatch;
  }

  async push(
    source: ArrayBuffer | VideoFrame,
    frameIndex: number,
  ): Promise<void> {
    if (frameIndex !== this.#nextFrame) {
      throw new Error(
        `HP score frames must arrive in order: expected ${this.#nextFrame}, got ${frameIndex}`,
      );
    }
    this.#nextFrame += 1;
    if (this.#filled === 0) this.#firstFrameOfBatch = frameIndex;
    if (source instanceof ArrayBuffer) {
      this.#backend.writeLayer(source, this.#filled);
    } else {
      this.#backend.extractLayer(source, this.#filled);
    }
    this.#filled += 1;
    if (this.#filled === HP_SCORE_BATCH) await this.#flush();
  }

  /** 残りを流し、走らせた分が全て届くまで待つ。 */
  async finish(): Promise<void> {
    await this.#flush();
    await Promise.all(this.#pending);
  }

  async #flush(): Promise<void> {
    if (this.#filled === 0) return;
    const frames = this.#filled;
    const firstFrame = this.#firstFrameOfBatch;
    this.#filled = 0;
    const counted = this.#backend.count(frames).then((result) => {
      this.#onBatch({ firstFrame, ...result });
    });
    this.#pending.push(counted);
    // 走らせすぎると読み戻し先が足りなくなる。古いものから待つ。
    if (this.#pending.length >= HP_SCORE_IN_FLIGHT) {
      await this.#pending.shift();
    }
  }
}
