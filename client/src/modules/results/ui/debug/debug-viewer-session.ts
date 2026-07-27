import type { AnalysisSide } from "~/modules/analysis/contracts.js";
import type { DebugFrameInspector } from "../../application/debug-frame-inspection.js";
import type {
  DebugFrameSource,
  DebugFrameSourceFactory,
} from "../../application/debug-frame-source.js";
import {
  DebugFrameNavigation,
  type DebugFrameNavigationAction,
} from "../../domain/debug-frame-navigation.js";
import type {
  DebugOverlayVisibility,
  DebugViewerData,
} from "./debug-viewer-model.js";
import { DebugFrameRenderer } from "./rendering/frame-renderer.js";

export interface DebugViewerSession {
  navigate(action: DebugFrameNavigationAction): Promise<void>;
  setVisibility(visibility: DebugOverlayVisibility): Promise<void>;
  saveCurrentFrame(): void;
  destroy(): void;
}

interface DebugViewerSessionOptions {
  canvas: HTMLCanvasElement;
  data: DebugViewerData;
  ownSide: AnalysisSide;
  visibility: DebugOverlayVisibility;
  frameSourceFactory: DebugFrameSourceFactory;
  frameInspector: DebugFrameInspector;
  onFrameInfo(label: string): void;
  onError(cause: unknown): void;
}

export async function createDebugViewerSession({
  canvas,
  data,
  ownSide,
  visibility,
  frameSourceFactory,
  frameInspector,
  onFrameInfo,
  onError,
}: DebugViewerSessionOptions): Promise<DebugViewerSession> {
  canvas.width = 1920;
  canvas.height = 1080;

  const hpGeometry = await frameInspector.initialize();
  const totalFrames =
    data.frameCount > 0 ? data.frameCount : data.hpFeatures.length;
  const renderer = new DebugFrameRenderer(
    canvas,
    onFrameInfo,
    data,
    ownSide,
    frameInspector,
    hpGeometry,
  );
  let frameIndex = 0;
  let latestSeekIndex = -1;
  let currentVisibility = { ...visibility };
  let source!: DebugFrameSource;

  const renderFallback = () => {
    void renderer
      .render(frameIndex, source.fallbackSource, currentVisibility)
      .catch(onError);
  };
  source = frameSourceFactory.create(
    {
      file: data.file,
      frameTimestamps: data.frameTimestamps,
      sampleData: data.sampleData,
      videoArrayBuffer: data.videoArrayBuffer,
      codecConfig: data.codecConfig,
      frameToSampleIndex: data.frameToSampleIndex,
    },
    renderFallback,
  );

  try {
    await source.initialize();
  } catch (cause) {
    source.destroy();
    throw cause;
  }

  const seekTo = async (requestedIndex: number): Promise<void> => {
    frameIndex = DebugFrameNavigation.clamp(requestedIndex, totalFrames);
    latestSeekIndex = frameIndex;
    if (!source.usesExactFrames) {
      source.seekFallback(frameIndex);
      return;
    }

    const requested = frameIndex;
    const frame = await source.decode(requested);
    if (latestSeekIndex !== requested) {
      frame?.close();
      return;
    }
    await renderer.render(
      requested,
      frame ?? source.fallbackSource,
      currentVisibility,
      Boolean(frame),
    );
  };

  if (source.usesExactFrames) await seekTo(0);
  else source.seekFallback(0);

  return {
    navigate(action) {
      return seekTo(DebugFrameNavigation.move(frameIndex, totalFrames, action));
    },
    setVisibility(nextVisibility) {
      currentVisibility = { ...nextVisibility };
      if (source.usesExactFrames) return seekTo(frameIndex);
      return renderer.render(
        frameIndex,
        source.fallbackSource,
        currentVisibility,
      );
    },
    saveCurrentFrame() {
      downloadFrame(canvas, frameIndex);
    },
    destroy() {
      source.destroy();
    },
  };
}

function downloadFrame(canvas: HTMLCanvasElement, frameIndex: number): void {
  canvas.toBlob((blob) => {
    if (!blob) return;
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement("a");
    anchor.href = url;
    anchor.download = `frame_${String(frameIndex).padStart(6, "0")}.png`;
    anchor.click();
    URL.revokeObjectURL(url);
  }, "image/png");
}
