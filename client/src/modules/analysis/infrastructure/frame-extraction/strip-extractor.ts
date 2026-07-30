import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  ATTACK_INFO_LAYOUT,
  FIGHT_MARKER_LAYOUT,
  LOWER_ATLAS_LAYOUT,
  MID_ATLAS_LAYOUT,
  SUPER_GAUGE_LAYOUT,
} from "./layout.js";
import type { AnalysisTransferBuffers } from "./strip-buffer-pool.js";

interface StripBitmaps {
  readonly hud: ImageBitmap;
  readonly lowerAtlas: ImageBitmap;
  readonly midAtlas: ImageBitmap;
}

interface PendingStripBitmaps {
  readonly hud: Promise<ImageBitmap>;
  readonly lowerAtlas: Promise<ImageBitmap>;
  readonly midAtlas: Promise<ImageBitmap>;
  readonly fightFrame?: VideoFrame;
}

export interface StripPixels {
  readonly hud: Uint8ClampedArray;
  readonly meter: Uint8ClampedArray;
  readonly input: Uint8ClampedArray;
}

export class FrameStripExtractor {
  readonly #hud = stripCanvas(ANALYSIS_STRIPS.hud.height);
  readonly #meter = stripCanvas(ANALYSIS_STRIPS.meter.height);
  readonly #input = stripCanvas(ANALYSIS_STRIPS.input.height);

  createBitmaps(frame: VideoFrame, frameIndex: number): PendingStripBitmaps {
    return {
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
    return {
      hud: drawHudBitmap(
        this.#hud,
        bitmaps.hud,
        bitmaps.lowerAtlas,
        pending.fightFrame,
      ),
      meter: drawMeterBitmap(this.#meter, bitmaps.lowerAtlas, bitmaps.midAtlas),
      input: drawInputBitmap(this.#input, bitmaps.midAtlas),
    };
  }
}

export function copyStripPixels(
  pixels: StripPixels,
  buffers: AnalysisTransferBuffers,
): void {
  new Uint8Array(buffers.hud).set(pixels.hud);
  new Uint8Array(buffers.meter).set(pixels.meter);
  new Uint8Array(buffers.input).set(pixels.input);
}

interface StripCanvas {
  readonly canvas: OffscreenCanvas;
  readonly context: OffscreenCanvasRenderingContext2D;
}

function stripCanvas(height: number): StripCanvas {
  const canvas = new OffscreenCanvas(ANALYSIS_WIDTH, height);
  const context = canvas.getContext("2d", {
    willReadFrequently: true,
  }) as OffscreenCanvasRenderingContext2D;
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

function drawHudBitmap(
  target: StripCanvas,
  bitmap: ImageBitmap,
  lowerAtlas: ImageBitmap,
  fightFrame: VideoFrame | undefined,
): Uint8ClampedArray {
  target.context.drawImage(bitmap, 0, 0);
  bitmap.close();
  target.context.imageSmoothingEnabled = true;
  target.context.imageSmoothingQuality = "high";
  drawSuperGauge(target.context, lowerAtlas);
  if (fightFrame) {
    const { source, target: destination } = FIGHT_MARKER_LAYOUT;
    target.context.drawImage(
      fightFrame,
      source.x,
      source.y,
      source.width,
      source.height,
      destination.x,
      destination.y,
      destination.width,
      destination.height,
    );
  }
  return readPixels(target);
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
      const { source, target } = patch;
      context.drawImage(
        lowerAtlas,
        source.x,
        source.y,
        source.width,
        source.height,
        target.x,
        target.y,
        target.width,
        target.height,
      );
    }
  }
}

function drawMeterBitmap(
  target: StripCanvas,
  lowerAtlas: ImageBitmap,
  midAtlas: ImageBitmap,
): Uint8ClampedArray {
  const { source, target: destination } = LOWER_ATLAS_LAYOUT.meter;
  target.context.drawImage(
    lowerAtlas,
    source.x,
    source.y,
    source.width,
    source.height,
    destination.x,
    destination.y,
    destination.width,
    destination.height,
  );
  drawAttackInfo(target.context, midAtlas);
  lowerAtlas.close();
  return readPixels(target);
}

function drawAttackInfo(
  context: OffscreenCanvasRenderingContext2D,
  midAtlas: ImageBitmap,
): void {
  for (const side of [ATTACK_INFO_LAYOUT.p1, ATTACK_INFO_LAYOUT.p2] as const) {
    for (const patch of [side.numeric, side.attribute]) {
      const { source, target } = patch;
      context.drawImage(
        midAtlas,
        source.x,
        source.y,
        source.width,
        source.height,
        target.x,
        target.y,
        target.width,
        target.height,
      );
    }
  }
}

function drawInputBitmap(
  target: StripCanvas,
  midAtlas: ImageBitmap,
): Uint8ClampedArray {
  const { source, target: destination } = MID_ATLAS_LAYOUT.input;
  target.context.drawImage(
    midAtlas,
    source.x,
    source.y,
    source.width,
    source.height,
    destination.x,
    destination.y,
    destination.width,
    destination.height,
  );
  midAtlas.close();
  return readPixels(target);
}

function readPixels(target: StripCanvas): Uint8ClampedArray {
  return target.context.getImageData(
    0,
    0,
    target.canvas.width,
    target.canvas.height,
  ).data;
}
