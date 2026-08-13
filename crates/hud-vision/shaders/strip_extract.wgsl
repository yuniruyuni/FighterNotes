// 復号したフレームから strip を切り出す。
//
// `importExternalTexture` は復号済みのフレームをそのまま GPU 上で読ませる。
// canvas へ合成して読み戻す経路と違い、画素が CPU を経由しない。実機で
// 突き合わせたところ、canvas 経由で得た画素とバイト単位で一致した。
//
// 外部テクスチャは 0..1 の浮動小数で返るので、ここで 8bit へ戻して以降の
// shader が今までどおり整数で読めるようにする。

@group(0) @binding(0) var frame: texture_external;
@group(0) @binding(1) var strip: texture_storage_2d_array<rgba8uint, write>;
@group(0) @binding(2) var<uniform> band: vec4<u32>;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let width = band.x;
  let height = band.y;
  if (id.x >= width || id.y >= height) { return; }
  let source = textureLoad(frame, vec2<u32>(id.x, band.z + id.y));
  let value = vec4<u32>(
    u32(round(source.r * 255.0)),
    u32(round(source.g * 255.0)),
    u32(round(source.b * 255.0)),
    255u,
  );
  textureStore(strip, vec2<u32>(id.x, id.y), band.w, value);
}
