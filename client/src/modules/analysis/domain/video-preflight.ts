import { MAX_ENCODED_SAMPLE_BYTES } from "./encoded-video-limits.js";

export const VIDEO_INPUT_WIDTH = 1920;
export const VIDEO_INPUT_HEIGHT = 1080;
export const VIDEO_INPUT_MIN_FPS = 59;
export const VIDEO_INPUT_MAX_FPS = 61;

export type VideoRotation = 0 | 90 | 180 | 270;

export type VideoPreflightFailureCode =
  | "non_mp4"
  | "missing_video_track"
  | "fragmented_mp4"
  | "timing_unavailable"
  | "rotation"
  | "dimensions"
  | "frame_rate"
  | "variable_frame_rate"
  | "metadata_size"
  | "encoded_sample_size"
  | "unsupported_codec"
  | "frame_extraction"
  | "invalid_mp4";

export interface VideoFileIdentity {
  readonly name: string;
  readonly size: number;
  readonly lastModified: number;
  readonly type: string;
}

export interface InspectedVideoTrack {
  readonly trackId: number;
  readonly codec: string;
  readonly codedWidth: number;
  readonly codedHeight: number;
  readonly displayWidth: number;
  readonly displayHeight: number;
  readonly rotation: VideoRotation | null;
  readonly framesPerSecond: number;
  readonly constantFrameRate: boolean;
  readonly totalSamples: number;
  readonly maxSampleBytes: number;
  readonly timescale: number;
  readonly duration: number;
  readonly decoderConfig: VideoDecoderConfig;
  readonly codecConfig: {
    readonly codec: string;
    readonly width: number;
    readonly height: number;
    readonly description?: Uint8Array;
  };
}

export interface InspectedVideo {
  readonly container: "mp4" | "other";
  readonly fragmented: boolean;
  readonly metadataBytesRead: number;
  readonly track: InspectedVideoTrack | null;
}

export interface ValidatedVideoInput {
  readonly file: File;
  readonly identity: VideoFileIdentity;
  readonly track: InspectedVideoTrack;
  readonly metadataBytesRead: number;
}

export interface VideoPreflightFailure {
  readonly status: "invalid";
  readonly code: VideoPreflightFailureCode;
  readonly message: string;
}

export type VideoPreflightResult =
  | {
      readonly status: "valid";
      readonly video: ValidatedVideoInput;
    }
  | VideoPreflightFailure;

export type VideoPreflightState =
  | { readonly status: "idle" }
  | { readonly status: "checking" }
  | VideoPreflightResult;

export function videoFileIdentity(file: File): VideoFileIdentity {
  return {
    name: file.name,
    size: file.size,
    lastModified: file.lastModified,
    type: file.type,
  };
}

export function matchesValidatedVideoFile(
  file: File,
  validated: ValidatedVideoInput,
): boolean {
  if (validated.file !== file) return false;
  const identity = videoFileIdentity(file);
  return (
    identity.name === validated.identity.name &&
    identity.size === validated.identity.size &&
    identity.lastModified === validated.identity.lastModified &&
    identity.type === validated.identity.type
  );
}

export function validateInspectedVideo(
  file: File,
  inspected: InspectedVideo,
): VideoPreflightResult {
  if (inspected.container !== "mp4") {
    return failure(
      "non_mp4",
      "MP4形式の動画を選択してください。OBSでは録画形式をMP4にするか、録画後にMP4へ再多重化してください。",
    );
  }
  if (!inspected.track) {
    return failure(
      "missing_video_track",
      "MP4内に解析できる映像トラックがありません。映像を含むMP4を書き出してください。",
    );
  }
  if (inspected.fragmented) {
    return failure(
      "fragmented_mp4",
      "分割MP4では固定フレームレートを事前確認できません。通常のMP4へ再多重化してから選択してください。",
    );
  }
  const track = inspected.track;
  if (
    track.totalSamples < 2 ||
    track.timescale <= 0 ||
    track.duration <= 0 ||
    !Number.isFinite(track.framesPerSecond)
  ) {
    return failure(
      "timing_unavailable",
      "動画のフレーム時刻を確認できません。固定60fps（CFR）のMP4として書き出し直してください。",
    );
  }
  if (track.rotation !== 0) {
    const detected =
      track.rotation === null ? "非標準の変形" : `${track.rotation}°`;
    return failure(
      "rotation",
      `回転メタデータ（${detected}）付きの動画には対応していません。横向き1920×1080・回転なしで書き出してください。`,
    );
  }
  if (
    track.codedWidth !== VIDEO_INPUT_WIDTH ||
    track.codedHeight !== VIDEO_INPUT_HEIGHT ||
    track.displayWidth !== VIDEO_INPUT_WIDTH ||
    track.displayHeight !== VIDEO_INPUT_HEIGHT
  ) {
    return failure(
      "dimensions",
      `動画は1920×1080で書き出してください（検出: coded ${formatDimensions(track.codedWidth, track.codedHeight)} / 表示 ${formatDimensions(track.displayWidth, track.displayHeight)}）。クロップ・拡大・黒帯追加も外してください。`,
    );
  }
  if (
    track.framesPerSecond < VIDEO_INPUT_MIN_FPS ||
    track.framesPerSecond > VIDEO_INPUT_MAX_FPS
  ) {
    return failure(
      "frame_rate",
      `フレームレートは固定60fps付近にしてください（検出: ${formatFrameRate(track.framesPerSecond)}fps）。OBSの「FPS共通値」を60に設定して録画し直してください。`,
    );
  }
  if (!track.constantFrameRate) {
    return failure(
      "variable_frame_rate",
      "可変フレームレート（VFR）を検出しました。OBSなどで固定60fps（CFR）を指定して録画し直してください。",
    );
  }
  if (
    !Number.isSafeInteger(track.maxSampleBytes) ||
    track.maxSampleBytes <= 0 ||
    track.maxSampleBytes > MAX_ENCODED_SAMPLE_BYTES
  ) {
    return failure(
      "encoded_sample_size",
      `動画内の圧縮フレームが大きすぎるため解析できません。映像品質またはビットレートを下げ、1フレームを${formatMegabytes(MAX_ENCODED_SAMPLE_BYTES)}MiB以下にしてMP4を再エンコードしてください。`,
    );
  }
  return {
    status: "valid",
    video: {
      file,
      identity: videoFileIdentity(file),
      track,
      metadataBytesRead: inspected.metadataBytesRead,
    },
  };
}

export function videoPreflightFailure(
  code: VideoPreflightFailureCode,
  message: string,
): VideoPreflightFailure {
  return failure(code, message);
}

function failure(
  code: VideoPreflightFailureCode,
  message: string,
): VideoPreflightFailure {
  return { status: "invalid", code, message };
}

function formatDimensions(width: number, height: number): string {
  return `${formatInteger(width)}×${formatInteger(height)}`;
}

function formatInteger(value: number): string {
  return Number.isFinite(value) ? String(Math.round(value)) : "不明";
}

// 非有限の frame rate は timing_unavailable で先に弾いているため、
// ここへ到達する値は必ず有限。
function formatFrameRate(value: number): string {
  return value.toFixed(2);
}

function formatMegabytes(bytes: number): string {
  return String(bytes / (1024 * 1024));
}
