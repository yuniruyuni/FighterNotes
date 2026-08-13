// ドライブゲージの列分類。1 呼び出しが 1 列を受け持つ。
//
// `classify_drive_col` を写したもの。彩度・明度と正規化は Rust が作った表から
// 引き、色相だけ同じ式で計算する。GPU の除算は Rust と一致しないので、
// 表に置けるものは置き、置けないものは残差を fma で補正する。
//
// 採択の割合は整数比較へ置き換えてある。行数は高々数十なので、
// `n / total >= 0.35` は `n * 20 >= total * 7` と同じ答えになる。

struct Scan {
  x1: u32,
  roi_w: u32,
  strip_y1: u32,
  roi_h: u32,
  gray_row_start: u32,
  falls_right: u32,
};

@group(0) @binding(0) var strip: texture_2d_array<u32>;
@group(0) @binding(1) var<uniform> scans: array<vec4<u32>, 4>;
@group(0) @binding(2) var<storage, read_write> columns: array<u32>;
@group(0) @binding(3) var<storage, read> sv_table: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read> norm_table: array<f32>;

const LIT = 0u;
const GRAY = 1u;
const FOREIGN = 2u;
const REST = 3u;
const OUTSIDE = 4u;
const FLAT = 1e-6;

fn scan_of(side: u32) -> Scan {
  let a = scans[side * 2u];
  let b = scans[side * 2u + 1u];
  return Scan(a.x, a.y, a.z, a.w, b.x, b.y);
}

/// GPU の除算は 1 ULP ほどずれる。残差を fma で取り出して一度補正する。
fn exact_div(a: f32, b: f32) -> f32 {
  let q = a / b;
  let residual = fma(-q, b, a);
  return q + residual / b;
}

/// Rust の f32::round と同じく、0 から遠い方へ丸める。
fn round_half_away(value: f32) -> f32 {
  if (value < 0.0) {
    return -floor(-value + 0.5);
  }
  return floor(value + 0.5);
}

/// pixel_color::rgb_to_hsv と同じ式。彩度と明度は表から引く。
fn to_hsv(r: f32, g: f32, b: f32) -> vec3<f32> {
  let high = u32(max(r, max(g, b)));
  let low = u32(min(r, min(g, b)));
  let sv = sv_table[high * 256u + low];
  let rn = norm_table[u32(r)];
  let gn = norm_table[u32(g)];
  let bn = norm_table[u32(b)];
  let mx = max(rn, max(gn, bn));
  let mn = min(rn, min(gn, bn));
  let delta = mx - mn;
  var h_deg = 0.0;
  if (delta < FLAT) {
    h_deg = 0.0;
  } else if (abs(mx - rn) < FLAT) {
    var t = exact_div(gn - bn, delta);
    if (t < 0.0) { t = t + 6.0; }
    h_deg = 60.0 * t;
  } else if (abs(mx - gn) < FLAT) {
    h_deg = 60.0 * (exact_div(bn - rn, delta) + 2.0);
  } else {
    h_deg = 60.0 * (exact_div(rn - gn, delta) + 4.0);
  }
  return vec3<f32>(round_half_away(h_deg / 2.0), sv.x, sv.y);
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let side = id.z & 1u;
  let frame = id.z >> 1u;
  let scan = scan_of(side);
  let column = id.x;
  if (column >= scan.roi_w) { return; }
  let slope = select(-0.625, 0.625, scan.falls_right == 1u);

  var n_lit = 0u;
  var n_gray = 0u;
  var n_foreign = 0u;
  var total = 0u;
  var gray_total = 0u;
  for (var row = 0u; row < scan.roi_h; row = row + 1u) {
    let offset = i32(round_half_away(f32(row) * slope));
    let x = i32(scan.x1) + i32(column) + offset;
    // ROI の外へ出た行は読まない。CPU 側も同じ条件で数から外す。
    if (x < i32(scan.x1) || x >= i32(scan.x1 + scan.roi_w)) { continue; }
    let texel = textureLoad(strip, vec2<u32>(u32(x), scan.strip_y1 + row), frame, 0);
    total = total + 1u;
    let in_gray_rows = row >= scan.gray_row_start;
    if (in_gray_rows) { gray_total = gray_total + 1u; }

    let hsv = to_hsv(f32(texel.r), f32(texel.g), f32(texel.b));
    let h = hsv.x;
    let s = hsv.y;
    let v = hsv.z;
    if (s > 120.0 && v > 120.0) {
      if (h >= 15.0 && h <= 60.0) {
        n_lit = n_lit + 1u;
      } else {
        n_foreign = n_foreign + 1u;
      }
    } else if (s < 60.0 && v > 120.0 && v < 210.0 && in_gray_rows) {
      n_gray = n_gray + 1u;
    }
  }

  var kind = REST;
  if (total < scan.roi_h) {
    // バーの測定になっていない列。読み取りから外す。
    kind = OUTSIDE;
  } else if (n_lit * 20u >= total * 7u) {
    kind = LIT;
  } else if (n_foreign * 20u >= total * 7u) {
    kind = FOREIGN;
  } else if (gray_total > 0u && n_gray * 5u >= gray_total * 2u) {
    kind = GRAY;
  }
  columns[(frame * 2u + side) * scan.roi_w + column] = kind;
}
