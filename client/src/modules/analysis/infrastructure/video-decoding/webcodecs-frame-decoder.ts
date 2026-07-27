import type { FrameSample, VideoCodecConfig } from "../../domain/result.js";
import { FrameDecodePlan } from "./frame-decode-plan.js";

interface DecodeSampleRangeOptions {
  readonly samples: readonly FrameSample[];
  readonly videoArrayBuffer: ArrayBuffer;
  readonly codecConfig: VideoCodecConfig;
  readonly firstSampleIndex: number;
  readonly lastSampleIndex: number;
  readonly onFrame: (frame: VideoFrame) => void;
  readonly signal?: AbortSignal;
}

export async function decodeSampleRange(
  options: DecodeSampleRangeOptions,
): Promise<void> {
  throwIfAborted(options.signal);
  const decoder = new VideoDecoder({
    output: options.onFrame,
    error: () => undefined,
  });
  const onAbort = () => {
    if (decoder.state !== "closed") decoder.close();
  };
  options.signal?.addEventListener("abort", onAbort, { once: true });
  try {
    decoder.configure({
      codec: options.codecConfig.codec,
      codedWidth: options.codecConfig.width,
      codedHeight: options.codecConfig.height,
      description: options.codecConfig.description,
    });
    for (
      let sampleIndex = options.firstSampleIndex;
      sampleIndex <= options.lastSampleIndex;
      sampleIndex += 1
    ) {
      throwIfAborted(options.signal);
      const sample = options.samples[sampleIndex];
      if (!sample) continue;
      decoder.decode(encodedChunk(sample, options.videoArrayBuffer));
    }
    await decoder.flush();
    throwIfAborted(options.signal);
  } catch (error) {
    throwIfAborted(options.signal);
    throw error;
  } finally {
    options.signal?.removeEventListener("abort", onAbort);
    if (decoder.state !== "closed") decoder.close();
  }
}

export async function decodeFrameAt(options: {
  readonly samples: readonly FrameSample[];
  readonly videoArrayBuffer: ArrayBuffer;
  readonly codecConfig: VideoCodecConfig;
  readonly frameToSampleIndex: readonly number[] | null;
  readonly frameIndex: number;
}): Promise<VideoFrame | null> {
  const plan = FrameDecodePlan.create(
    options.samples,
    options.frameToSampleIndex,
    options.frameIndex,
  );
  if (!plan) return null;

  let result: VideoFrame | null = null;
  await decodeSampleRange({
    ...options,
    firstSampleIndex: plan.firstSampleIndex,
    lastSampleIndex: plan.lastSampleIndex,
    onFrame(frame) {
      if (!result && frame.timestamp === plan.targetTimestampUs) {
        result = frame;
      } else {
        frame.close();
      }
    },
  });
  return result;
}

function encodedChunk(
  sample: FrameSample,
  videoArrayBuffer: ArrayBuffer,
): EncodedVideoChunk {
  return new EncodedVideoChunk({
    type: sample.isSync ? "key" : "delta",
    timestamp: sample.timestampUs,
    data: new Uint8Array(videoArrayBuffer, sample.offset, sample.size),
  });
}

function throwIfAborted(signal?: AbortSignal): void {
  if (!signal?.aborted) return;
  throw signal.reason instanceof Error
    ? signal.reason
    : new Error("動画のデコードを中断しました");
}
