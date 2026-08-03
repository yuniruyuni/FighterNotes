import {
  createFile,
  DataStream,
  Endianness,
  type ISOFile,
  type Movie,
  type MP4BoxBuffer,
  type Sample,
} from "mp4box";
import {
  DEMUX_METADATA_CHUNK_BYTES,
  MAX_DEMUX_METADATA_BYTES,
} from "../../domain/encoded-video-limits.js";
import type {
  InspectedVideo,
  InspectedVideoTrack,
  VideoRotation,
} from "../../domain/video-preflight.js";

const ROTATION_SCALE = 1 << 16;
const MATRIX_TOLERANCE = 1 / ROTATION_SCALE;
const PERSPECTIVE_SCALE = 2 ** 30;
const MINIMUM_QUANTIZED_TICKS_PER_FRAME = 2;
const SUPPORTED_FRAME_CADENCES = [
  { numerator: 60, denominator: 1 },
  { numerator: 60_000, denominator: 1001 },
] as const;
const MP4_MAJOR_BRANDS = new Set([
  "isom",
  "iso2",
  "iso3",
  "iso4",
  "iso5",
  "iso6",
  "iso7",
  "iso8",
  "iso9",
  "mp41",
  "mp42",
  "avc1",
  "m4v",
  "msnv",
  "cmfc",
  "cmfs",
  "cmfl",
  "cmff",
  "dash",
]);

export class Mp4InspectionError extends Error {
  readonly code: "non_mp4" | "invalid_mp4" | "metadata_size";

  constructor(
    code: "non_mp4" | "invalid_mp4" | "metadata_size",
    message: string,
  ) {
    super(message);
    this.name = "Mp4InspectionError";
    this.code = code;
  }
}

interface Mp4MetadataReaderDependencies {
  readonly createIsoFile?: () => ISOFile;
  readonly chunkBytes?: number;
  /** Test seam; production always uses the shared 32 MiB metadata cap. */
  readonly maxMetadataBytes?: number;
}

export async function inspectMp4VideoFile(
  source: File,
  signal: AbortSignal,
  dependencies: Mp4MetadataReaderDependencies = {},
): Promise<InspectedVideo> {
  if (!hasMp4SelectionHint(source)) {
    throw new Mp4InspectionError(
      "non_mp4",
      "選択されたファイルはMP4形式ではありません。",
    );
  }
  const file = dependencies.createIsoFile?.() ?? createFile(false);
  const chunkBytes = dependencies.chunkBytes ?? DEMUX_METADATA_CHUNK_BYTES;
  const maxMetadataBytes =
    dependencies.maxMetadataBytes ?? MAX_DEMUX_METADATA_BYTES;
  if (!Number.isSafeInteger(chunkBytes) || chunkBytes <= 0) {
    throw new Error("MP4 metadata chunk size must be a positive integer");
  }
  if (!Number.isSafeInteger(maxMetadataBytes) || maxMetadataBytes <= 0) {
    throw new Error("MP4 metadata byte limit must be a positive integer");
  }

  let movie: Movie | undefined;
  let parserError: Error | undefined;
  let metadataBytesRead = 0;
  file.onReady = (info) => {
    movie = info;
  };
  file.onError = (module, message) => {
    parserError = new Error(`${module}: ${message}`);
  };

  try {
    let offset = 0;
    while (!movie && offset < source.size) {
      throwIfAborted(signal);
      const remainingMetadataBytes = maxMetadataBytes - metadataBytesRead;
      if (remainingMetadataBytes <= 0) {
        throw metadataSizeError(maxMetadataBytes);
      }
      const end = Math.min(
        source.size,
        offset + chunkBytes,
        offset + remainingMetadataBytes,
      );
      const buffer = (await source
        .slice(offset, end)
        .arrayBuffer()) as MP4BoxBuffer;
      throwIfAborted(signal);
      buffer.fileStart = offset;
      metadataBytesRead += buffer.byteLength;
      let suggestedOffset: number;
      try {
        suggestedOffset = file.appendBuffer(buffer, end === source.size);
      } catch (error) {
        parserError = toError(error);
        break;
      }
      if (movie) break;
      offset = nextMetadataOffset(offset, end, suggestedOffset, source.size);
    }

    if (!movie && !parserError) {
      try {
        file.flush();
      } catch (error) {
        parserError = toError(error);
      }
    }
    throwIfAborted(signal);
    if (!movie) {
      const sawFileType = Boolean((file as ISOFile & { ftyp?: unknown }).ftyp);
      throw new Mp4InspectionError(
        sawFileType ? "invalid_mp4" : "non_mp4",
        parserError?.message ?? "MP4 metadataを読み取れませんでした。",
      );
    }
    return inspectMovie(file, movie, metadataBytesRead);
  } finally {
    file.stop();
  }
}

function metadataSizeError(limit: number): Mp4InspectionError {
  return new Mp4InspectionError(
    "metadata_size",
    `MP4の動画情報が${limit / (1024 * 1024)}MiBを超えています。通常のMP4へ再多重化するか、動画を再エンコードしてください。`,
  );
}

export function inspectMovie(
  file: ISOFile,
  movie: Movie,
  metadataBytesRead: number,
): InspectedVideo {
  const selectedTrack = movie.videoTracks[0];
  return {
    container: isMp4Container(movie.brands) ? "mp4" : "other",
    fragmented: movie.isFragmented,
    metadataBytesRead,
    track: selectedTrack ? inspectVideoTrack(file, selectedTrack) : null,
  };
}

export function rotationFromTrackMatrix(
  matrix: ArrayLike<number> | undefined,
): VideoRotation | null {
  if (!matrix || matrix.length < 9) return null;
  if (
    Math.abs(matrix[2]) > 1 ||
    Math.abs(matrix[5]) > 1 ||
    Math.abs(matrix[8] - PERSPECTIVE_SCALE) > 1
  ) {
    return null;
  }
  const a = matrix[0] / ROTATION_SCALE;
  const b = matrix[1] / ROTATION_SCALE;
  const c = matrix[3] / ROTATION_SCALE;
  const d = matrix[4] / ROTATION_SCALE;
  const candidates: ReadonlyArray<{
    rotation: VideoRotation;
    values: readonly [number, number, number, number];
  }> = [
    { rotation: 0, values: [1, 0, 0, 1] },
    { rotation: 90, values: [0, 1, -1, 0] },
    { rotation: 180, values: [-1, 0, 0, -1] },
    { rotation: 270, values: [0, -1, 1, 0] },
  ];
  return (
    candidates.find(({ values }) =>
      [a, b, c, d].every(
        (value, index) => Math.abs(value - values[index]) <= MATRIX_TOLERANCE,
      ),
    )?.rotation ?? null
  );
}

export function summarizeFrameTiming(
  samples: readonly Pick<Sample, "cts">[],
  timescale: number,
  expectedSamples: number,
): { readonly framesPerSecond: number; readonly constantFrameRate: boolean } {
  if (
    !Number.isFinite(timescale) ||
    timescale <= 0 ||
    expectedSamples < 2 ||
    samples.length !== expectedSamples
  ) {
    return { framesPerSecond: Number.NaN, constantFrameRate: false };
  }
  const timestamps = samples.map(({ cts }) => cts).sort((a, b) => a - b);
  const firstTimestamp = timestamps[0];
  if (!Number.isSafeInteger(firstTimestamp)) {
    return { framesPerSecond: Number.NaN, constantFrameRate: false };
  }
  const cadenceStates = SUPPORTED_FRAME_CADENCES.flatMap(
    ({ numerator, denominator }) => {
      const ticksPerFrame = (timescale * denominator) / numerator;
      const quantized = !Number.isInteger(ticksPerFrame);
      if (quantized && ticksPerFrame < MINIMUM_QUANTIZED_TICKS_PER_FRAME) {
        return [];
      }
      return [
        {
          ticksPerFrame,
          floor: true,
          round: true,
          ceil: true,
        },
      ];
    },
  );
  let previousTimestamp = firstTimestamp;
  for (let index = 1; index < timestamps.length; index += 1) {
    const timestamp = timestamps[index];
    const delta = timestamp - previousTimestamp;
    if (!Number.isSafeInteger(timestamp) || delta <= 0) {
      return { framesPerSecond: Number.NaN, constantFrameRate: false };
    }
    const elapsed = timestamp - firstTimestamp;
    for (const state of cadenceStates) {
      const expected = index * state.ticksPerFrame;
      const tolerance = floatingPointTolerance(expected);
      state.floor &&= elapsed === Math.floor(expected + tolerance);
      state.round &&= elapsed === Math.floor(expected + 0.5 + tolerance);
      state.ceil &&= elapsed === Math.ceil(expected - tolerance);
    }
    previousTimestamp = timestamp;
  }
  const totalDelta = previousTimestamp - firstTimestamp;
  return {
    framesPerSecond: (timescale * (timestamps.length - 1)) / totalDelta,
    // Require every CTS to follow one consistent integer quantization of a
    // supported 60fps cadence. Merely allowing adjacent one-tick deltas would
    // misclassify a dropped frame as CFR.
    constantFrameRate: cadenceStates.some(
      ({ floor, round, ceil }) => floor || round || ceil,
    ),
  };
}

function inspectVideoTrack(
  file: ISOFile,
  track: Movie["videoTracks"][number],
): InspectedVideoTrack {
  const samples = file.getTrackSamplesInfo(track.id) ?? [];
  const codedWidth = track.video?.width ?? Number.NaN;
  const codedHeight = track.video?.height ?? Number.NaN;
  const rotation = rotationFromTrackMatrix(track.matrix);
  const trackWidth = positiveDimension(track.track_width, codedWidth);
  const trackHeight = positiveDimension(track.track_height, codedHeight);
  const swapsDimensions = rotation === 90 || rotation === 270;
  const timing = summarizeFrameTiming(
    samples,
    track.timescale,
    track.nb_samples,
  );
  const description = extractCodecDescription(file, track.id);
  return {
    trackId: track.id,
    codec: track.codec,
    codedWidth,
    codedHeight,
    displayWidth: swapsDimensions ? trackHeight : trackWidth,
    displayHeight: swapsDimensions ? trackWidth : trackHeight,
    rotation,
    ...timing,
    totalSamples: track.nb_samples,
    maxSampleBytes: maximumSampleBytes(samples),
    timescale: track.timescale,
    duration: track.duration,
    decoderConfig: {
      codec: track.codec,
      codedWidth,
      codedHeight,
      description,
    },
    codecConfig: {
      codec: track.codec,
      width: codedWidth,
      height: codedHeight,
      description,
    },
  };
}

function maximumSampleBytes(samples: readonly Sample[]): number {
  let maximum = 0;
  for (const sample of samples) {
    if (!Number.isSafeInteger(sample.size) || sample.size <= 0) {
      return Number.NaN;
    }
    maximum = Math.max(maximum, sample.size);
  }
  return maximum;
}

function extractCodecDescription(
  file: ISOFile,
  trackId: number,
): Uint8Array | undefined {
  const entry =
    file.getTrackById(trackId)?.mdia?.minf?.stbl?.stsd?.entries?.[0];
  const boxes = entry as
    | {
        avcC?: { write(stream: DataStream): void };
        hvcC?: { write(stream: DataStream): void };
        av1C?: { write(stream: DataStream): void };
      }
    | undefined;
  const box = boxes?.avcC ?? boxes?.hvcC ?? boxes?.av1C;
  if (!box) return undefined;
  const stream = new DataStream(undefined, 0, Endianness.BIG_ENDIAN);
  box.write(stream);
  return new Uint8Array(stream.buffer, 8);
}

function hasMp4SelectionHint(file: File): boolean {
  const mime = file.type.trim().toLowerCase();
  if (mime === "video/mp4" || mime === "application/mp4") return true;
  return file.name.toLowerCase().endsWith(".mp4");
}

function isMp4Container(brands: readonly string[]): boolean {
  const majorBrand = brands[0]?.trim().toLowerCase();
  return majorBrand !== undefined && MP4_MAJOR_BRANDS.has(majorBrand);
}

function floatingPointTolerance(value: number): number {
  return Math.min(
    0.25,
    Math.max(1e-9, Number.EPSILON * Math.max(1, Math.abs(value)) * 8),
  );
}

function positiveDimension(value: number, fallback: number): number {
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

function nextMetadataOffset(
  start: number,
  end: number,
  suggested: number,
  fileSize: number,
): number {
  if (Number.isFinite(suggested) && suggested >= end) {
    return Math.min(suggested, fileSize);
  }
  return Math.max(end, start + 1);
}

function throwIfAborted(signal: AbortSignal): void {
  if (!signal.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new DOMException("動画確認を中止しました", "AbortError");
}

function toError(error: unknown): Error {
  return error instanceof Error ? error : new Error(String(error));
}
