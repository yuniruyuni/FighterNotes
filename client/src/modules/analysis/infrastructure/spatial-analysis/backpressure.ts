export const SPATIAL_DECODER_QUEUE_WATERMARKS = {
  high: 12,
  low: 6,
} as const;

export const SPATIAL_DECODER_OUTSTANDING_WATERMARKS = {
  high: 12,
  low: 6,
} as const;

export const SPATIAL_WORKER_PENDING_WATERMARKS = {
  high: 12,
  low: 6,
} as const;

export interface SpatialDecodeStats {
  readonly peakDecoderQueueSize: number;
  readonly peakDecoderOutstandingFrames: number;
}

export interface SpatialPerformanceStats extends SpatialDecodeStats {
  readonly frameCount: number;
  readonly decoderQueueHighWatermark: number;
  readonly decoderQueueLowWatermark: number;
  readonly decoderOutstandingHighWatermark: number;
  readonly decoderOutstandingLowWatermark: number;
  readonly workerPendingHighWatermark: number;
  readonly workerPendingLowWatermark: number;
  readonly peakWorkerPendingFrames: number;
}

export const EMPTY_SPATIAL_DECODE_STATS: SpatialDecodeStats = {
  peakDecoderQueueSize: 0,
  peakDecoderOutstandingFrames: 0,
};
