import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  PACKED_BANDS,
  PACKED_HEIGHT,
} from "../frame-extraction/layout.js";
import type { StripPixels } from "../frame-extraction/strip-extractor.js";
import {
  HP_SCORE_BATCH,
  HP_SCORE_IN_FLIGHT,
  type HpScoreBackend,
  type HudGpuBatch,
} from "./hp-score-batcher.js";

/** 読み戻しの 1 行の長さ。256 バイト単位でなければならない。 */
const ROW_BYTES = ANALYSIS_WIDTH * 4;

export interface PendingStrip {
  readonly pixels: Promise<Uint8Array>;
}

interface Readback {
  readonly buffer: GPUBuffer;
  readonly scratch: Uint8Array;
}

/**
 * strip を GPU から受け取る取り出し器。
 *
 * canvas へ合成して `getImageData` で読み戻す経路を置き換える。復号フレームを
 * そのまま GPU で切り出すので、画素は一度も CPU を経由しない。
 *
 * 1 枚ずつ投げて数枚先行させる。`createBitmaps` で submit まで済ませ、
 * `readBitmaps` で順に待つ。往復の遅延は先行分で隠れる。
 *
 * 読み取り (HP・ドライブ) は層を埋め終えた直後にまとめて積む。次のフレームが
 * 同じ層を上書きするより先に読まれる。GPU の実行は積んだ順に進むため、
 * 待ち合わせは要らない。
 */
export class GpuStripExtractor {
  readonly #device: GPUDevice;
  readonly #backend: HpScoreBackend;
  readonly #texture: GPUTexture;
  readonly #onBatch: (batch: HudGpuBatch) => void;
  /**
   * 読み戻し先は使い終わったものを貸し出す。番号で使い回すと、先行の枚数が
   * 想定を超えたときに、まだ読み終えていないものを上書きしてしまう。
   */
  readonly #free: Readback[] = [];
  readonly #pending: Array<Promise<void>> = [];
  readonly #bytes = ROW_BYTES * PACKED_HEIGHT;
  #filled = 0;
  #firstFrameOfBatch = 0;

  constructor(options: {
    readonly device: GPUDevice;
    readonly backend: HpScoreBackend;
    readonly texture: GPUTexture;
    readonly onBatch: (batch: HudGpuBatch) => void;
  }) {
    this.#device = options.device;
    this.#backend = options.backend;
    this.#texture = options.texture;
    this.#onBatch = options.onBatch;
  }

  createBitmaps(frame: VideoFrame, frameIndex: number): PendingStrip {
    const layer = this.#filled;
    if (layer === 0) this.#firstFrameOfBatch = frameIndex;
    this.#backend.extractLayer(frame, layer);
    const pixels = this.#readBack(layer);
    this.#filled += 1;
    if (this.#filled === HP_SCORE_BATCH) this.#runBatch();
    return { pixels };
  }

  async readBitmaps(pending: PendingStrip): Promise<StripPixels> {
    // 読み取りのまとまりを走らせすぎると、読み戻し先が足りなくなる。
    // 積むのは同期の場で行うので、待ち合わせはここで入れる。
    while (this.#pending.length >= HP_SCORE_IN_FLIGHT) {
      await this.#pending.shift();
    }
    const pixels = await pending.pixels;
    return {
      hud: band(pixels, PACKED_BANDS.hud, ANALYSIS_STRIPS.hud.height),
      meter: band(pixels, PACKED_BANDS.meter, ANALYSIS_STRIPS.meter.height),
      input: band(pixels, PACKED_BANDS.input, ANALYSIS_STRIPS.input.height),
    };
  }

  /** 残りを流し、走らせた分が全て届くまで待つ。 */
  async finish(): Promise<void> {
    this.#runBatch();
    await Promise.all(this.#pending);
  }

  #runBatch(): void {
    if (this.#filled === 0) return;
    const frames = this.#filled;
    const firstFrame = this.#firstFrameOfBatch;
    this.#filled = 0;
    this.#pending.push(
      this.#backend.count(frames).then((result) => {
        this.#onBatch({ firstFrame, ...result });
      }),
    );
  }

  async #readBack(layer: number): Promise<Uint8Array> {
    const lent = this.#free.pop() ?? {
      buffer: this.#device.createBuffer({
        label: "strip-readback",
        size: this.#bytes,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
      }),
      scratch: new Uint8Array(this.#bytes),
    };
    const encoder = this.#device.createCommandEncoder();
    encoder.copyTextureToBuffer(
      { texture: this.#texture, origin: [0, 0, layer] },
      {
        buffer: lent.buffer,
        bytesPerRow: ROW_BYTES,
        rowsPerImage: PACKED_HEIGHT,
      },
      [ANALYSIS_WIDTH, PACKED_HEIGHT, 1],
    );
    this.#device.queue.submit([encoder.finish()]);
    await lent.buffer.mapAsync(GPUMapMode.READ);
    // 写してから貸し出しへ戻す。画素は呼び出し側が転送用の場所へ複製する。
    lent.scratch.set(new Uint8Array(lent.buffer.getMappedRange()));
    lent.buffer.unmap();
    this.#free.push(lent);
    return lent.scratch;
  }
}

/** 読み戻した 1 枚は band ごとに連続しているので、strip へは view で渡す。 */
function band(
  pixels: Uint8Array,
  from: number,
  height: number,
): Uint8ClampedArray {
  return new Uint8ClampedArray(
    pixels.buffer,
    pixels.byteOffset + from * ROW_BYTES,
    height * ROW_BYTES,
  );
}
