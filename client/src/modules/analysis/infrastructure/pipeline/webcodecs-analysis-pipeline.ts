import type { AnalysisContext } from "../../domain/context.js";
import {
  ENCODED_QUEUE_BYTE_LOW_WATERMARK,
  ENCODED_QUEUE_SAMPLE_LOW_WATERMARK,
  MAX_ENCODED_QUEUE_BYTES,
  MAX_ENCODED_QUEUE_SAMPLES,
} from "../../domain/encoded-video-limits.js";
import type {
  AnalysisProgress,
  AnalysisResult,
  FrameSample as FrameSampleData,
  VideoCodecConfig,
} from "../../domain/result.js";
import type { ValidatedVideoInput } from "../../domain/video-preflight.js";
import {
  CopyStripExtractor,
  preferCopyExtraction,
  supportsRgbaCopy,
} from "../frame-extraction/copy-strip-extractor.js";
import { FrameStripExtractor } from "../frame-extraction/strip-extractor.js";
import { createAnalysisVideoDecoder } from "../video-decoding/analysis-video-decoder.js";
import {
  Mp4VideoSource,
  type Mp4VideoTrack,
} from "../video-decoding/mp4-video-source.js";
import { SampleTimestampIndex } from "../video-decoding/sample-timestamp-index.js";
import {
  AnalyzerWorkerSession,
  MeterWorkerSession,
} from "../worker-bridge/client.js";
import { abortReason, throwIfAborted } from "./abort.js";
import { completeAnalysis } from "./complete-analysis.js";
import { DecodePump } from "./decode-pump.js";
import {
  FrameDispatcher,
  type StripFrameExtractor,
} from "./frame-dispatcher.js";
import { logPerformance } from "./performance-log.js";
import { WorkerFrameBridge } from "./worker-frame-bridge.js";

// ─── WebCodecs + Worker 実装 ──────────────────────────────────────────────────

/**
 * VideoDecoder（WebCodecs）+ Worker によるパイプライン解析。
 *
 * デコード、非同期ビットマップ抽出、Worker の WASM 解析を重ねて実行し、
 * 最も遅い段の処理時間へスループットを近づける。
 *
 * データ転送: Transferable ArrayBuffer のピンポン（COOP/COEP 不要）
 * バッファを 2 スロット用意し、片方が Worker にある間に
 * Main はもう片方を準備する。
 */
export async function analyzeWithWebCodecs(
  file: File,
  validatedVideo: ValidatedVideoInput,
  ownSide: string,
  onProgress: AnalysisProgress,
  analysisContext: AnalysisContext,
  signal: AbortSignal,
): Promise<AnalysisResult> {
  throwIfAborted(signal);
  // Startup latency begins at the analysis pipeline entry. Keeping this
  // origin outside the demux source also captures any file materialization or
  // worker/decoder setup that precedes the first encoded sample.
  const analysisStartedAt = performance.now();
  // GPU のある環境では copyTo が GPU→CPU の読み戻しを強制し、canvas 経路より
  // 遅い。実機計測で canvas 2.19ms/frame に対し copyTo 8.49ms/frame。GPU の
  // 無い環境では逆に copyTo が速いが、既定は実利用者の環境へ合わせる。
  const extractor: StripFrameExtractor<VideoFrame, unknown> =
    (await supportsRgbaCopy()) && preferCopyExtraction()
      ? new CopyStripExtractor()
      : new FrameStripExtractor();
  // 独立した WASM インスタンスでメーターと HUD・入力を並列解析する。
  const workerUrl = new URL("./analyzer-worker.js", import.meta.url);
  const resultWorker = new Worker(workerUrl, { type: "module" });
  const meterWorker = new Worker(workerUrl, { type: "module" });

  return new Promise<AnalysisResult>((resolve, reject) => {
    let decoder: VideoDecoder | undefined;
    let settled = false;
    let totalSamples = 0;
    let frameIndex = 0;

    const frameTimestamps: number[] = [];
    const sampleData: FrameSampleData[] = [];
    const frameToSampleIdx: number[] = [];
    let savedCodecConfig: VideoCodecConfig | null = null;
    const sampleByTs = new SampleTimestampIndex();

    let resultWorkerSession: AnalyzerWorkerSession;
    let meterWorkerSession: MeterWorkerSession;
    let videoSource: Mp4VideoSource | undefined;

    const cleanup = (reason?: unknown) => {
      signal.removeEventListener("abort", onAbort);
      videoSource?.stop();
      decodePump.stop();
      resultWorkerSession?.terminate(reason);
      meterWorkerSession?.terminate(reason);
      if (decoder && decoder.state !== "closed") decoder.close();
    };
    const fail = (error: unknown) => {
      if (settled) return;
      settled = true;
      cleanup(error);
      reject(error);
    };
    const succeed = (result: AnalysisResult) => {
      if (settled) return;
      settled = true;
      cleanup();
      resolve(result);
    };
    const onAbort = () => fail(abortReason(signal));
    let frameBridge!: WorkerFrameBridge;
    const frameDispatcher = new FrameDispatcher({
      extractor,
      sendFrame: (frameIndex, pixels) => frameBridge.send(frameIndex, pixels),
      onError: fail,
    });
    frameBridge = new WorkerFrameBridge({
      sendMeter: (message) => meterWorkerSession.sendFrame(message),
      sendResult: (message) => resultWorkerSession.sendFrame(message),
      totalSamples: () => totalSamples,
      drawTime: () => frameDispatcher.drawTime,
      onProgress,
      onFrameCompleted: pumpDecoder,
      signal,
    });
    const decodePump = new DecodePump<EncodedVideoChunk>({
      maxDecodeQueue: 12,
      maxOutstandingFrames: 12,
      maxQueuedSamples: MAX_ENCODED_QUEUE_SAMPLES,
      queuedSampleLowWatermark: ENCODED_QUEUE_SAMPLE_LOW_WATERMARK,
      maxQueuedBytes: MAX_ENCODED_QUEUE_BYTES,
      queuedByteLowWatermark: ENCODED_QUEUE_BYTE_LOW_WATERMARK,
      onQueueLow() {
        videoSource?.pull();
      },
      onReadyToFlush() {
        const activeDecoder = decoder;
        if (!activeDecoder) {
          fail(new Error("デコーダが初期化されていません"));
          return;
        }
        activeDecoder
          .flush()
          .then(() => frameDispatcher.drain())
          .then(() =>
            Promise.all([
              resultWorkerSession.drainFrames(),
              meterWorkerSession.drainFrames(),
            ]),
          )
          .then(() => meterWorkerSession.finish())
          .then((meterTimeline) =>
            resultWorkerSession.finishFirstPass(meterTimeline),
          )
          .catch(fail);
      },
      onError: fail,
    });

    resultWorkerSession = new AnalyzerWorkerSession(resultWorker, {
      onError: fail,
      onFrameResult(message) {
        if (!settled) frameBridge.acceptResult(message);
      },
    });
    meterWorkerSession = new MeterWorkerSession(meterWorker, {
      onError: fail,
      onFrameResult(message) {
        if (!settled) frameBridge.acceptMeter(message);
      },
    });
    signal.addEventListener("abort", onAbort, { once: true });
    if (signal.aborted) {
      onAbort();
      return;
    }

    void completeAnalysis({
      session: resultWorkerSession,
      analysisContext,
      videoFile: file,
      sampleData,
      frameToSampleIdx,
      frameTimestamps,
      getCodecConfig: () => savedCodecConfig,
      onProgress,
      signal,
    })
      .then((result) => {
        logPerformance({
          frameIndex,
          tDraw: frameDispatcher.drawTime,
          ...frameBridge.timing,
          ...(videoSource
            ? {
                streaming: {
                  videoBytes: file.size,
                  preflightMetadataBytes: validatedVideo.metadataBytesRead,
                  demux: videoSource.statistics,
                  encodedQueue: decodePump.statistics,
                },
              }
            : {}),
        });
        succeed(result);
      })
      .catch(fail);

    // ── デコードのバックプレッシャー ─────────────────────────────────────
    // 全サンプルを一括投入するとデコーダがハードウェア速度で出力し、
    // Worker（解析）が追いつかない場合に VideoFrame とビットマップが
    // 無制限に滞留して OOM でタブが落ちる。
    // 「デコーダへ投入済み - Worker 完了」を最大 12 フレームに制限する。
    // この上限にはデコーダ内部、ビットマップ抽出中、Worker 処理中の全段が
    // 含まれる。再開契機は dequeue イベントと frameResult。
    function pumpDecoder() {
      if (settled) return;
      decodePump.pump(decoder, frameBridge.completedFrames);
    }

    async function configureDecoder(track: Mp4VideoTrack): Promise<void> {
      totalSamples = track.totalSamples;
      decodePump.setTotalSamples(totalSamples);
      savedCodecConfig = track.codecConfig;
      decoder = await createAnalysisVideoDecoder(track, {
        onFrame(frame) {
          if (settled) {
            frame.close();
            return;
          }
          try {
            frameTimestamps.push(frame.timestamp / 1_000_000);
            frameToSampleIdx.push(sampleByTs.resolve(frame.timestamp));
            const fi = frameIndex++;
            frameDispatcher.dispatch(frame, fi);
          } catch (error) {
            try {
              frame.close();
            } catch {
              // The dispatcher may already have released the frame.
            }
            fail(error);
          }
        },
        onDequeue: pumpDecoder,
        onError: fail,
        signal,
      });
    }

    videoSource = new Mp4VideoSource(
      file,
      {
        onTrack: configureDecoder,
        onSamples(samples) {
          for (const sample of samples) {
            sampleByTs.add(sample.metadata.timestampUs, sampleData.length);
            sampleData.push(sample.metadata);
            decodePump.enqueue(sample.chunk, sample.chunk.byteLength);
          }
          pumpDecoder();
        },
        onError: fail,
      },
      validatedVideo.track,
      { signal },
    );

    resultWorkerSession.initialize(ownSide, analysisContext);
    meterWorkerSession.initialize(ownSide, analysisContext);
    try {
      videoSource.start(analysisStartedAt);
    } catch (error) {
      fail(error);
    }
  });
}
