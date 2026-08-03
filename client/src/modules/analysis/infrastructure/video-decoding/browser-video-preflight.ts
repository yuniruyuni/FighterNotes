import {
  type InspectedVideo,
  type VideoPreflightResult,
  validateInspectedVideo,
  videoPreflightFailure,
} from "../../domain/video-preflight.js";
import {
  inspectMp4VideoFile,
  Mp4InspectionError,
} from "./mp4-video-preflight.js";
import type { VideoDecoderSupportChecker } from "./webcodecs-support.js";

type VideoInspector = (
  file: File,
  signal: AbortSignal,
) => Promise<InspectedVideo>;

interface BrowserVideoPreflightDependencies {
  readonly inspect?: VideoInspector;
  readonly checkDecoder?: VideoDecoderSupportChecker;
  readonly probeFrameBitmap?: (signal: AbortSignal) => Promise<boolean>;
}

interface VideoFrameBitmapProbeDependencies {
  readonly createCanvas?: () => OffscreenCanvas;
  readonly createFrame?: (canvas: OffscreenCanvas) => VideoFrame;
  readonly createBitmap?: (frame: VideoFrame) => Promise<ImageBitmap>;
}

export async function preflightBrowserVideo(
  file: File,
  signal: AbortSignal,
  dependencies: BrowserVideoPreflightDependencies = {},
): Promise<VideoPreflightResult> {
  throwIfAborted(signal);
  let inspected: InspectedVideo;
  try {
    inspected = await (dependencies.inspect ?? inspectMp4VideoFile)(
      file,
      signal,
    );
  } catch (error) {
    throwIfAborted(signal);
    if (error instanceof Mp4InspectionError && error.code === "non_mp4") {
      return videoPreflightFailure(
        "non_mp4",
        "MP4形式の動画を選択してください。OBSでは録画形式をMP4にするか、録画後にMP4へ再多重化してください。",
      );
    }
    if (error instanceof Mp4InspectionError && error.code === "metadata_size") {
      return videoPreflightFailure(
        "metadata_size",
        "MP4の動画情報が大きすぎるため解析できません。通常のMP4へ再多重化するか、H.264のMP4へ再エンコードしてください。",
      );
    }
    return videoPreflightFailure(
      "invalid_mp4",
      `MP4の動画情報を読み取れませんでした。ファイルが破損していないか確認し、MP4として書き出し直してください。${errorMessageSuffix(error)}`,
    );
  }
  throwIfAborted(signal);
  const validation = validateInspectedVideo(file, inspected);
  if (validation.status === "invalid") return validation;

  try {
    const support = await (
      dependencies.checkDecoder ??
      ((config) => VideoDecoder.isConfigSupported(config))
    )(validation.video.track.decoderConfig);
    throwIfAborted(signal);
    if (!support.supported) {
      return videoPreflightFailure(
        "unsupported_codec",
        `このブラウザでは動画のコーデック（${validation.video.track.codec}）をデコードできません。H.264のMP4で書き出すか、対応ブラウザを使用してください。`,
      );
    }
  } catch (error) {
    throwIfAborted(signal);
    return videoPreflightFailure(
      "unsupported_codec",
      `動画コーデックの対応状況を確認できませんでした。ブラウザを最新版に更新するか、H.264のMP4で書き出してください。${errorMessageSuffix(error)}`,
    );
  }

  try {
    const supported = await (
      dependencies.probeFrameBitmap ?? probeVideoFrameBitmap
    )(signal);
    throwIfAborted(signal);
    if (!supported) {
      return videoPreflightFailure(
        "frame_extraction",
        "このブラウザではVideoFrameから解析画像を切り出せません。ブラウザを最新版に更新するか、対応ブラウザで開いてください。",
      );
    }
  } catch (error) {
    throwIfAborted(signal);
    return videoPreflightFailure(
      "frame_extraction",
      `VideoFrameから解析画像を切り出せません。ブラウザを最新版に更新するか、対応ブラウザで開いてください。${errorMessageSuffix(error)}`,
    );
  }
  return validation;
}

export async function probeVideoFrameBitmap(
  signal: AbortSignal,
  dependencies: VideoFrameBitmapProbeDependencies = {},
): Promise<boolean> {
  throwIfAborted(signal);
  const canvas = (
    dependencies.createCanvas ?? (() => new OffscreenCanvas(2, 2))
  )();
  if (!canvas.getContext("2d")) return false;
  const frame = (
    dependencies.createFrame ??
    ((source) => new VideoFrame(source, { timestamp: 0 }))
  )(canvas);
  let bitmap: ImageBitmap | undefined;
  try {
    bitmap = await (
      dependencies.createBitmap ?? ((source) => createImageBitmap(source))
    )(frame);
    throwIfAborted(signal);
    return bitmap.width > 0 && bitmap.height > 0;
  } finally {
    bitmap?.close();
    frame.close();
  }
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new DOMException("動画確認を中止しました", "AbortError");
}

function errorMessageSuffix(error: unknown): string {
  const message = error instanceof Error ? error.message.trim() : "";
  return message ? `（詳細: ${message}）` : "";
}
