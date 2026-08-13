struct Scan {
  x1: u32,
  roi_w: u32,
  strip_y1: u32,
  row_start: u32,
  row_end: u32,
  falls_right: u32,
};

@group(0) @binding(0) var strip: texture_2d_array<u32>;
@group(0) @binding(1) var<uniform> scans: array<vec4<u32>, 4>;
@group(0) @binding(2) var<storage, read_write> columns: array<u32>;
@group(0) @binding(3) var<storage, read> sv_table: array<vec2<f32>>;
@group(0) @binding(4) var<storage, read> norm_table: array<f32>;

const WHITE = 0u;
const FILL = 1u;
const GHOST = 2u;
const YELLOW_WHITE = 3u;
const ORANGE = 4u;
const DARK = 5u;
const FLAT = 1e-6;

fn scan_of(side: u32) -> Scan {
  let a = scans[side * 2u];
  let b = scans[side * 2u + 1u];
  return Scan(a.x, a.y, a.z, a.w, b.x, b.y);
}

/// GPU の除算は 1 ULP ほどずれることがある。残差を fma で厳密に取り出し、
/// 一度補正して IEEE の丸めへ寄せる。彩度と明度は表から引いているので、
/// 除算が残るのは色相だけ。
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

/// classify_hp_pixel。優先順位は 純白 > 残量 > 残像 > 黄白 > 橙 > 空き。
fn classify_pixel(r: f32, g: f32, b: f32, is_red: bool) -> u32 {
  if (r > 180.0 && g > 180.0 && b > 180.0) {
    return WHITE;
  }
  let hsv = to_hsv(r, g, b);
  let h = hsv.x;
  let s = hsv.y;
  let v = hsv.z;

  var is_primary = false;
  if (is_red) {
    is_primary = (h <= 20.0 || h >= 145.0) && s > 100.0 && v > 60.0;
  } else {
    is_primary = (h >= 88.0 && h <= 160.0) && s > 45.0 && v > 60.0;
  }
  let is_pinch_yellow =
    (h >= 22.0 && h <= 35.0) && s > 120.0 && v > 200.0 && g > r * 0.80;
  if (is_primary || is_pinch_yellow) {
    return FILL;
  }
  if ((h >= 20.0 && h <= 30.0) && s > 150.0 && (v >= 100.0 && v < 200.0) && g > r * 0.82) {
    return GHOST;
  }
  if (r > 165.0 && g > 150.0 && b > 100.0) {
    return YELLOW_WHITE;
  }
  if ((h >= 10.0 && h <= 27.0) && s > 60.0 && v > 80.0) {
    return ORANGE;
  }
  return DARK;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
  let side = id.z & 1u;
  let frame = id.z >> 1u;
  let scan = scan_of(side);
  let column = id.x;
  if (column >= scan.roi_w) { return; }
  let is_red = scan.falls_right == 1u;
  let slope = select(-0.75, 0.75, is_red);

  var n_white = 0u;
  var n_fill = 0u;
  var n_ghost = 0u;
  var n_yw = 0u;
  var n_orange = 0u;
  var total = 0u;
  for (var row = scan.row_start; row < scan.row_end; row = row + 1u) {
    let offset = i32(round_half_away(f32(row - scan.row_start) * slope));
    let x = i32(scan.x1) + i32(column) + offset;
    // ROI の外へ出た行は読まない。CPU 側も同じ条件で数から外す。
    if (x < i32(scan.x1) || x >= i32(scan.x1 + scan.roi_w)) { continue; }
    let texel = textureLoad(strip, vec2<u32>(u32(x), scan.strip_y1 + row), frame, 0);
    total = total + 1u;
    switch classify_pixel(f32(texel.r), f32(texel.g), f32(texel.b), is_red) {
      case 0u: { n_white = n_white + 1u; }
      case 1u: { n_fill = n_fill + 1u; }
      case 2u: { n_ghost = n_ghost + 1u; }
      case 3u: { n_yw = n_yw + 1u; }
      case 4u: { n_orange = n_orange + 1u; }
      default: {}
    }
  }

  var color = DARK;
  if (total == 0u) {
    color = DARK;
  } else if (n_white * 2u >= total) {
    color = WHITE;
  } else if (
    n_white * 10u >= total
    && (n_white + n_yw) * 10u >= total * 8u
    && n_fill == 0u && n_ghost == 0u && n_orange == 0u
  ) {
    color = WHITE;
  } else if (n_fill * 10u >= total) {
    color = FILL;
  } else if (n_ghost * 5u >= total * 2u) {
    color = GHOST;
  } else if (n_yw * 5u >= total * 2u) {
    color = YELLOW_WHITE;
  } else if (n_orange * 20u >= total * 3u) {
    color = ORANGE;
  }
  columns[(frame * 2u + side) * scan.roi_w + column] = color;
}
