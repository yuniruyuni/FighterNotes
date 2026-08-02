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
  signal: AbortSignal;
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
  signal,
  visibility,
  frameSourceFactory,
  frameInspector,
  onFrameInfo,
  onError,
}: DebugViewerSessionOptions): Promise<DebugViewerSession> {
  let frameIndex = 0;
  let latestSeekIndex = -1;
  let currentVisibility = { ...visibility };
  let source: DebugFrameSource | undefined;
  let sourceData: Parameters<DebugFrameSourceFactory["create"]>[0] | undefined;
  let destroyed = false;

  const destroy = () => {
    if (destroyed) return;
    destroyed = true;
    latestSeekIndex += 1;
    signal.removeEventListener("abort", destroy);
    data.videoArrayBuffer = null;
    if (sourceData) sourceData.videoArrayBuffer = null;
    source?.destroy();
  };
  signal.addEventListener("abort", destroy, { once: true });
  if (signal.aborted) destroy();

  try {
    throwIfAborted(signal);
    canvas.width = 1920;
    canvas.height = 1080;

    const hpGeometry = await frameInspector.initialize();
    throwIfAborted(signal);
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

    const renderFallback = () => {
      if (destroyed || !source) return;
      void renderer
        .render(frameIndex, source.fallbackSource, currentVisibility)
        .catch((cause) => {
          if (!destroyed) onError(cause);
        });
    };
    sourceData = {
      file: data.file,
      frameTimestamps: data.frameTimestamps,
      sampleData: data.sampleData,
      videoArrayBuffer: data.videoArrayBuffer,
      codecConfig: data.codecConfig,
      frameToSampleIndex: data.frameToSampleIndex,
    };
    data.videoArrayBuffer = null;
    const activeSource = frameSourceFactory.create(sourceData, renderFallback);
    source = activeSource;
    await activeSource.initialize();
    throwIfAborted(signal);

    const seekTo = async (requestedIndex: number): Promise<void> => {
      if (destroyed) return;
      frameIndex = DebugFrameNavigation.clamp(requestedIndex, totalFrames);
      latestSeekIndex = frameIndex;
      if (!activeSource.usesExactFrames) {
        activeSource.seekFallback(frameIndex);
        return;
      }

      const requested = frameIndex;
      const frame = await activeSource.decode(requested);
      if (destroyed || signal.aborted || latestSeekIndex !== requested) {
        frame?.close();
        throwIfAborted(signal);
        return;
      }
      await renderer.render(
        requested,
        frame ?? activeSource.fallbackSource,
        currentVisibility,
        Boolean(frame),
      );
    };

    if (activeSource.usesExactFrames) await seekTo(0);
    else activeSource.seekFallback(0);
    throwIfAborted(signal);

    return {
      navigate(action) {
        return seekTo(
          DebugFrameNavigation.move(frameIndex, totalFrames, action),
        );
      },
      setVisibility(nextVisibility) {
        if (destroyed) return Promise.resolve();
        currentVisibility = { ...nextVisibility };
        if (activeSource.usesExactFrames) return seekTo(frameIndex);
        return renderer.render(
          frameIndex,
          activeSource.fallbackSource,
          currentVisibility,
        );
      },
      saveCurrentFrame() {
        if (!destroyed) downloadFrame(canvas, frameIndex);
      },
      destroy,
    };
  } catch (cause) {
    destroy();
    throwIfAborted(signal);
    throw cause;
  }
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

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new DOMException("認識デバッグを終了しました", "AbortError");
}
