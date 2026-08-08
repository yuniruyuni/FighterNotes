/// フレームごとのメーター状態（コンタクト/確反分析用の粗い分類）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MeterState {
    /// 行動可能（empty / その他）
    Free,
    /// 発生前（counter 表示）
    Startup,
    /// 無敵技の発生（inv_full / inv_strike / inv_proj = SA・無敵リバーサル）。
    /// 自分から仕掛けた無敵技であり、相手の後隙への反撃とは区別する
    Invincible,
    /// 攻撃判定発生中（active）
    Active,
    /// 弾の攻撃判定（projectile_active）。「弾を撃った」ことの証拠に使う
    ProjectileActive,
    /// ドライブパリィ・当身判定。複数脅威に対する防御応答として保持する
    Parry,
    /// 通常後隙（motion_recovery = 確反にはならない硬直）
    MotionRecovery,
    /// 技の後隙（punish_counter 表示 = 被確反区間）
    Recovery,
    /// やられ・ガード硬直・ダウン（stun 表示）
    Stun,
}

/// 確定反撃の機会と結果。
///
/// 相手の後隙（Recovery run）中に自分が行動可能だった場面。
/// advantage = 自分が動けた時点から相手の後隙終端までのフレーム数
/// （距離が届く場合、発生がこのフレーム数以下なら時間上は確定する）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PunishChance {
    /// 自分が行動可能になったフレーム
    pub frame: u32,
    pub side: u8,
    /// 実効有利フレーム数
    pub advantage: u32,
    pub outcome: PunishOutcome,
    /// 時間上の反撃候補を裏付けた根拠。
    #[serde(default)]
    pub origin: PunishOrigin,
    /// 相手の Recovery run の実測範囲。
    #[serde(default)]
    pub recovery_start_frame: u32,
    #[serde(default)]
    pub recovery_end_frame: u32,
    /// ガード起点の場合、その接触フレーム。
    #[serde(default)]
    pub source_contact_frame: Option<u32>,
    /// 反撃を試みた場合の発生開始と、最初の攻撃判定フレーム。
    #[serde(default)]
    pub attack_start_frame: Option<u32>,
    #[serde(default)]
    pub attack_active_frame: Option<u32>,
    /// `Missed` / `WhiffFail` の反撃が実際の位置関係で届くか。
    /// フレーム上の有利だけでは、長い通常技の先端ガードを確反と断定できない。
    #[serde(default)]
    pub reachability: PunishReachability,
    /// 空振り失敗の直後に喰らった HP（被弾がなければ 0）
    pub punished_drop: f32,
    /// 反撃に使ったボタン（入力バッジのラベル。押していなければ空）
    pub pressed: String,
    pub round_no: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PunishOrigin {
    /// 相手の攻撃をガードした直後の後隙。
    BlockedMove,
    /// 後隙中のヒット接触により、実際のスカ確だったと事後確認できた。
    VerifiedWhiff,
    /// Recovery 表示との時間的重なりだけで、反撃の因果は未確認。
    #[default]
    UnverifiedRecoveryOverlap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PunishOutcome {
    /// 確反が入った（後隙中〜直後に自分のヒットコンタクト）
    Success,
    /// 攻撃を出したが当たらなかった（距離・選択ミス）
    WhiffFail,
    /// ガード後にフレーム上の反撃猶予があったが、反撃しなかった。
    /// 実際の確反指摘には `reachability == Confirmed` も必要。
    Missed,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PunishReachability {
    /// 位置情報がない、または技ごとの到達距離を確定できない。
    #[default]
    Unknown,
    /// 空間解析で反撃候補として扱える距離を確認した。
    /// 未入力の `Missed` は重なり、通常技を出した `WhiffFail` は近〜中距離。
    Confirmed,
    /// 空間解析で離れており、近距離の最速反撃は届かない。
    OutOfRange,
}
