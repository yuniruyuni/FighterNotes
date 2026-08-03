import type { Mp4VideoSourceStats } from "../video-decoding/mp4-video-source.js";
import type { DecodePumpStats } from "./decode-pump.js";

export interface AnalysisTiming {
  readonly frameIndex: number;
  readonly tDraw: number;
  readonly tCopy: number;
  readonly tMeter: number;
  readonly tHud: number;
  readonly streaming?: {
    readonly videoBytes: number;
    readonly preflightMetadataBytes: number;
    readonly demux: Mp4VideoSourceStats;
    readonly encodedQueue: DecodePumpStats;
  };
}

export function logPerformance(timing: AnalysisTiming): void {
  if (timing.frameIndex <= 0) return;
  const ms = (value: number) => `${value.toFixed(0)}ms`;
  console.log(
    `[perf] ${timing.frameIndex}f total:` +
      ` draw+get=${ms(timing.tDraw)} (${ms(timing.tDraw / timing.frameIndex)}/f)` +
      ` worker_copy=${ms(timing.tCopy)} (${ms(timing.tCopy / timing.frameIndex)}/f)` +
      ` meter=${ms(timing.tMeter)} (${ms(timing.tMeter / timing.frameIndex)}/f)` +
      ` hud=${ms(timing.tHud)} (${ms(timing.tHud / timing.frameIndex)}/f)`,
  );
  if (timing.streaming) {
    const { videoBytes, preflightMetadataBytes, demux, encodedQueue } =
      timing.streaming;
    console.log(
      "[perf] streaming" +
        ` video=${videoBytes}B` +
        ` preflight_read=${preflightMetadataBytes}B` +
        ` demux_read=${demux.totalBytesRead}B` +
        ` demux_reads=${demux.readCount}` +
        ` demux_chunk=${demux.chunkBytes}B` +
        ` demux_metadata_read=${demux.metadataBytesRead}B` +
        ` demux_media_read=${demux.mediaBytesRead}B` +
        ` metadata_ranges=${demux.metadataSparseRangeCount}` +
        ` metadata_range_ops=${demux.metadataSparseRangeOperations}` +
        ` sample_max=${demux.maxEncodedSampleBytes}B` +
        ` observed_sample_max=${demux.observedMaxSampleBytes}B` +
        ` batch_samples_max=${demux.extractionBatchSamples}` +
        ` batch_bytes_max=${demux.maxExtractionBatchBytes}B` +
        ` metadata_bytes_max=${demux.maxMetadataBytes}B` +
        ` metadata_mp4_buffer_max=${demux.maxMetadataMp4BufferBytes}B` +
        ` media_mp4_buffer_max=${demux.maxMediaMp4BufferBytes}B` +
        ` mp4_sample_bytes_max=${demux.maxMp4SampleBytes}B` +
        ` demux_retained_max=${demux.maxDemuxRetainedBytes}B` +
        ` peak_blob=${demux.peakReadBytes}B` +
        ` peak_metadata_blob=${demux.peakMetadataReadBytes}B` +
        ` peak_media_blob=${demux.peakMediaReadBytes}B` +
        ` peak_mp4_buffer=${demux.peakMp4BufferBytes}B` +
        ` peak_metadata_mp4_buffer=${demux.peakMetadataMp4BufferBytes}B` +
        ` peak_media_mp4_buffer=${demux.peakMediaMp4BufferBytes}B` +
        ` peak_mp4_sample=${demux.peakMp4SampleBytes}B` +
        ` peak_demux_retained=${demux.peakDemuxRetainedBytes}B` +
        ` peak_batch_samples=${demux.peakBatchSamples}` +
        ` peak_batch=${demux.peakBatchBytes}B` +
        ` delivered_samples=${demux.deliveredSamples}` +
        ` released_samples=${demux.releasedSamples}` +
        ` peak_encoded_samples=${encodedQueue.peakQueuedSamples}` +
        ` peak_encoded_queue=${encodedQueue.peakQueuedBytes}B` +
        ` encoded_samples_high=${encodedQueue.maxQueuedSamples}` +
        ` encoded_samples_low=${encodedQueue.queuedSampleLowWatermark}` +
        ` encoded_bytes_high=${encodedQueue.maxQueuedBytes}B` +
        ` encoded_bytes_low=${encodedQueue.queuedByteLowWatermark}B` +
        ` demux_first_sample=${(demux.timeToFirstSampleMs ?? 0).toFixed(0)}ms`,
    );
  }
}
