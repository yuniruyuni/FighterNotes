use super::EventConfidence;

/// SA を使った時点の対戦文脈。技ごとのキャンセル可否を推測せず、
/// 観測済みの直前状態だけで分類する。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperArtContext {
    Combo,
    Punish,
    DefensiveReversal,
    Neutral,
    #[default]
    Unknown,
}

/// SA の直後に確認できた最初の結果。設置型 SA2 等は
/// NoImmediateContact とし、空振り失敗とは断定しない。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperArtOutcome {
    Hit,
    Blocked,
    NoImmediateContact,
    #[default]
    Unconfirmed,
}

/// SA ゲージ消費とフレームメーター・接触・HP を統合した使用イベント。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuperArtEvent {
    pub side: u8,
    /// 暗転または発生開始へ寄せた使用フレーム。
    pub frame: u32,
    /// 低いゲージ表示を最初に再確認できたフレーム。
    pub gauge_drop_frame: u32,
    /// 消費ストックから確定した SA レベル（1〜3）。
    pub level: u8,
    pub critical_art: bool,
    pub gauge_before: f32,
    pub gauge_after: f32,
    pub context: SuperArtContext,
    pub outcome: SuperArtOutcome,
    #[serde(default)]
    pub contact_frame: Option<u32>,
    /// SA 単体へ安全に帰属できた HP 減少。コンボ全体しか読めない場合は 0。
    #[serde(default)]
    pub damage: f32,
    #[serde(default)]
    pub ko: bool,
    /// ガードまたは非接触後の後隙中に受けた反撃。
    #[serde(default)]
    pub punished: bool,
    #[serde(default)]
    pub punished_damage: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}
