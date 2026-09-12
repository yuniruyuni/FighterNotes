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
/// 接触の物差しにする接地サンプルを、contact からこのフレーム数前まで
/// 遡って探す。hitstop と演出で接触の瞬間は追跡が乱れるため、直前の
/// 安定した立ち姿勢を基準にする。
pub(super) const CONTACT_ACTOR_SAMPLE_LOOKBACK: u32 = 20;
/// 身長の物差しに使う追跡箱の最低身長(正規化)。これより小さい追跡箱は
/// しゃがみ・部分追跡の可能性が高く、身長単位が歪む。
/// 1/8 は 2 進で正確に表せる値で、境界そのものを検査できる。
pub(super) const HEIGHT_REFERENCE_MIN_ACTOR_HEIGHT: f32 = 0.125;
/// ガードスパークが攻撃側の anchor からこの距離(攻撃側の身長単位)以内に
/// あれば「めり込み」。攻撃は根元で当たっており、ガード側の最速反撃も
/// 同等以下の距離で届く。これより遠い先端ガードは何も断定しない。
/// 1/2 は 2 進で正確に表せる値で、境界そのものを検査できる。
pub(super) const PUNISH_DEEP_CONTACT_MAX_REACH: f32 = 0.5;
/// Round 開始からこのフレーム数は、初期間合いの広さから側の入れ替わりが
/// 物理的に起きない。この間だけプレイヤーの色シグネチャを学習する。
pub(super) const ROUND_OPEN_CERTAIN_FRAMES: u32 = 60;
/// 画面中点がこの量以上ずれていれば、カメラは壁でクランプされている。
/// クランプの無いカメラは常に両者の中点を画面中央へ寄せるため、片側へ
/// 追い込まれた場面だけがこの偏りを作る。3/32 は 2 進で正確に表せる値で、
/// 境界そのものを検査できる。
pub(super) const CORNER_MIDPOINT_OFFSET: f32 = 0.09375;
/// 端を背負っている側の anchor が、画面端からこの距離以内にあること。
/// 中点の偏りは knockback の土煙などで anchor が流れても起きるため、
/// 壁の幾何(端側の人物が実際に画面端域にいる)を併せて要求する。
/// 3/16 は 2 進で正確に表せる値で、境界そのものを検査できる。
pub(super) const CORNER_EDGE_X: f32 = 0.1875;
/// corner span を作るのに要する最低サンプル数。
pub(super) const CORNER_MIN_SAMPLES: usize = 3;
/// span の終わり方(入れ替え・離脱)を探す、終端からの猶予フレーム数。
/// 入れ替えの最中は overlap と演出で追跡が乱れるため、落ち着いた後の
/// 観測まで待つ。これを超えたら window 切れとして Unobserved に残す。
pub(super) const CORNER_END_LOOKAHEAD: u32 = 90;
/// corner span 内で許す未確認フレームの最大ギャップ。
pub(super) const CORNER_MAX_GAP: u32 = 8;
