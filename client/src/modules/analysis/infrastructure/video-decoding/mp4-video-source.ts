import {
  createFile,
  DataStream,
  Endianness,
  type ISOFile,
  type Movie,
  type MP4BoxBuffer,
  type Sample,
} from "mp4box";
import type { FrameSample, VideoCodecConfig } from "../../domain/result.js";
import { mp4TimestampUs } from "./sample-timestamp-index.js";

export interface Mp4VideoTrack {
  readonly totalSamples: number;
  readonly decoderConfig: VideoDecoderConfig;
  readonly codecConfig: VideoCodecConfig;
}

export interface Mp4VideoSample {
  readonly metadata: FrameSample;
  readonly chunk: EncodedVideoChunk;
}

interface Mp4VideoSourceCallbacks {
  readonly onTrack: (track: Mp4VideoTrack) => Promise<void>;
  readonly onSamples: (samples: readonly Mp4VideoSample[]) => void;
  readonly onError: (error: unknown) => void;
}

export class Mp4VideoSource {
  readonly #arrayBuffer: ArrayBuffer;
  readonly #callbacks: Mp4VideoSourceCallbacks;
  readonly #file: ISOFile;
  readonly #pendingSamples: Mp4VideoSample[][] = [];
  #trackReady = false;

  constructor(
    arrayBuffer: ArrayBuffer,
    callbacks: Mp4VideoSourceCallbacks,
    file: ISOFile = createFile(),
  ) {
    this.#arrayBuffer = arrayBuffer;
    this.#callbacks = callbacks;
    this.#file = file;
  }

  start(): void {
    this.#file.onReady = (info) => {
      void this.#startVideoTrack(info).catch(this.#callbacks.onError);
    };
    this.#file.onSamples = (_id, _user, samples) => {
      try {
        this.#acceptSamples(samples.map(toVideoSample));
      } catch (error) {
        this.#callbacks.onError(error);
      }
    };
    this.#file.onError = (module, message) => {
      this.#callbacks.onError(new Error(`${module}: ${message}`));
    };

    const input = this.#arrayBuffer as MP4BoxBuffer;
    input.fileStart = 0;
    this.#file.appendBuffer(input);
    this.#file.flush();
  }

  async #startVideoTrack(info: Movie): Promise<void> {
    const track = info.videoTracks[0];
    if (!track?.video) throw new Error("動画トラックが見つかりません");

    const description = extractCodecDescription(this.#file, track.id);
    const trackInitialization = this.#callbacks.onTrack({
      totalSamples: track.nb_samples,
      decoderConfig: {
        codec: track.codec,
        codedWidth: track.video.width,
        codedHeight: track.video.height,
        description,
      },
      codecConfig: {
        codec: track.codec,
        width: track.video.width,
        height: track.video.height,
        description,
      },
    });
    this.#file.setExtractionOptions(track.id, undefined, { nbSamples: 200 });
    this.#file.start();
    await trackInitialization;
    this.#trackReady = true;
    for (const samples of this.#pendingSamples.splice(0)) {
      this.#callbacks.onSamples(samples);
    }
  }

  #acceptSamples(samples: Mp4VideoSample[]): void {
    if (this.#trackReady) {
      this.#callbacks.onSamples(samples);
    } else {
      this.#pendingSamples.push(samples);
    }
  }
}

function toVideoSample(sample: Sample): Mp4VideoSample {
  if (!sample.data) throw new Error("MP4 sample dataがありません");
  const timestampUs = mp4TimestampUs(sample.cts, sample.timescale);
  return {
    metadata: {
      isSync: sample.is_sync,
      timestampUs,
      offset: sample.offset,
      size: sample.size,
    },
    chunk: new EncodedVideoChunk({
      type: sample.is_sync ? "key" : "delta",
      timestamp: timestampUs,
      duration: (sample.duration * 1_000_000) / sample.timescale,
      data: sample.data,
    }),
  };
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
