import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  ATTACK_INFO_COPY_WINDOWS,
  type CopyWindow,
  FIGHT_MARKER_LAYOUT,
  METER_COPY_WINDOW,
  type ScaledCopyWindow,
  SUPER_GAUGE_COPY_WINDOWS,
} from "./layout.js";
import type { StripPixels } from "./strip-extractor.js";

/**
 * `VideoFrame.copyTo` で必要な領域だけを取り出す抽出器。
 *
 * `createImageBitmap` は切り出し範囲に関係なくフレーム全体を変換するため、
 * 1回あたり約6msかかる。1フレーム3回呼ぶ従来経路は実測21.7ms/frameで、
 * 解析時間の大半を占めていた。等倍でよい領域は `copyTo` で直接読み、縮小が
 * 必要な SA ゲージと FIGHT だけを1枚の ImageBitmap から描く。
 *
 * I420 の彩度は 2px 単位なので、奇数 x から読むと canvas 経路と値がずれる。
 * 偶数境界で読んで書き写すことで、strip の内容を従来と同一に保つ。
 */
export class CopyStripExtractor {
  readonly #hud = new Uint8ClampedArray(ANALYSIS_STRIPS.hud.byteLength);
  readonly #meter = new Uint8ClampedArray(ANALYSIS_STRIPS.meter.byteLength);
  readonly #input = new Uint8ClampedArray(ANALYSIS_STRIPS.input.byteLength);
  readonly #meterRow = new Uint8ClampedArray(ANALYSIS_STRIPS.meter.byteLength);
  readonly #patches = new Map<string, Uint8ClampedArray<ArrayBuffer>>();
  readonly #insets = insetCanvas();

  /**
   * 先に始めてよいのは共有バッファへ触らない ImageBitmap の生成だけ。
   * copyTo は書き込み先を共有しているため、直列区間の readBitmaps で行う。
   */
  createBitmaps(frame: VideoFrame, frameIndex: number): PendingCopies {
    return { frame, frameIndex };
  }

  async readBitmaps(pending: PendingCopies): Promise<StripPixels> {
    const { frame } = pending;
    const copies: Promise<unknown>[] = [
      frame.copyTo(this.#hud, stripRect(ANALYSIS_STRIPS.hud)),
      frame.copyTo(this.#input, stripRect(ANALYSIS_STRIPS.input)),
      frame.copyTo(this.#meterRow, stripRect(ANALYSIS_STRIPS.meter)),
    ];
    for (const window of ATTACK_INFO_COPY_WINDOWS) {
      copies.push(frame.copyTo(this.#patchBuffer(window), windowRect(window)));
    }
    const insets = this.#drawInsets(pending);
    await Promise.all(copies);
    // meter は strip 幅そのままの行を読んでいるので、読み出し原点は x=0。
    writeWindow(
      this.#meterRow,
      ANALYSIS_WIDTH,
      0,
      METER_COPY_WINDOW,
      this.#meter,
    );
    for (const window of ATTACK_INFO_COPY_WINDOWS) {
      const patch = this.#patches.get(window.key);
      if (!patch) continue;
      writeWindow(patch, window.readWidth, window.readX, window, this.#meter);
    }
    // 縮小した領域は HUD 基部のコピーが終わってから重ねる。順序が入れ替わると
    // 基部が SA ゲージを上書きしてしまう。
    for (const inset of await insets) {
      writeRows(
        inset.pixels,
        inset.target.width,
        inset.target.width,
        inset.target.height,
        this.#hud,
        inset.target,
      );
    }
    return { hud: this.#hud, meter: this.#meter, input: this.#input };
  }

  #patchBuffer(window: CopyWindow): Uint8ClampedArray<ArrayBuffer> {
    const existing = this.#patches.get(window.key);
    if (existing) return existing;
    const created = new Uint8ClampedArray(
      new ArrayBuffer(window.readWidth * window.source.height * 4),
    );
    this.#patches.set(window.key, created);
    return created;
  }

  /**
   * 縮小が要る領域だけを描き、その小領域だけ読み戻す。
   *
   * 元領域を `copyTo` で読んでから小さな bitmap を作る。フレームを
   * `createImageBitmap` へ渡すと切り出し範囲に関係なく全体を変換するため、
   * 実測で 9.3ms/frame かかっていたものが 1.0ms/frame で済む。
   */
  async #drawInsets(pending: PendingCopies): Promise<readonly InsetPixels[]> {
    const { context } = this.#insets;
    const targets: InsetTarget[] = [];
    for (const window of SUPER_GAUGE_COPY_WINDOWS) {
      const buffer = this.#scaledBuffer(window);
      await pending.frame.copyTo(buffer, {
        rect: {
          x: window.readX,
          y: window.readY,
          width: window.readWidth,
          height: window.readHeight,
        },
        format: "RGBA",
      });
      const bitmap = await createImageBitmap(
        new ImageData(buffer, window.readWidth, window.readHeight),
      );
      context.drawImage(
        bitmap,
        window.offsetX,
        window.offsetY,
        window.source.width,
        window.source.height,
        window.target.x,
        window.target.y,
        window.target.width,
        window.target.height,
      );
      bitmap.close();
      targets.push(window.target);
    }

    if (pending.frameIndex % FIGHT_MARKER_LAYOUT.sampleInterval === 0) {
      drawPatch(context, pending.frame, FIGHT_MARKER_LAYOUT);
      targets.push(FIGHT_MARKER_LAYOUT.target);
    }

    return targets.map((target) => ({
      target,
      pixels: context.getImageData(
        target.x,
        target.y,
        target.width,
        target.height,
      ).data,
    }));
  }

  #scaledBuffer(window: ScaledCopyWindow): Uint8ClampedArray<ArrayBuffer> {
    const existing = this.#patches.get(window.key);
    if (existing) return existing;
    // ImageData は ArrayBuffer 由来の配列だけを受け取る。
    const created = new Uint8ClampedArray(
      new ArrayBuffer(window.readWidth * window.readHeight * 4),
    );
    this.#patches.set(window.key, created);
    return created;
  }
}

interface InsetPixels {
  readonly target: InsetTarget;
  readonly pixels: Uint8ClampedArray;
}

/**
 * copyTo での取り出しを選ぶか。
 *
 * copyTo は画素を CPU バッファへ書き出すため、GPU 上の復号フレームでは
 * 読み戻しの待ちが入る。実機計測では canvas 2.19ms/frame に対し copyTo
 * 8.49ms/frame で、GPU の無い環境の結果（canvas 21.7 対 copyTo 1.3）とは
 * 逆になる。既定は実利用者の環境に合わせ、明示した場合だけ copyTo を使う。
 */
export function preferCopyExtraction(): boolean {
  return (
    typeof globalThis.location === "object" &&
    new URLSearchParams(globalThis.location?.search ?? "").get("extraction") ===
      "copy"
  );
}

/**
 * `copyTo` の RGBA 変換に対応しているかを一度だけ確かめる。
 * 対応しない環境では従来の canvas 経路へ落とす。
 */
export async function supportsRgbaCopy(): Promise<boolean> {
  if (typeof VideoFrame !== "function") return false;
  const canvas = new OffscreenCanvas(2, 2);
  canvas.getContext("2d")?.fillRect(0, 0, 2, 2);
  const frame = new VideoFrame(canvas, { timestamp: 0 });
  try {
    await frame.copyTo(new Uint8ClampedArray(2 * 2 * 4), {
      rect: { x: 0, y: 0, width: 2, height: 2 },
      format: "RGBA",
    });
    return true;
  } catch {
    return false;
  } finally {
    frame.close();
  }
}

export interface PendingCopies {
  readonly frame: VideoFrame;
  readonly frameIndex: number;
}

interface InsetTarget {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

function drawPatch(
  context: OffscreenCanvasRenderingContext2D,
  image: CanvasImageSource,
  patch: {
    readonly source: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
    readonly target: InsetTarget;
  },
): void {
  context.drawImage(
    image,
    patch.source.x,
    patch.source.y,
    patch.source.width,
    patch.source.height,
    patch.target.x,
    patch.target.y,
    patch.target.width,
    patch.target.height,
  );
}

function stripRect(strip: {
  readonly y: number;
  readonly height: number;
}): VideoFrameCopyToOptions {
  return {
    rect: { x: 0, y: strip.y, width: ANALYSIS_WIDTH, height: strip.height },
    format: "RGBA",
  };
}

function windowRect(window: CopyWindow): VideoFrameCopyToOptions {
  return {
    rect: {
      x: window.readX,
      y: window.source.y,
      width: window.readWidth,
      height: window.source.height,
    },
    format: "RGBA",
  };
}

function insetCanvas() {
  const canvas = new OffscreenCanvas(
    ANALYSIS_WIDTH,
    ANALYSIS_STRIPS.hud.height,
  );
  const context = canvas.getContext("2d", {
    willReadFrequently: true,
  }) as OffscreenCanvasRenderingContext2D;
  context.imageSmoothingEnabled = true;
  context.imageSmoothingQuality = "high";
  return { canvas, context };
}

function writeWindow(
  source: Uint8ClampedArray,
  sourceStride: number,
  readOriginX: number,
  window: CopyWindow,
  strip: Uint8ClampedArray,
): void {
  writeRows(
    source.subarray((window.source.x - readOriginX) * 4),
    sourceStride,
    window.source.width,
    window.source.height,
    strip,
    window.target,
  );
}

/** 行ごとに窓を写す。strip の行の刻みは常に解析幅で固定。 */
function writeRows(
  source: Uint8ClampedArray,
  sourceStride: number,
  width: number,
  height: number,
  strip: Uint8ClampedArray,
  target: { readonly x: number; readonly y: number },
): void {
  const rowBytes = width * 4;
  for (let row = 0; row < height; row += 1) {
    const from = row * sourceStride * 4;
    const to = ((target.y + row) * ANALYSIS_WIDTH + target.x) * 4;
    strip.set(source.subarray(from, from + rowBytes), to);
  }
}
