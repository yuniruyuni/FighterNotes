import {
  STRIP_BASE_RECTS,
  type StripRect,
} from "../frame-extraction/layout.js";
import { DRIVE_COLUMN_SHADER } from "./drive-column-shader.js";
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
  /** ドライブゲージの走査の形。左・右の順。 */
  readonly driveScans: Uint32Array;
  /** `max * 256 + min` で引く彩度と明度。 */
  readonly sv: Float32Array;
  /** チャンネル値を 0..1 へ正規化した値。 */
  readonly norm: Float32Array;
  readonly stripWidth: number;
  readonly stripHeight: number;
  /** 復号フレームから切り出す等倍の矩形。 */
  readonly rects: readonly StripRect[];
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

interface ColumnPass {
  readonly pipeline: GPUComputePipeline;
  readonly bindGroup: GPUBindGroup;
  readonly buffer: GPUBuffer;
  readonly stagings: GPUBuffer[];
  readonly width: number;
}

interface Resources {
  readonly device: GPUDevice;
  readonly pipeline: GPUComputePipeline;
  readonly hpPass: ColumnPass;
  readonly drivePass: ColumnPass;
  readonly texture: GPUTexture;
  readonly counts: GPUBuffer;
  readonly bindGroup: GPUBindGroup;
  readonly stagings: GPUBuffer[];
  readonly rois: Uint32Array;
  readonly stripWidth: number;
  readonly extractPipeline: GPUComputePipeline;
  readonly extractLayout: GPUBindGroupLayout;
  readonly sampler: GPUSampler;
  readonly bands: GPUBuffer;
  readonly slotStride: number;
  readonly rectBuffer: GPUBuffer;
  readonly rectCount: number;
  readonly extractSize: { readonly width: number; readonly height: number };
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

/**
 * 列分類のパスを組み立てる。HP バーもドライブゲージも、走査の形が違うだけで
 * 同じ形をしている。
 */
function buildColumnPass(
  device: GPUDevice,
  code: string,
  scanValues: Uint32Array,
  texture: GPUTexture,
  sv: GPUBuffer,
  norm: GPUBuffer,
): ColumnPass {
  const width = scanValues[1] ?? 0;
  const pipeline = device.createComputePipeline({
    layout: "auto",
    compute: {
      module: device.createShaderModule({ code }),
      entryPoint: "main",
    },
  });
  const bytes = HP_SCORE_BATCH * 2 * width * 4;
  const buffer = device.createBuffer({
    size: bytes,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC,
  });
  // uniform は 16 バイト単位で読むので、走査の形も 4 つずつに詰める。
  const scans = device.createBuffer({
    size: 4 * 4 * 4,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  const packed = new Uint32Array(new ArrayBuffer(4 * 4 * 4));
  for (const side of [0, 1]) {
    const at = side * 6;
    packed.set(scanValues.subarray(at, at + 4), side * 8);
    packed.set(scanValues.subarray(at + 4, at + 6), side * 8 + 4);
  }
  device.queue.writeBuffer(scans, 0, packed);
  return {
    pipeline,
    buffer,
    width,
    bindGroup: device.createBindGroup({
      layout: pipeline.getBindGroupLayout(0),
      entries: [
        { binding: 0, resource: texture.createView({ dimension: "2d-array" }) },
        { binding: 1, resource: { buffer: scans } },
        { binding: 2, resource: { buffer } },
        { binding: 3, resource: { buffer: sv } },
        { binding: 4, resource: { buffer: norm } },
      ],
    }),
    stagings: Array.from({ length: HP_SCORE_IN_FLIGHT }, () =>
      device.createBuffer({
        label: "column-readback",
        size: bytes,
        usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
      }),
    ),
  };
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
      GPUTextureUsage.COPY_SRC |
      GPUTextureUsage.STORAGE_BINDING,
  });
  // 復号フレームから直接切り出すパス。まとめの中の位置ごとに書き込む層が違う。
  // 縮小する領域だけが使う。等倍の領域は textureLoad で読む。
  const sampler = device.createSampler({
    magFilter: "linear",
    minFilter: "linear",
  });
  // 読み出し位置を動的に切り替えるので、束ね方は明示する。
  const extractLayout = device.createBindGroupLayout({
    entries: [
      { binding: 0, visibility: GPUShaderStage.COMPUTE, externalTexture: {} },
      {
        binding: 1,
        visibility: GPUShaderStage.COMPUTE,
        storageTexture: {
          access: "write-only",
          format: "rgba8uint",
          viewDimension: "2d-array",
        },
      },
      { binding: 2, visibility: GPUShaderStage.COMPUTE, buffer: {} },
      {
        binding: 3,
        visibility: GPUShaderStage.COMPUTE,
        buffer: { hasDynamicOffset: true, minBindingSize: 16 },
      },
      { binding: 4, visibility: GPUShaderStage.COMPUTE, sampler: {} },
    ],
  });
  const extractPipeline = device.createComputePipeline({
    layout: device.createPipelineLayout({
      bindGroupLayouts: [extractLayout],
    }),
    compute: {
      module: device.createShaderModule({ code: STRIP_EXTRACT_SHADER }),
      entryPoint: "main",
    },
  });
  // 切り出す矩形は毎フレーム同じ。まとめの中で変わるのは書き込む層だけ。
  const rectValues = new Uint32Array(new ArrayBuffer(24 * 4 * 4));
  layout.rects.forEach((rect, index) => {
    rectValues.set(
      [rect.src.x, rect.src.y, rect.src.width, rect.src.height],
      index * 8,
    );
    rectValues.set(
      [rect.dst.x, rect.dst.y, rect.dst.width, rect.dst.height],
      index * 8 + 4,
    );
  });
  const rectBuffer = device.createBuffer({
    size: rectValues.byteLength,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  device.queue.writeBuffer(rectBuffer, 0, rectValues);
  // 層ごとに「土台」と「重ね書き」の 2 つ。動的な読み出し位置で切り替える
  // ので、束ねる操作はフレームあたり 1 回で済む。
  const SLOT_STRIDE = 256;
  const bands = device.createBuffer({
    size: SLOT_STRIDE * HP_SCORE_BATCH * 2,
    usage: GPUBufferUsage.UNIFORM | GPUBufferUsage.COPY_DST,
  });
  for (let layer = 0; layer < HP_SCORE_BATCH; layer += 1) {
    for (const [phase, first] of [
      [0, 0],
      [1, STRIP_BASE_RECTS],
    ] as const) {
      const values = new Uint32Array(new ArrayBuffer(16));
      values.set([layer, first]);
      device.queue.writeBuffer(
        bands,
        (layer * 2 + phase) * SLOT_STRIDE,
        values,
      );
    }
  }
  const extractSize = layout.rects.reduce(
    (largest, rect) => ({
      width: Math.max(largest.width, rect.dst.width),
      height: Math.max(largest.height, rect.dst.height),
    }),
    { width: 0, height: 0 },
  );

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
      label: "score-readback",
      size: countBytes,
      usage: GPUBufferUsage.MAP_READ | GPUBufferUsage.COPY_DST,
    }),
  );

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

  // 列分類。同じ strip を読むので、画素を渡すのは 1 回で済む。
  const hpPass = buildColumnPass(
    device,
    HP_COLUMN_SHADER,
    layout.scans,
    texture,
    svBuffer,
    normBuffer,
  );
  const drivePass = buildColumnPass(
    device,
    DRIVE_COLUMN_SHADER,
    layout.driveScans,
    texture,
    svBuffer,
    normBuffer,
  );

  return {
    device,
    extractPipeline,
    extractLayout,
    sampler,
    bands,
    slotStride: SLOT_STRIDE,
    rectBuffer,
    rectCount: layout.rects.length,
    extractSize,
    stripHeight: layout.stripHeight,
    pipeline,
    hpPass,
    drivePass,
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

  get texture(): GPUTexture {
    return this.#resources.texture;
  }

  get device(): GPUDevice {
    return this.#resources.device;
  }
  #nextStaging = 0;

  constructor(resources: Resources) {
    this.#resources = resources;
  }

  extractLayer(frame: VideoFrame, layer: number): void {
    const {
      device,
      extractPipeline,
      texture,
      bands,
      slotStride,
      rectBuffer,
      rectCount,
      extractSize,
      sampler,
      extractLayout,
    } = this.#resources;
    const encoder = device.createCommandEncoder();
    const pass = encoder.beginComputePass();
    pass.setPipeline(extractPipeline);
    const bindGroup = device.createBindGroup({
      layout: extractLayout,
      entries: [
        {
          binding: 0,
          resource: device.importExternalTexture({ source: frame }),
        },
        {
          binding: 1,
          resource: texture.createView({ dimension: "2d-array" }),
        },
        { binding: 2, resource: { buffer: rectBuffer } },
        { binding: 3, resource: { buffer: bands, size: 16 } },
        { binding: 4, resource: sampler },
      ],
    });
    // 土台を書いてから重ねる。同じパスの中でも、積んだ順に実行される。
    for (const [phase, count] of [
      [0, STRIP_BASE_RECTS],
      [1, rectCount - STRIP_BASE_RECTS],
    ] as const) {
      if (count <= 0) continue;
      pass.setBindGroup(0, bindGroup, [(layer * 2 + phase) * slotStride]);
      pass.dispatchWorkgroups(
        Math.ceil(extractSize.width / 64),
        extractSize.height,
        count,
      );
    }
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
      counts,
      bindGroup,
      stagings,
      rois,
      hpPass,
      drivePass,
    } = this.#resources;
    const slot = this.#nextStaging % stagings.length;
    this.#nextStaging += 1;
    const staging = stagings[slot];
    const hpStaging = hpPass.stagings[slot];
    const driveStaging = drivePass.stagings[slot];
    if (!staging || !hpStaging || !driveStaging) {
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
    for (const columnPass of [hpPass, drivePass]) {
      pass.setPipeline(columnPass.pipeline);
      pass.setBindGroup(0, columnPass.bindGroup);
      pass.dispatchWorkgroups(Math.ceil(columnPass.width / 64), 1, frames * 2);
    }
    pass.end();
    const wantedCounts = frames * HP_SCORE_VALUES_PER_FRAME * 4;
    const wantedHp = frames * 2 * hpPass.width * 4;
    const wantedDrive = frames * 2 * drivePass.width * 4;
    encoder.copyBufferToBuffer(counts, 0, staging, 0, wantedCounts);
    encoder.copyBufferToBuffer(hpPass.buffer, 0, hpStaging, 0, wantedHp);
    encoder.copyBufferToBuffer(
      drivePass.buffer,
      0,
      driveStaging,
      0,
      wantedDrive,
    );
    device.queue.submit([encoder.finish()]);

    await Promise.all([
      staging.mapAsync(GPUMapMode.READ, 0, wantedCounts),
      hpStaging.mapAsync(GPUMapMode.READ, 0, wantedHp),
      driveStaging.mapAsync(GPUMapMode.READ, 0, wantedDrive),
    ]);
    const scores = Uint32Array.from(
      new Uint32Array(staging.getMappedRange(0, wantedCounts)),
    );
    const columns = Uint32Array.from(
      new Uint32Array(hpStaging.getMappedRange(0, wantedHp)),
    );
    const drive = Uint32Array.from(
      new Uint32Array(driveStaging.getMappedRange(0, wantedDrive)),
    );
    staging.unmap();
    hpStaging.unmap();
    driveStaging.unmap();
    return { scores, columns, drive };
  }
}
