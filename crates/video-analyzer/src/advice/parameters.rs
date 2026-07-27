pub const RULESET_VERSION: u32 = 6;

/// 「大被弾」とみなす HP ドロップ（暴れ指摘の対象）
pub(crate) const BIG_DAMAGE: f32 = 0.10;
/// 暴れ判定: 被弾開始からこのフレーム数以内のボタン押下を「直前のボタン」とする
pub(crate) const MASH_PRESS_WINDOW: u32 = 25;
/// 入力表示とフレームメーターの時間差を吸収する、技発生リンクの前後幅。
pub(crate) const MASH_STARTUP_LEAD: u32 = 2;
pub(crate) const MASH_STARTUP_LAG: u32 = 8;
/// 因果診断に利用するフレームメーターの最低信頼度。
pub(crate) const MASH_METER_CONFIDENCE: f32 = 0.5;
/// 読み合いの単発結果を「癖」と呼ばないための、判断偏重カード共通条件。
pub(crate) const MIN_DECISION_BIAS_OPPORTUNITIES: usize = 4;
pub(crate) const MIN_DECISION_BIAS_SELECTIONS: usize = 3;
pub(crate) const MIN_DECISION_BIAS_LOSSES: usize = 2;
pub(crate) const MIN_DECISION_BIAS_PERCENT: usize = 70;
/// 読み合いを含む結果を「改善すべき反復」と呼ぶ最低回数。
pub(crate) const MIN_REPEATED_NEGATIVE_OUTCOMES: usize = 2;
/// 単発・低頻度の確認場面で、断定を避けつつ振り返りを促す共通文言。
pub(crate) const OBSERVATION_REVIEW_CAVEAT: &str =
    "断定できませんが、検討の対象にしてもよいかもしれません";
/// 暴れ判定: 押下からこのフレーム数以内に自分側メーターへ projectile_active
/// が出たら「弾を撃った」とみなし暴れから除外する
pub(crate) const MASH_PROJECTILE_WINDOW: usize = 15;
/// 暴れ判定: 被弾直前このフレーム数以内に相手が無敵（弾抜け・無敵技）なら
/// 「弾を読まれた」場面として暴れから除外する
pub(crate) const MASH_INV_LOOKBACK: usize = 20;
/// 被圧コンテキスト: 直近この範囲内に別の被弾があった
pub(crate) const PRESSURE_DMG_WINDOW: u32 = 240;
/// 被圧コンテキスト: この範囲でドライブが削られていた（ガード中の証拠）
pub(crate) const PRESSURE_DRIVE_WINDOW: usize = 120;
pub(crate) const PRESSURE_DRIVE_DROP: f32 = 0.08;
/// 投げループ指摘に必要な「相手投げ連続成功」回数
pub(crate) const THROW_STREAK_MIN: u32 = 3;
/// 開幕被弾の判定フレーム数（3 秒）
pub(crate) const EARLY_HIT_FRAMES: u32 = 180;
/// リード喪失: このリード幅を持ちながら落としたラウンドを指摘
pub(crate) const LEAD_MARGIN: f32 = 0.30;
/// リターン不足とみなす確反成功のドロップ上限（これ未満は伸ばし損ね。
/// 実コンボは 20-35%、軽い単発止まりは 6-9% で分離できる）
pub(crate) const LOW_RETURN_DROP: f32 = 0.12;
/// 「大ダメージを受けた場面」カードの対象とする HP ドロップ
/// （旧フロント側の被弾クリップ一覧の閾値 0.18 を踏襲）
pub(crate) const BIG_HIT_LIST: f32 = 0.18;
/// コンボ（連続 hit コンタクト）のグループ化ギャップ
pub(crate) const COMBO_GAP: u32 = 45;
