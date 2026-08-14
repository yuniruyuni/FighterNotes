import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  ATTACK_INFO_LAYOUT,
  FIGHT_MARKER_LAYOUT,
  LOWER_ATLAS_LAYOUT,
  MID_ATLAS_LAYOUT,
  PACKED_BANDS,
  SUPER_BAND_HEIGHT,
  SUPER_GAUGE_LAYOUT,
  SUPER_NATIVE_RECTS,
} from "./layout.js";
import type { AnalysisTransferBuffers } from "./strip-buffer-pool.js";

interface StripBitmaps {
  readonly hud: ImageBitmap;
  readonly lowerAtlas: ImageBitmap;
  readonly midAtlas: ImageBitmap;
}

/** SA ゲージを等倍で取るための切り出し。偶数境界から取る。 */
const SUPER_SOURCE = { x: 0, y: 954, width: ANALYSIS_WIDTH, height: 78 };

interface PendingStripBitmaps {
  readonly superSource: Promise<ImageBitmap>;
  readonly hud: Promise<ImageBitmap>;
  readonly lowerAtlas: Promise<ImageBitmap>;
  readonly midAtlas: Promise<ImageBitmap>;
  readonly fightFrame?: VideoFrame;
}

export interface StripPixels {
  readonly hud: Uint8ClampedArray;
  /** 等倍で置いた SA ゲージ。 */
  readonly super: Uint8ClampedArray;
  readonly meter: Uint8ClampedArray;
  readonly input: Uint8ClampedArray;
}

/** 3 つの strip を縦に並べた 1 枚の中での、各 strip の位置。 */
const BANDS = {
  hud: { y: 0, height: ANALYSIS_STRIPS.hud.height },
  super: { y: PACKED_BANDS.super, height: SUPER_BAND_HEIGHT },
  meter: {
    y: ANALYSIS_STRIPS.hud.height,
    height: ANALYSIS_STRIPS.meter.height,
  },
  input: {
    y: ANALYSIS_STRIPS.hud.height + ANALYSIS_STRIPS.meter.height,
    height: ANALYSIS_STRIPS.input.height,
  },
} as const;

const CANVAS_HEIGHT = PACKED_BANDS.super + SUPER_BAND_HEIGHT;

/**
 * 動画フレームから 3 つの strip を取り出す。
 *
 * 3 つの strip を 1 枚の canvas へ縦に並べ、読み戻しを 1 回にまとめる。
 * `getImageData` の費用はほぼ全てが GPU との同期待ちで、実機計測では読む
 * 範囲を 1 画素にしても 1 フレームあたり 1.11ms かかり、全域を読む 1.33ms と
 * ほとんど変わらなかった。呼ぶ回数そのものが解析時間を決めている。
 *
 * 描画元の ImageBitmap は従来どおり `createBitmaps` で先に作る。生成は次の
 * フレームの復号と重なるため、待ち時間は 10000 フレームで 18ms しかない。
 */
export class FrameStripExtractor {
  readonly #packed = packedCanvas();

  createBitmaps(frame: VideoFrame, frameIndex: number): PendingStripBitmaps {
    return {
      superSource: createPatchBitmap(frame, SUPER_SOURCE),
      hud: createStripBitmap(frame, ANALYSIS_STRIPS.hud),
      lowerAtlas: createPatchBitmap(frame, LOWER_ATLAS_LAYOUT.source),
      midAtlas: createPatchBitmap(frame, MID_ATLAS_LAYOUT.source),
      ...(frameIndex % FIGHT_MARKER_LAYOUT.sampleInterval === 0
        ? { fightFrame: frame }
        : {}),
    };
  }

  async readBitmaps(pending: PendingStripBitmaps): Promise<StripPixels> {
    const bitmaps: StripBitmaps = {
      hud: await pending.hud,
      lowerAtlas: await pending.lowerAtlas,
      midAtlas: await pending.midAtlas,
    };
    const superSource = await pending.superSource;
    const { context } = this.#packed;
    // 縮小するのは HUD 帯へ描く SA ゲージと FIGHT だけで、そこだけが高品質
    // 補間だった。1 枚に束ねても品質の切り替えを帯ごとに保ち、strip の内容を
    // 従来と同じにする。
    context.imageSmoothingQuality = "high";
    drawHud(context, bitmaps.hud, bitmaps.lowerAtlas, pending.fightFrame);
    context.imageSmoothingQuality = "low";
    drawMeter(context, bitmaps.lowerAtlas, bitmaps.midAtlas);
    drawInput(context, bitmaps.midAtlas);
    // SA ゲージは等倍で置く。彩度は 2px 単位で持たれているので、偶数境界から
    // 切り出したものを使う。
    for (const rect of SUPER_NATIVE_RECTS) {
      context.drawImage(
        superSource,
        rect.src.x - SUPER_SOURCE.x,
        rect.src.y - SUPER_SOURCE.y,
        rect.src.width,
        rect.src.height,
        rect.dst.x,
        rect.dst.y,
        rect.dst.width,
        rect.dst.height,
      );
    }
    superSource.close();
    bitmaps.lowerAtlas.close();
    bitmaps.midAtlas.close();

    const pixels = context.getImageData(
      0,
      0,
      ANALYSIS_WIDTH,
      CANVAS_HEIGHT,
    ).data;
    return {
      super: band(pixels, BANDS.super),
      hud: band(pixels, BANDS.hud),
      meter: band(pixels, BANDS.meter),
      input: band(pixels, BANDS.input),
    };
  }
}

export function copyStripPixels(
  pixels: StripPixels,
  buffers: AnalysisTransferBuffers,
): void {
  new Uint8Array(buffers.hud).set(pixels.hud);
  new Uint8Array(buffers.super).set(pixels.super);
  new Uint8Array(buffers.meter).set(pixels.meter);
  // 攻撃情報のワーカーは meter strip をそのまま読む。転送すると所有権が移る
  // ため、同じ画素をもう 1 枚渡す。
  new Uint8Array(buffers.attack).set(pixels.meter);
  new Uint8Array(buffers.input).set(pixels.input);
}

/** 読み戻した 1 枚は band ごとに連続しているので、strip へは view で渡す。 */
function band(
  pixels: Uint8ClampedArray,
  target: { readonly y: number; readonly height: number },
): Uint8ClampedArray {
  const from = target.y * ANALYSIS_WIDTH * 4;
  return pixels.subarray(from, from + target.height * ANALYSIS_WIDTH * 4);
}

function packedCanvas() {
  const canvas = new OffscreenCanvas(ANALYSIS_WIDTH, CANVAS_HEIGHT);
  // `willReadFrequently` は canvas を CPU 側へ置く。GPU 上の ImageBitmap を
  // 描くたびに転送と software 合成が起き、実機計測で 1 フレーム 6.24ms の
  // うち 5.2ms を占めていた。GPU 側に置いたまま描いて最後に読み戻す方が速い。
  const context = canvas.getContext("2d") as OffscreenCanvasRenderingContext2D;
  context.imageSmoothingEnabled = true;
  return { canvas, context };
}

function createStripBitmap(
  frame: VideoFrame,
  strip: { readonly y: number; readonly height: number },
): Promise<ImageBitmap> {
  return createImageBitmap(frame, 0, strip.y, ANALYSIS_WIDTH, strip.height);
}

function createPatchBitmap(
  frame: VideoFrame,
  patch: {
    readonly x: number;
    readonly y: number;
    readonly width: number;
    readonly height: number;
  },
): Promise<ImageBitmap> {
  return createImageBitmap(frame, patch.x, patch.y, patch.width, patch.height);
}

function drawHud(
  context: OffscreenCanvasRenderingContext2D,
  bitmap: ImageBitmap,
  lowerAtlas: ImageBitmap,
  fightFrame: VideoFrame | undefined,
): void {
  context.drawImage(bitmap, 0, BANDS.hud.y);
  bitmap.close();
  drawSuperGauge(context, lowerAtlas);
  if (fightFrame) {
    drawPatch(context, fightFrame, FIGHT_MARKER_LAYOUT, BANDS.hud.y);
  }
}

function drawSuperGauge(
  context: OffscreenCanvasRenderingContext2D,
  lowerAtlas: ImageBitmap,
): void {
  for (const side of [
    SUPER_GAUGE_LAYOUT.left,
    SUPER_GAUGE_LAYOUT.right,
  ] as const) {
    for (const patch of [side.label, side.bar]) {
      drawPatch(context, lowerAtlas, patch, BANDS.hud.y);
    }
  }
}

function drawMeter(
  context: OffscreenCanvasRenderingContext2D,
  lowerAtlas: ImageBitmap,
  midAtlas: ImageBitmap,
): void {
  drawPatch(context, lowerAtlas, LOWER_ATLAS_LAYOUT.meter, BANDS.meter.y);
  drawAttackInfo(context, midAtlas);
}

function drawAttackInfo(
  context: OffscreenCanvasRenderingContext2D,
  midAtlas: ImageBitmap,
): void {
  for (const side of [ATTACK_INFO_LAYOUT.p1, ATTACK_INFO_LAYOUT.p2] as const) {
    for (const patch of [side.numeric, side.attribute]) {
      drawPatch(context, midAtlas, patch, BANDS.meter.y);
    }
  }
}

function drawInput(
  context: OffscreenCanvasRenderingContext2D,
  midAtlas: ImageBitmap,
): void {
  drawPatch(context, midAtlas, MID_ATLAS_LAYOUT.input, BANDS.input.y);
}

/** strip 内の座標で書かれた配置を、束ねた 1 枚の中の位置へずらして描く。 */
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
    readonly target: {
      readonly x: number;
      readonly y: number;
      readonly width: number;
      readonly height: number;
    };
  },
  bandY: number,
): void {
  context.drawImage(
    image,
    patch.source.x,
    patch.source.y,
    patch.source.width,
    patch.source.height,
    patch.target.x,
    bandY + patch.target.y,
    patch.target.width,
    patch.target.height,
  );
}
