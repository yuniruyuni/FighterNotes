pub const CELL_COUNT: usize = 80;

pub(crate) const ROW_X1: i32 = 359;
pub(crate) const ROW_X2: i32 = 1559;
pub(crate) const LEFT_ROW_Y1: i32 = 796;
pub(crate) const LEFT_ROW_Y2: i32 = 834;
pub(crate) const RIGHT_ROW_Y1: i32 = 836;
pub(crate) const RIGHT_ROW_Y2: i32 = 874;

/// メーターストリップの開始 Y 座標（1920x1080 基準）
pub const METER_STRIP_Y: u32 = 796;
/// メーターストリップの高さ（左右メーター両方を含む最小行数）
pub const METER_STRIP_H: u32 = 78;

pub(crate) const STRIPE_REGION1_ROWS: &[usize] = &[4, 5, 9, 10, 14, 15, 19, 20, 24];
pub(crate) const STRIPE_REGION2_ROWS: &[usize] =
    &[1, 2, 3, 6, 7, 8, 11, 12, 13, 16, 17, 18, 21, 22, 23];
pub(crate) const REGION_REF_ROWS: usize = 26;

pub const HIGHLIGHT_V_MIN: f32 = 90.0;
pub const BLACKISH_PATCH_V: f32 = 55.0;
// WebCodecs の YUV→RGB 変換は OpenCV と異なり、同じ状態セルでもパレット距離が最大 ~92 になる。
pub const PAIR_REJECT_DIST: f32 = 100.0;
pub const STRIPE_WF_MIN: f32 = 0.10;
pub const EMPTY_V_MAX: f32 = 55.0;
pub const FAMILY_ASSIGN_DIST: f32 = 45.0;
pub const RESCUE_MIN_FRAC: f32 = 0.35;

pub(crate) const WHITE_V: f32 = 200.0;
pub const STRIPE_MIN_TRANSITIONS: usize = 6;
pub const STRIPE_MIN_CONTRAST: f32 = 18.0;
pub const STRIPE_MAX_ROW_XSTD: f32 = 30.0;
pub const LIT_ROW_V_MIN: f32 = 60.0;

pub const DIM_V_SCALE: f32 = 0.75;
pub const DIGIT_CHARS: &str = "0123456789";
pub const DIGIT_TEMPLATE_H: usize = 26;
pub const DIGIT_TEMPLATE_W: usize = 13;
