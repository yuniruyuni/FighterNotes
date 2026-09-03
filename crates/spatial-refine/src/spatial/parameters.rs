pub(super) const PUNISH_SPATIAL_LOOKBACK: u32 = 36;
pub(super) const PUNISH_SPATIAL_LOOKAHEAD: u32 = 8;
pub(super) const PUNISH_SPATIAL_SAMPLE_PADDING: u32 = 2;
pub(super) const PUNISH_SPATIAL_MIN_SAMPLES: usize = 2;
pub(super) const JUMP_SPATIAL_LOOKBACK: u32 = 6;
pub(super) const JUMP_SPATIAL_LOOKAHEAD: u32 = 2;
pub(super) const JUMP_AIR_SAMPLE_LOOKBACK: u32 = 8;
pub(super) const JUMP_AIR_MIN_SAMPLES: usize = 2;
/// contact frame から何フレーム後までを hitstop のヒント区間に含めるか。
/// SF6 の hitstop は概ね 8〜13F で、スパークはその間表示され続ける。
pub(super) const CONTACT_HINT_TAIL_FRAMES: u32 = 10;
