use crate::frame_data;
use crate::match_events::DamageAttackEvidence;
use crate::match_events::EventConfidence;

/// HP 被弾イベント（互換出力。フロントのクリップ一覧が使用）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DamageTakenEvent {
    pub frame: u32,
    pub own_hp_before: f32,
    pub own_hp_after: f32,
    pub hp_drop: f32,
    pub meter_state: Option<String>,
}

/// 被ダメージ列の主起点。1つの列には必ず1種類だけを割り当てる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageOrigin {
    CompoundThreat,
    Teleport,
    Throw,
    DriveImpact,
    RawDriveRush,
    OwnJumpCaught,
    OpponentJumpIn,
    Projectile,
    Strike,
    Unclassified,
}

/// 被弾へ至った接近経路。接触種別とは独立して保持する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageApproach {
    RawDriveRush,
}

/// 接近経路とは独立した、実際の接触種別。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageContact {
    Throw,
    Strike,
    DriveImpact,
    Projectile,
}

/// 主起点とは独立した、被弾時の状況。複数の状況を同じ列へ付与できる。
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DamageContext {
    Mashing,
    PressWhileMinus,
    GuardBreak,
    ReversalPunished,
    PunishWhiff,
    Burnout,
}

/// 起点を帰属した被ダメージ列。HP は最大体力を 1.0 とする正規化値。
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AttributedDamageEvent {
    pub sequence_no: u32,
    pub round_no: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    pub scene_frame: u32,
    pub hp_before: f32,
    pub hp_after: f32,
    pub hp_drop: f32,
    pub origin: DamageOrigin,
    pub confidence: EventConfidence,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approach: Option<DamageApproach>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact: Option<DamageContact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contact_confidence: Option<EventConfidence>,
    /// 打撃起点を公式入力と照合できた場合のガード属性。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike_kind: Option<frame_data::StrikeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strike_kind_confidence: Option<EventConfidence>,
    #[serde(default)]
    pub contexts: Vec<DamageContext>,
    /// ゲーム内中央表示から同じ被弾列へ帰属できた正確な攻撃情報。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attack_evidence: Option<DamageAttackEvidence>,
}

/// 被ダメージ起点グラフ用の、排他的な帰属結果。
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DamageBreakdown {
    /// ruleset とは独立した起点帰属アルゴリズムの世代。
    pub attribution_version: u32,
    pub total_hp_lost: f32,
    pub classified_hp_lost: f32,
    pub events: Vec<AttributedDamageEvent>,
}
