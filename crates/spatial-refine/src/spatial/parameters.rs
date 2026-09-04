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
/// contact スパークがこの正規化 y より上(値が小さい)なら、空中での
/// 接触の傍証とみなす。立ち姿勢の頭はこれより下に写るので、これより
/// 高い位置の打撃は空中の身体にしか当たらない。確認にだけ使い、
/// 降格には使わない。
pub(super) const CONTACT_AIRBORNE_MAX_Y: f32 = 0.42;
/// 空中接触の傍証として要求する contact 観測の最低 confidence。
pub(super) const CONTACT_AIRBORNE_MIN_CONFIDENCE: f32 = 0.5;
/// Round 開始からこのフレーム数は、初期間合いの広さから側の入れ替わりが
/// 物理的に起きない。この間だけプレイヤーの色シグネチャを学習する。
pub(super) const ROUND_OPEN_CERTAIN_FRAMES: u32 = 60;
