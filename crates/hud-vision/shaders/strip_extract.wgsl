// 復号したフレームから strip を切り出す。
//
// `importExternalTexture` は復号済みのフレームをそのまま GPU 上で読ませる。
// canvas へ合成して読み戻す経路と違い、画素が CPU を経由しない。等倍の領域は
// 実機で突き合わせたところ、canvas 経由で得た画素とバイト単位で一致した。
//
// 外部テクスチャは 0..1 の浮動小数で返るので、ここで 8bit へ戻して以降の
// shader が今までどおり整数で読めるようにする。
//
// 縮小の要る領域 (SA ゲージと FIGHT) は線形補間で写す。canvas の高品質縮小
// とは 1〜2 だけ違う画素が出る。丸め方も Mitchell 三次補間も試したが、
// 線形補間が最も近かった。ここだけは画素が一致しないと分かったうえで載せて
// いる。合成を無くすために必要で、代わりに読み取り結果は変わる。

/// 1 つの矩形。`src` は元フレーム、`dst` は strip の中での位置。
/// 大きさが違えば縮小して写す。
struct Rect {
  src_x: u32,
  src_y: u32,
  src_width: u32,
  src_height: u32,
  dst_x: u32,
  dst_y: u32,
  dst_width: u32,
  dst_height: u32,
};

@group(0) @binding(0) var frame: texture_external;
@group(0) @binding(1) var strip: texture_storage_2d_array<rgba8uint, write>;
@group(0) @binding(2) var<uniform> rects: array<vec4<u32>, 24>;
/// `slot.x` は書き込む層、`slot.y` は写す矩形。
///
/// 矩形ごとにその大きさでディスパッチする。まとめて最大の大きさで投げると、
/// 範囲外で即座に降りるだけのスレッドが大量に走る。実 GPU では安いが、
/// ソフトウェア実装では実費がかかり、試験が桁違いに遅くなる。
@group(0) @binding(3) var<uniform> slot: vec4<u32>;
@group(0) @binding(4) var samp: sampler;

fn rect_of(index: u32) -> Rect {
  let a = rects[index * 2u];
  let b = rects[index * 2u + 1u];
  return Rect(a.x, a.y, a.z, a.w, b.x, b.y, b.z, b.w);
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let rect = rect_of(slot.y);
  if (id.x >= rect.dst_width || id.y >= rect.dst_height) { return; }

  var value = vec4<f32>(0.0);
  if (rect.src_width == rect.dst_width && rect.src_height == rect.dst_height) {
    value = textureLoad(frame, vec2<u32>(rect.src_x + id.x, rect.src_y + id.y));
  } else {
    // 目標画素の中心を元領域へ写して線形補間で読む。
    let u = (f32(rect.src_x)
      + (f32(id.x) + 0.5) * f32(rect.src_width) / f32(rect.dst_width)) / 1920.0;
    let v = (f32(rect.src_y)
      + (f32(id.y) + 0.5) * f32(rect.src_height) / f32(rect.dst_height)) / 1080.0;
    value = textureSampleBaseClampToEdge(frame, samp, vec2<f32>(u, v));
  }

  let stored = vec4<u32>(
    u32(round(value.r * 255.0)),
    u32(round(value.g * 255.0)),
    u32(round(value.b * 255.0)),
    255u,
  );
  // slot.x はまとめの中で書き込む層。
  textureStore(
    strip,
    vec2<u32>(rect.dst_x + id.x, rect.dst_y + id.y),
    slot.x,
    stored,
  );
}
