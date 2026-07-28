import type { AnalysisContext } from "../../domain/context.js";
import type {
  AnalysisProgress,
  AnalysisResult,
  FrameSample as FrameSampleData,
  VideoCodecConfig,
} from "../../domain/result.js";
import { FrameStripExtractor } from "../frame-extraction/strip-extractor.js";
import { createAnalysisVideoDecoder } from "../video-decoding/analysis-video-decoder.js";
import {
  Mp4VideoSource,
  type Mp4VideoTrack,
} from "../video-decoding/mp4-video-source.js";
import { SampleTimestampIndex } from "../video-decoding/sample-timestamp-index.js";
import { AnalyzerWorkerSession } from "../worker-bridge/client.js";
import { abortReason, throwIfAborted } from "./abort.js";
import { completeAnalysis } from "./complete-analysis.js";
import { DecodePump } from "./decode-pump.js";
import { FrameDispatcher } from "./frame-dispatcher.js";
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
  ownSide: string,
  onProgress: AnalysisProgress,
  analysisContext: AnalysisContext,
  signal: AbortSignal,
): Promise<AnalysisResult> {
  throwIfAborted(signal);
  const arrayBuffer = await file.arrayBuffer();
  throwIfAborted(signal);
  // Worker 起動
  const worker = new Worker(new URL("./analyzer-worker.js", import.meta.url), {
    type: "module",
  });

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

    let workerSession: AnalyzerWorkerSession;

    const cleanup = () => {
      signal.removeEventListener("abort", onAbort);
      workerSession?.terminate();
      if (decoder && decoder.state !== "closed") decoder.close();
    };
    const fail = (error: unknown) => {
      if (settled) return;
      settled = true;
      cleanup();
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
      extractor: new FrameStripExtractor(),
      sendFrame: (frameIndex, pixels) => frameBridge.send(frameIndex, pixels),
      onError: fail,
    });
    frameBridge = new WorkerFrameBridge({
      send: (message) => workerSession.sendFrame(message),
      totalSamples: () => totalSamples,
      drawTime: () => frameDispatcher.drawTime,
      onProgress,
      onFrameCompleted: pumpDecoder,
      signal,
    });
    const decodePump = new DecodePump<EncodedVideoChunk>({
      maxDecodeQueue: 12,
      maxOutstandingFrames: 12,
      onReadyToFlush() {
        const activeDecoder = decoder;
        if (!activeDecoder) {
          fail(new Error("デコーダが初期化されていません"));
          return;
        }
        activeDecoder
          .flush()
          .then(() => frameDispatcher.drain())
          .then(() => workerSession.drainFrames())
          .then(() => workerSession.finishFirstPass())
          .catch(fail);
      },
      onError: fail,
    });

    workerSession = new AnalyzerWorkerSession(worker, {
      onError: fail,
      onFrameResult(message) {
        if (!settled) frameBridge.accept(message);
      },
    });
    signal.addEventListener("abort", onAbort, { once: true });
    if (signal.aborted) {
      onAbort();
      return;
    }

    void completeAnalysis({
      session: workerSession,
      analysisContext,
      videoArrayBuffer: arrayBuffer,
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

    const videoSource = new Mp4VideoSource(arrayBuffer, {
      onTrack: configureDecoder,
      onSamples(samples) {
        for (const sample of samples) {
          sampleByTs.add(sample.metadata.timestampUs, sampleData.length);
          sampleData.push(sample.metadata);
          decodePump.enqueue(sample.chunk);
        }
        pumpDecoder();
      },
      onError: fail,
    });

    workerSession.initialize(ownSide, analysisContext);
    try {
      videoSource.start();
    } catch (error) {
      fail(error);
    }
  });
}
