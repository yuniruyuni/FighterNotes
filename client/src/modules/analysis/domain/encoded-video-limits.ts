export const DEMUX_METADATA_CHUNK_BYTES = 1024 * 1024;
export const MAX_ENCODED_SAMPLE_BYTES = 16 * 1024 * 1024;
export const MAX_ENCODED_BATCH_SAMPLES = 8;
export const MAX_ENCODED_BATCH_BYTES = 16 * 1024 * 1024;
export const MAX_ENCODED_QUEUE_SAMPLES = 16;
export const ENCODED_QUEUE_SAMPLE_LOW_WATERMARK = 8;
export const MAX_ENCODED_QUEUE_BYTES = 32 * 1024 * 1024;
export const ENCODED_QUEUE_BYTE_LOW_WATERMARK = 16 * 1024 * 1024;

// MP4 metadata is read progressively, but MP4Box may retain parsed/input
// buffers. Reject pathological box layouts before their logical ownership can
// grow with the whole file.
export const MAX_DEMUX_METADATA_BYTES = 32 * 1024 * 1024;
export const MAX_DEMUX_METADATA_MP4_BUFFER_BYTES = MAX_DEMUX_METADATA_BYTES;
export const MAX_DEMUX_MEDIA_MP4_BUFFER_BYTES =
  MAX_DEMUX_METADATA_BYTES + MAX_ENCODED_BATCH_BYTES;
export const MAX_DEMUX_MP4_SAMPLE_BYTES = MAX_ENCODED_BATCH_BYTES;
// metadata/current stream + media stream + MP4 sample data + copied batch +
// an input slice not represented by MP4Box's observable buffers.
export const MAX_DEMUX_RETAINED_BYTES =
  MAX_DEMUX_METADATA_BYTES + 4 * MAX_ENCODED_BATCH_BYTES;
