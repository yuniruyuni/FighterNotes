import { HP_COLUMN_SHADER } from "./hp-column-shader.js";
import {
  HP_SCORE_BATCH,
  HP_SCORE_IN_FLIGHT,
  HP_SCORE_VALUES_PER_FRAME,
  type HpScoreBackend,
  type HudGpuResult,
} from "./hp-score-batcher.js";
import { STRIP_EXTRACT_SHADER } from "./strip-extract-shader.js";

export interface HpScoreRois {
  /** `[p1_x1, p1_y1, p1_x2, p1_y2, p2_x1, p2_y1, p2_x2, p2_y2]` (strip 座標)。 */
  readonly rois: Uint32Array;
  /** `max * 256 + min` で引く画素判定表。 */
  readonly table: Uint8Array;
  /** 列走査の形。p1・p2 の順に `[x1, roi_w, strip_y1, row_start, row_end, 右下がりか]`。 */
  readonly scans: Uint32Array;
  /** `max * 256 + min` で引く彩度と明度。 */
  readonly sv: Float32Array;
  /** チャンネル値を 0..1 へ正規化した値。 */
  readonly norm: Float32Array;
  readonly stripWidth: number;
  readonly stripHeight: number;
  /** フレーム全体の中での strip の縦位置。 */
  readonly stripY: number;
}

/**
 * HP スコアの画素数えを WebGPU で行う。
 *
 * shader は画素を整数のまま読み、Rust 側が作った表を引くだけにしてある。
 * 彩度と明度の計算を GPU でやり直すと、除算の丸めが処理系依存になって
 * 走査していた頃と違う答えが出うる。数えた数だけを返し、割り算は Rust に
 * 残すことで、結果は経路によらず同じになる。
 */
const SHADER = `
@group(0) @binding(0) var strip: texture_2d_array<u32>;
@group(0) @binding(1) var<storage, read> table: array<u32>;
@group(0) @binding(2) var<uniform> rois: array<vec4<u32>, 2>;
@group(0) @binding(3) var<storage, read_write> counts: array<atomic<u32>>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let side = id.z & 1u;
  let frame = id.z >> 1u;
  let roi = rois[side];
  let x = roi.x + id.x;
  let y = roi.y + id.y;
  if (x >= roi.z || y >= roi.w) { return; }

  let texel = textureLoad(strip, vec2<u32>(x, y), frame, 0);
  let high = max(texel.r, max(texel.g, texel.b));
  let low = min(texel.r, min(texel.g, texel.b));
  // 表は 1 要素 1 判定。索引の計算だけで答えが決まる。
  let matched = table[high * 256u + low];
  let base = frame * ${HP_SCORE_VALUES_PER_FRAME}u + side * 2u;
  if (matched == 1u) {
    atomicAdd(&counts[base], 1u);
  }
  atomicAdd(&counts[base + 1u], 1u);
}
`;

interface Resources {
  readonly device: GPUDevice;
  readonly pipeline: GPUComputePipeline;
  readonly columnPipeline: GPUComputePipeline;
  readonly texture: GPUTexture;
  readonly counts: GPUBuffer;
  readonly columns: GPUBuffer;
  readonly bindGroup: GPUBindGroup;
  readonly columnBindGroup: GPUBindGroup;
  readonly stagings: GPUBuffer[];
  readonly columnStagings: GPUBuffer[];
  readonly rois: Uint32Array;
  readonly stripWidth: number;
  readonly roiWidth: number;
  readonly extractPipeline: GPUComputePipeline;
  readonly bands: GPUBuffer[];
  readonly stripHeight: number;
}

/**
 * WebGPU が使えるなら数え手を作る。使えなければ `null` を返し、
 * 呼び出し側は従来どおり画素を走査する。
 */
export async function createHpScoreBackend(
  layout: HpScoreRois,
): Promise<HpScoreBackend | null> {
  const adapter = await navigator.gpu?.requestAdapter().catch(() => null);
  const device = await adapter?.requestDevice().catch(() => null);
  if (!device) return null;
  try {
    return new WebGpuHpScoreBackend(build(device, layout));
  } catch {
    device.destroy();
    return null;
  }
}

function build(device: GPUDevice, layout: HpScoreRois): Resources {
  const pipeline = device.createComputePipeline({
    layout: "auto",
    compute: {
      module: device.createShaderModule({ code: SHADER }),
      entryPoint: "main",
    },
  });
  const texture = device.createTexture({
    size: [layout.stripWidth, layout.stripHeight, HP_SCORE_BATCH],
    format: "rgba8uint",
    usage:
      GPUTextureUsage.TEXTURE_BINDING |
      GPUTextureUsage.COPY_DST |
      GPUTextureUsage.STORAGE_BINDING,
  });
  // 復号フレームから直接切り出すパス。まとめの中の位置ごとに書き込む層が違う。
  const extractPipeline = device.createComputePipeline({
    layout: "auto",
    compute: {
      module: device.createShaderModule({ code: STRIP_EXTRACT_SHADER }),
      entryPoint: "main",
    },
  });
  const bands = Array.from({ length: HP_SCORE_BATCH }, (_, layer) => {
    const buffer = device.createBuffer({
      size: 16,
      usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
    });
    const values = new Uint32Array(new ArrayBuffer(16));
    values.set([layout.stripWidth, layout.stripHeight, layout.stripY, layer]);
    device.queue.writeBuffer(buffer, 0, values);
    return buffer;
  });
  const countBytes = HP_SCORE_BATCH * HP_SCORE_VALUES_PER_FRAME * 4;
  const counts = device.createBuffer({
    size: countBytes,
    // まとめごとに 0 へ戻すので、書き込み先としても使う。
    usage:
      GPUBufferUsage.STORAGE |
      GPUBufferUsage.COPY_SRC |
      GPUBufferUsage.COPY_DST,
  });
  const table = device.createBuffer({
    size: layout.table.length * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });
  const expanded = new Uint32Array(new ArrayBuffer(layout.table.length * 4));
  expanded.set(layout.table);
  device.queue.writeBuffer(table, 0, expanded);
  const rois = device.createBuffer({
    size: layout.rois.length * 4,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  const roiValues = new Uint32Array(new ArrayBuffer(layout.rois.length * 4));
  roiValues.set(layout.rois);
  device.queue.writeBuffer(rois, 0, roiValues);
  const bindGroup = device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
      { binding: 0, resource: texture.createView({ dimension: "2d-array" }) },
      { binding: 1, resource: { buffer: table } },
      { binding: 2, resource: { buffer: rois } },
      { binding: 3, resource: { buffer: counts } },
    ],
  });
  const stagings = Array.from({ length: HP_SCORE_IN_FLIGHT }, () =>
    device.createBuffer({
      size: countBytes,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    }),
  );

  // 列分類。同じ strip を読むので、画素を渡すのは 1 回で済む。
  const roiWidth = layout.scans[1] ?? 0;
  const columnPipeline = device.createComputePipeline({
    layout: "auto",
    compute: {
      module: device.createShaderModule({ code: HP_COLUMN_SHADER }),
      entryPoint: "main",
    },
  });
  const columnBytes = HP_SCORE_BATCH * 2 * roiWidth * 4;
  const columns = device.createBuffer({
    size: columnBytes,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
  });
  // uniform は 16 バイト単位で読むので、走査の形も 4 つずつに詰める。
  const scans = device.createBuffer({
    size: 4 * 4 * 4,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  const scanValues = new Uint32Array(new ArrayBuffer(4 * 4 * 4));
  for (const side of [0, 1]) {
    const at = side * 6;
    scanValues.set(layout.scans.subarray(at, at + 4), side * 8);
    scanValues.set(layout.scans.subarray(at + 4, at + 6), side * 8 + 4);
  }
  device.queue.writeBuffer(scans, 0, scanValues);
  const svBuffer = device.createBuffer({
    size: layout.sv.length * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });
  const svValues = new Float32Array(new ArrayBuffer(layout.sv.length * 4));
  svValues.set(layout.sv);
  device.queue.writeBuffer(svBuffer, 0, svValues);
  const normBuffer = device.createBuffer({
    size: layout.norm.length * 4,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_DST,
  });
  const normValues = new Float32Array(new ArrayBuffer(layout.norm.length * 4));
  normValues.set(layout.norm);
  device.queue.writeBuffer(normBuffer, 0, normValues);
  const columnBindGroup = device.createBindGroup({
    layout: columnPipeline.getBindGroupLayout(0),
    entries: [
      {
        binding: 0,
        resource: texture.createView({ dimension: "2d-array" }),
      },
      { binding: 1, resource: { buffer: scans } },
      { binding: 2, resource: { buffer: columns } },
      { binding: 3, resource: { buffer: svBuffer } },
      { binding: 4, resource: { buffer: normBuffer } },
    ],
  });
  const columnStagings = Array.from({ length: HP_SCORE_IN_FLIGHT }, () =>
    device.createBuffer({
      size: columnBytes,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    }),
  );

  return {
    device,
    extractPipeline,
    bands,
    stripHeight: layout.stripHeight,
    pipeline,
    columnPipeline,
    columns,
    columnBindGroup,
    columnStagings,
    roiWidth,
    texture,
    counts,
    bindGroup,
    stagings,
    rois: roiValues,
    stripWidth: layout.stripWidth,
  };
}

class WebGpuHpScoreBackend implements HpScoreBackend {
  readonly #resources: Resources;
  #nextStaging = 0;

  constructor(resources: Resources) {
    this.#resources = resources;
  }

  extractLayer(frame: VideoFrame, layer: number): void {
    const { device, extractPipeline, texture, bands, stripWidth, stripHeight } =
      this.#resources;
    const band = bands[layer];
    if (!band) throw new Error(`Unknown strip layer: ${layer}`);
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(extractPipeline);
    pass.setBindGroup(
      0,
      device.createBindGroup({
        layout: extractPipeline.getBindGroupLayout(0),
        entries: [
          {
            binding: 0,
            resource: device.importExternalTexture({ source: frame }),
          },
          {
            binding: 1,
            resource: texture.createView({ dimension: "2d-array" }),
          },
          { binding: 2, resource: { buffer: band } },
        ],
      }),
    );
    pass.dispatchWorkgroups(Math.ceil(stripWidth / 64), stripHeight, 1);
    pass.end();
    device.queue.submit([encoder.finish()]);
  }

  writeLayer(pixels: ArrayBuffer, layer: number): void {
    const { device, texture, stripWidth } = this.#resources;
    const height = pixels.byteLength / (stripWidth * 4);
    device.queue.writeTexture(
      { texture, origin: [0, 0, layer] },
      pixels,
      { bytesPerRow: stripWidth * 4, rowsPerImage: height },
      [stripWidth, height, 1],
    );
  }

  async count(frames: number): Promise<HudGpuResult> {
    const {
      device,
      pipeline,
      columnPipeline,
      counts,
      columns,
      bindGroup,
      columnBindGroup,
      stagings,
      columnStagings,
      rois,
      roiWidth,
    } = this.#resources;
    const slot = this.#nextStaging % stagings.length;
    this.#nextStaging += 1;
    const staging = stagings[slot];
    const columnStaging = columnStagings[slot];
    if (!staging || !columnStaging) {
      throw new Error("HP score staging buffer is missing");
    }

    const width = Math.max(rois[2] - rois[0], rois[6] - rois[4]);
    const height = Math.max(rois[3] - rois[1], rois[7] - rois[5]);
    const encoder = device.createCommandEncoder();
    encoder.clearBuffer(counts);
    const pass = encoder.beginComputePass();
    pass.setPipeline(pipeline);
    pass.setBindGroup(0, bindGroup);
    pass.dispatchWorkgroups(Math.ceil(width / 64), height, frames * 2);
    pass.setPipeline(columnPipeline);
    pass.setBindGroup(0, columnBindGroup);
    pass.dispatchWorkgroups(Math.ceil(roiWidth / 64), 1, frames * 2);
    pass.end();
    const wantedCounts = frames * HP_SCORE_VALUES_PER_FRAME * 4;
    const wantedColumns = frames * 2 * roiWidth * 4;
    encoder.copyBufferToBuffer(counts, 0, staging, 0, wantedCounts);
    encoder.copyBufferToBuffer(columns, 0, columnStaging, 0, wantedColumns);
    device.queue.submit([encoder.finish()]);

    await Promise.all([
      staging.mapAsync(GPUMapMode.READ, 0, wantedCounts),
      columnStaging.mapAsync(GPUMapMode.READ, 0, wantedColumns),
    ]);
    const scores = Uint32Array.from(
      new Uint32Array(staging.getMappedRange(0, wantedCounts)),
    );
    const columnValues = Uint32Array.from(
      new Uint32Array(columnStaging.getMappedRange(0, wantedColumns)),
    );
    staging.unmap();
    columnStaging.unmap();
    return { scores, columns: columnValues };
  }
}
