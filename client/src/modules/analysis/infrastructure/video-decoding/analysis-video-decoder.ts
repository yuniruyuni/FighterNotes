import type { Mp4VideoTrack } from "./mp4-video-source.js";
import { requireSupportedVideoDecoder } from "./webcodecs-support.js";

interface AnalysisVideoDecoderOptions {
  readonly onFrame: (frame: VideoFrame) => void;
  readonly onDequeue: () => void;
  readonly onError: (error: unknown) => void;
  readonly signal: AbortSignal;
}

export async function createAnalysisVideoDecoder(
  track: Mp4VideoTrack,
  options: AnalysisVideoDecoderOptions,
): Promise<VideoDecoder> {
  await requireSupportedVideoDecoder(track.decoderConfig);
  if (options.signal.aborted) throw options.signal.reason;

  const decoder = new VideoDecoder({
    output: options.onFrame,
    error: options.onError,
  });
  decoder.configure(track.decoderConfig);
  decoder.addEventListener("dequeue", options.onDequeue);
  return decoder;
}
