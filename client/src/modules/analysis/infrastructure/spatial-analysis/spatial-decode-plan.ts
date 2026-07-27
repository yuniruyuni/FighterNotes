import type {
  FrameSample,
  SpatialCandidateWindow,
  SpatialFrameHints,
  SpatialHintRange,
} from "../../domain/result.js";
import { precedingKeyframeIndex } from "../video-decoding/frame-decode-plan.js";

interface SpatialDecodeTarget {
  readonly timestampUs: number;
  readonly frameIndex: number;
}

export interface SpatialDecodePlan {
  readonly firstSampleIndex: number;
  readonly lastSampleIndex: number;
  readonly targets: readonly SpatialDecodeTarget[];
}

export namespace SpatialDecodePlan {
  export function create(
    window: SpatialCandidateWindow,
    samples: readonly FrameSample[],
    frameToSampleIndex: readonly number[],
  ): SpatialDecodePlan | null {
    const targets: SpatialDecodeTarget[] = [];
    let firstSampleIndex = Number.POSITIVE_INFINITY;
    let lastSampleIndex = -1;
    for (
      let frameIndex = window.start_frame;
      frameIndex <= window.end_frame;
      frameIndex += 1
    ) {
      const sampleIndex = frameToSampleIndex[frameIndex] ?? -1;
      const sample = samples[sampleIndex];
      if (sampleIndex < 0 || !sample) continue;
      targets.push({ timestampUs: sample.timestampUs, frameIndex });
      firstSampleIndex = Math.min(firstSampleIndex, sampleIndex);
      lastSampleIndex = Math.max(lastSampleIndex, sampleIndex);
    }
    if (!Number.isFinite(firstSampleIndex) || lastSampleIndex < 0) return null;

    return {
      firstSampleIndex: precedingKeyframeIndex(samples, firstSampleIndex),
      lastSampleIndex,
      targets,
    };
  }
}

export function spatialHintsAt(
  window: SpatialCandidateWindow,
  frameIndex: number,
): SpatialFrameHints {
  const active = (ranges: readonly SpatialHintRange[], side: number) =>
    ranges.some(
      (range) =>
        range.side === side &&
        frameIndex >= range.start_frame &&
        frameIndex <= range.end_frame,
    );
  return {
    p1Teleport: active(window.teleport_hints, 1),
    p2Teleport: active(window.teleport_hints, 2),
    p1Airborne: active(window.airborne_hints, 1),
    p2Airborne: active(window.airborne_hints, 2),
  };
}
