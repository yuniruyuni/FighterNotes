import {
  ANALYSIS_STRIPS,
  ANALYSIS_WIDTH,
  FIGHT_MARKER_LAYOUT,
} from "./layout.js";
import type { AnalysisTransferBuffers } from "./strip-buffer-pool.js";

interface StripBitmaps {
  readonly hud: ImageBitmap;
  readonly meter: ImageBitmap;
  readonly input: ImageBitmap;
}

interface PendingStripBitmaps {
  readonly hud: Promise<ImageBitmap>;
  readonly meter: Promise<ImageBitmap>;
  readonly input: Promise<ImageBitmap>;
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
      meter: createStripBitmap(frame, ANALYSIS_STRIPS.meter),
      input: createStripBitmap(frame, ANALYSIS_STRIPS.input),
      ...(frameIndex % FIGHT_MARKER_LAYOUT.sampleInterval === 0
        ? { fightFrame: frame }
        : {}),
    };
  }

  async readBitmaps(pending: PendingStripBitmaps): Promise<StripPixels> {
    const bitmaps: StripBitmaps = {
      hud: await pending.hud,
      meter: await pending.meter,
      input: await pending.input,
    };
    return {
      hud: drawHudBitmap(this.#hud, bitmaps.hud, pending.fightFrame),
      meter: drawBitmap(this.#meter, bitmaps.meter),
      input: drawBitmap(this.#input, bitmaps.input),
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

function drawBitmap(
  target: StripCanvas,
  bitmap: ImageBitmap,
): Uint8ClampedArray {
  target.context.drawImage(bitmap, 0, 0);
  bitmap.close();
  return readPixels(target);
}

function drawHudBitmap(
  target: StripCanvas,
  bitmap: ImageBitmap,
  fightFrame: VideoFrame | undefined,
): Uint8ClampedArray {
  target.context.drawImage(bitmap, 0, 0);
  bitmap.close();
  if (fightFrame) {
    const { source, target: destination } = FIGHT_MARKER_LAYOUT;
    target.context.imageSmoothingEnabled = true;
    target.context.imageSmoothingQuality = "high";
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

function readPixels(target: StripCanvas): Uint8ClampedArray {
  return target.context.getImageData(
    0,
    0,
    target.canvas.width,
    target.canvas.height,
  ).data;
}
