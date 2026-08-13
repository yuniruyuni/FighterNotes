// 復号したフレームから strip を切り出す。
//
// `importExternalTexture` は復号済みのフレームをそのまま GPU 上で読ませる。
// canvas へ合成して読み戻す経路と違い、画素が CPU を経由しない。実機で
// 突き合わせたところ、canvas 経由で得た画素とバイト単位で一致した。
//
// 外部テクスチャは 0..1 の浮動小数で返るので、ここで 8bit へ戻して以降の
// shader が今までどおり整数で読めるようにする。
//
// 切り出しは等倍の矩形だけを扱う。縮小の要る SA ゲージと FIGHT は、
// canvas の高品質縮小と 1 だけずれる画素が出るため、ここでは扱わない。

/// 1 つの矩形。`src` は元フレーム、`dst` は strip の中での位置。
struct Rect {
  src_x: u32,
  src_y: u32,
  width: u32,
  height: u32,
  dst_x: u32,
  dst_y: u32,
};

@group(0) @binding(0) var frame: texture_external;
@group(0) @binding(1) var strip: texture_storage_2d_array<rgba8uint, write>;
@group(0) @binding(2) var<uniform> rects: array<vec4<u32>, 16>;
@group(0) @binding(3) var<uniform> slot: vec4<u32>;

fn rect_of(index: u32) -> Rect {
  let a = rects[index * 2u];
  let b = rects[index * 2u + 1u];
  return Rect(a.x, a.y, a.z, a.w, b.x, b.y);
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let rect = rect_of(id.z);
  if (id.x >= rect.width || id.y >= rect.height) { return; }
  let source = textureLoad(frame, vec2<u32>(rect.src_x + id.x, rect.src_y + id.y));
  let value = vec4<u32>(
    u32(round(source.r * 255.0)),
    u32(round(source.g * 255.0)),
    u32(round(source.b * 255.0)),
    255u,
  );
  // slot.x はまとめの中で書き込む層。
  textureStore(strip, vec2<u32>(rect.dst_x + id.x, rect.dst_y + id.y), slot.x, value);
}
