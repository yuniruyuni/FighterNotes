import { ANALYSIS_STRIPS, ANALYSIS_WIDTH } from "./layout.js";
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

  createBitmaps(frame: VideoFrame): PendingStripBitmaps {
    return {
      hud: createStripBitmap(frame, ANALYSIS_STRIPS.hud),
      meter: createStripBitmap(frame, ANALYSIS_STRIPS.meter),
      input: createStripBitmap(frame, ANALYSIS_STRIPS.input),
    };
  }

  async readBitmaps(pending: PendingStripBitmaps): Promise<StripPixels> {
    const bitmaps: StripBitmaps = {
      hud: await pending.hud,
      meter: await pending.meter,
      input: await pending.input,
    };
    return {
      hud: drawBitmap(this.#hud, bitmaps.hud),
      meter: drawBitmap(this.#meter, bitmaps.meter),
      input: drawBitmap(this.#input, bitmaps.input),
    };
  }

  readFrame(frame: VideoFrame): StripPixels {
    return {
      hud: drawFrameStrip(this.#hud, frame, ANALYSIS_STRIPS.hud),
      meter: drawFrameStrip(this.#meter, frame, ANALYSIS_STRIPS.meter),
      input: drawFrameStrip(this.#input, frame, ANALYSIS_STRIPS.input),
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
  return target.context.getImageData(
    0,
    0,
    target.canvas.width,
    target.canvas.height,
  ).data;
}

function drawFrameStrip(
  target: StripCanvas,
  frame: VideoFrame,
  strip: { readonly y: number; readonly height: number },
): Uint8ClampedArray {
  target.context.drawImage(
    frame,
    0,
    strip.y,
    ANALYSIS_WIDTH,
    strip.height,
    0,
    0,
    ANALYSIS_WIDTH,
    strip.height,
  );
  return target.context.getImageData(0, 0, ANALYSIS_WIDTH, strip.height).data;
}
