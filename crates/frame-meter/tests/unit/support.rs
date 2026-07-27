/// Python-independent unit tests for frame-meter.
///
/// All expected values are derived from the Rust palette constants and
/// algorithm logic embedded in lib.rs — no external fixture files required.
///
/// Color helpers in this file use the exact values from PaletteName::color()
/// so that tests break if the palette is accidentally changed.
pub(super) use frame_meter::{
    brightness_class, classify_cell_pair, classify_cell_raw, extract_row_obs, fresh_color_edge,
    BrightClass, CellState, CELL_COUNT, EMPTY_V_MAX, STRIPE_WF_MIN,
};

// ─── パレット色ヘルパー ────────────────────────────────────────────────────────
//
// dim_anchor(bgr) = [ round(x * 0.75 * 10) / 10  for x in bgr ]
// これは lib.rs の dim_anchor() と完全一致する計算。

pub(super) fn counter() -> [f32; 3] {
    [146.0, 201.0, 19.0]
}
pub(super) fn counter_dim() -> [f32; 3] {
    [109.5, 150.8, 14.3]
} // dim_anchor(counter)
pub(super) fn counter_tint() -> [f32; 3] {
    [130.0, 162.0, 49.0]
} // 別名パレット
pub(super) fn motion_recovery() -> [f32; 3] {
    [237.0, 255.0, 88.0]
}
pub(super) fn motion_recovery_dim() -> [f32; 3] {
    [177.8, 191.3, 66.0]
} // dim_anchor
pub(super) fn punish_counter() -> [f32; 3] {
    [180.0, 112.0, 15.0]
}
pub(super) fn punish_counter_dim() -> [f32; 3] {
    [135.0, 84.0, 11.3]
}
pub(super) fn active() -> [f32; 3] {
    [93.0, 20.0, 176.0]
}
pub(super) fn active_dim() -> [f32; 3] {
    [69.8, 15.0, 132.0]
} // dim_anchor
pub(super) fn projectile_active() -> [f32; 3] {
    [18.0, 127.0, 186.0]
}
pub(super) fn projectile_active_dim() -> [f32; 3] {
    [13.5, 95.3, 139.5]
}
pub(super) fn stun() -> [f32; 3] {
    [55.0, 255.0, 247.0]
}
pub(super) fn stun_dim() -> [f32; 3] {
    [41.3, 191.3, 185.3]
}
pub(super) fn parry() -> [f32; 3] {
    [87.0, 17.0, 65.0]
}
pub(super) fn parry_dim() -> [f32; 3] {
    [65.3, 12.8, 48.8]
}
pub(super) fn white() -> [f32; 3] {
    [236.0, 233.0, 233.0]
}
pub(super) fn white_dim() -> [f32; 3] {
    [177.0, 174.8, 174.8]
} // dim_anchor
pub(super) fn gray() -> [f32; 3] {
    [200.0, 196.0, 197.0]
}
pub(super) fn gray_dim() -> [f32; 3] {
    [150.0, 147.0, 147.8]
} // dim_anchor
pub(super) fn stripe_pink() -> [f32; 3] {
    [140.0, 80.0, 200.0]
}
pub(super) fn stripe_pink_dim() -> [f32; 3] {
    [105.0, 60.0, 150.0]
} // dim_anchor
pub(super) fn stripe_orange() -> [f32; 3] {
    [40.0, 130.0, 230.0]
}
pub(super) fn stripe_orange_dim() -> [f32; 3] {
    [30.0, 97.5, 172.5]
} // dim_anchor
pub(super) fn black() -> [f32; 3] {
    [23.0, 20.0, 23.0]
}
