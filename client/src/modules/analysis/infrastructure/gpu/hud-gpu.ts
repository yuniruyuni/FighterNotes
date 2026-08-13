import init, {
  Analyzer,
} from "../../../../../../crates/wasm-bridge/pkg/wasm_bridge.js";
import {
  ANALYSIS_WIDTH,
  PACKED_HEIGHT,
  STRIP_RECTS,
} from "../frame-extraction/layout.js";
import { GpuStripExtractor } from "./gpu-strip-extractor.js";
import type { HudGpuBatch } from "./hp-score-batcher.js";
import { createHpScoreBackend } from "./hp-score-webgpu.js";

/**
 * HUD の画素ごとの読み取りを GPU で行う。
 *
 * 復号したフレームをそのまま GPU へ渡すため、復号器と同じ場所に置く。
 * ワーカーへフレームを送る形も試したが、`importExternalTexture` は
 * フレームの資源を GPU の処理が終わるまで押さえるので、受け渡しの往復ぶん
 * 解放が遅れて復号器の持ち玉が尽き、実機で解析が止まった。
 *
 * 読み取った値はまとまりごとに `onBatch` で渡す。値はフレームの順番に
 * 依存しないので、解析の側は受け取った順に置き場所へ入れるだけでよい。
 */
export class HudGpu {
  static async create(
    onBatch: (batch: HudGpuBatch) => void,
  ): Promise<GpuStripExtractor | null> {
    // WebGPU が無ければ表も要らない。主スレッドで WASM を読む前に降りる。
    if (!navigator.gpu) return null;
    // 表は参照実装が作る。GPU に除算をやり直させると、丸めが処理系依存に
    // なって答えが変わりうる。
    await init();
    const backend = await createHpScoreBackend({
      rois: Analyzer.hp_score_rois() as Uint32Array,
      table: Analyzer.hp_score_table() as Uint8Array,
      sv: Analyzer.hsv_sv_table() as Float32Array,
      norm: Analyzer.channel_norm_table() as Float32Array,
      driveScans: Uint32Array.from([
        ...(Analyzer.drive_column_scan("left") as Uint32Array),
        ...(Analyzer.drive_column_scan("right") as Uint32Array),
      ]),
      scans: Uint32Array.from([
        ...(Analyzer.hp_column_scan("p1") as Uint32Array),
        ...(Analyzer.hp_column_scan("p2") as Uint32Array),
      ]),
      stripWidth: ANALYSIS_WIDTH,
      stripHeight: PACKED_HEIGHT,
      rects: STRIP_RECTS,
    });
    if (!backend) return null;
    return new GpuStripExtractor({
      device: backend.device,
      backend,
      texture: backend.texture,
      onBatch,
    });
  }
}

export type { HudGpuBatch };
