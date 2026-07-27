use frame_meter::CELL_COUNT;

pub(crate) const DIFF_V_MIN: f32 = 14.0;
pub(crate) const DIFF_WF_MIN: f32 = 0.10;
pub(crate) const BLACKISH_V_MAX: f32 = 55.0;
pub(crate) const READ_WINDOW: i64 = 12;
pub(crate) const FREEZE_TIMEOUT: i64 = 120;
pub(crate) const RESET_DIVERGENCE: i64 = 3;
pub(crate) const RESYNC_TOLERANCE: i64 = 2;
pub(crate) const WIPE_GUARD_MIN_CELLS: i64 = 20;
pub(crate) const DIM_READ_POSITIONS: &[i64] = &[78, 79];
pub(crate) const READ_FRESH_CONF: f64 = 1.0;
pub(crate) const READ_EARLY_CONF: f64 = 0.9;
pub(crate) const READ_DIM_CONF: f64 = 0.8;
pub(crate) const READ_FADE_CONF: f64 = 0.5;
pub(crate) const READ_SETTLE_OFFSET: i64 = 3;
pub(crate) const SLAB_SLIDE_MAD_MAX: f64 = 10.0;
pub(crate) const SLAB_STATIC_MAD_MAX: f64 = 6.0;
pub(crate) const LABEL_DIGIT_BASE: f64 = 0.45;
pub(crate) const LABEL_BLANK_BASE: f64 = 0.50;
pub(crate) const LABEL_DECIDE_MARGIN: f64 = 0.15;
pub(crate) const LABEL_DIGIT_MIN: f64 = 0.55;

pub(crate) const CELL_COUNT_I64: i64 = CELL_COUNT as i64;
