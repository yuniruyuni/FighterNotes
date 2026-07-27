import type { FrameSample } from "../../domain/result.js";

export interface FrameDecodePlan {
  readonly firstSampleIndex: number;
  readonly lastSampleIndex: number;
  readonly targetTimestampUs: number;
}

export namespace FrameDecodePlan {
  export function create(
    samples: readonly FrameSample[],
    frameToSampleIndex: readonly number[] | null,
    frameIndex: number,
  ): FrameDecodePlan | null {
    const sampleIndex = frameToSampleIndex
      ? (frameToSampleIndex[frameIndex] ?? -1)
      : frameIndex;
    const sample = samples[sampleIndex];
    if (sampleIndex < 0 || !sample) return null;

    return {
      firstSampleIndex: precedingKeyframeIndex(samples, sampleIndex),
      lastSampleIndex: sampleIndex,
      targetTimestampUs: sample.timestampUs,
    };
  }
}

export function precedingKeyframeIndex(
  samples: readonly FrameSample[],
  sampleIndex: number,
): number {
  let index = sampleIndex;
  while (index > 0 && !samples[index]?.isSync) index -= 1;
  return index;
}
