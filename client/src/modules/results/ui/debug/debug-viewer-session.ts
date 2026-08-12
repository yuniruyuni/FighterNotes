import type { AnalysisSide } from "~/modules/analysis/contracts.js";
import type { DebugFrameInspector } from "../../application/debug-frame-inspection.js";
import type {
  DebugFrameSource,
  DebugFrameSourceFactory,
} from "../../application/debug-frame-source.js";
import {
  FrameNavigation,
  type FrameNavigationAction,
} from "../../domain/frame-navigation.js";
import type { PlaybackRate } from "../playback-rate.js";
import {
  type DebugOverlayVisibility,
  type DebugViewerData,
  frameDataAt,
} from "./debug-viewer-model.js";
import { DebugFrameRenderer } from "./rendering/frame-renderer.js";

export interface DebugViewerSession {
  navigate(action: FrameNavigationAction): Promise<void>;
  setVisibility(visibility: DebugOverlayVisibility): Promise<void>;
  /** 再生の開始・停止。復号が追いつかない分は駒を落として実時間へ寄せる。 */
  setPlaying(playing: boolean): void;
  setPlaybackRate(rate: PlaybackRate): void;
  saveCurrentFrame(): void;
  saveCurrentFrameData(): void;
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
  onPlayingChange(playing: boolean): void;
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
  onPlayingChange,
  onError,
}: DebugViewerSessionOptions): Promise<DebugViewerSession> {
  let frameIndex = 0;
  let latestRequest = 0;
  let currentVisibility = { ...visibility };
  let source: DebugFrameSource | undefined;
  let playing = false;
  let playbackRate: PlaybackRate = 1;
  let playTimer: ReturnType<typeof setTimeout> | undefined;
  let sourceData: Parameters<DebugFrameSourceFactory["create"]>[0] | undefined;
  let destroyed = false;

  const stopPlayback = (notify: boolean) => {
    if (playTimer !== undefined) clearTimeout(playTimer);
    playTimer = undefined;
    if (!playing) return;
    playing = false;
    if (notify) onPlayingChange(false);
  };

  const destroy = () => {
    if (destroyed) return;
    destroyed = true;
    stopPlayback(false);
    latestRequest += 1;
    signal.removeEventListener("abort", destroy);
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
      codecConfig: data.codecConfig,
      frameToSampleIndex: data.frameToSampleIndex,
    };
    const activeSource = frameSourceFactory.create(sourceData, renderFallback);
    source = activeSource;
    await activeSource.initialize();
    throwIfAborted(signal);

    const seekTo = async (requestedIndex: number): Promise<void> => {
      if (destroyed) return;
      frameIndex = FrameNavigation.clamp(requestedIndex, totalFrames);
      const request = ++latestRequest;
      if (!activeSource.usesExactFrames) {
        activeSource.seekFallback(frameIndex);
        return;
      }

      const requestedFrame = frameIndex;
      const frame = await activeSource.decode(requestedFrame);
      if (destroyed || signal.aborted || latestRequest !== request) {
        frame?.close();
        throwIfAborted(signal);
        return;
      }
      await renderer.render(
        requestedFrame,
        frame ?? activeSource.fallbackSource,
        currentVisibility,
        Boolean(frame),
      );
    };

    if (activeSource.usesExactFrames) await seekTo(0);
    else activeSource.seekFallback(0);
    throwIfAborted(signal);

    const tick = async () => {
      playTimer = undefined;
      if (destroyed || !playing) return;
      if (frameIndex >= totalFrames - 1) {
        stopPlayback(true);
        return;
      }
      const startedAt = performance.now();
      await seekTo(frameIndex + 1);
      if (destroyed || !playing) return;
      const remaining =
        1000 / (FRAMES_PER_SECOND * playbackRate) -
        (performance.now() - startedAt);
      playTimer = setTimeout(() => void tick(), Math.max(0, remaining));
    };

    return {
      navigate(action) {
        // 手で動かしたら再生は止める。動画プレイヤーのコマ送りと同じ扱い。
        stopPlayback(true);
        return seekTo(FrameNavigation.move(frameIndex, totalFrames, action));
      },
      setPlaying(nextPlaying) {
        if (destroyed || playing === nextPlaying) return;
        if (!nextPlaying) {
          stopPlayback(true);
          return;
        }
        playing = true;
        // 末尾で押したら最初から見直す。
        if (frameIndex >= totalFrames - 1) void seekTo(0);
        void tick();
      },
      setPlaybackRate(rate) {
        playbackRate = rate;
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
      saveCurrentFrameData() {
        if (!destroyed) downloadFrameData(data, frameIndex);
      },
      destroy,
    };
  } catch (cause) {
    destroy();
    throwIfAborted(signal);
    throw cause;
  }
}

const FRAMES_PER_SECOND = 60;

function downloadFrameData(data: DebugViewerData, frameIndex: number): void {
  const json = JSON.stringify(frameDataAt(data, frameIndex), null, 2);
  download(
    new Blob([json], { type: "application/json" }),
    `frame_${String(frameIndex).padStart(6, "0")}.json`,
  );
}

function downloadFrame(canvas: HTMLCanvasElement, frameIndex: number): void {
  canvas.toBlob((blob) => {
    if (!blob) return;
    download(blob, `frame_${String(frameIndex).padStart(6, "0")}.png`);
  }, "image/png");
}

function download(blob: Blob, name: string): void {
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = name;
  anchor.click();
  URL.revokeObjectURL(url);
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new DOMException("認識デバッグを終了しました", "AbortError");
}
