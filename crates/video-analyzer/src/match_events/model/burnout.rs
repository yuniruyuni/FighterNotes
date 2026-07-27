use super::EventConfidence;

/// バーンアウト期間。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BurnoutPeriod {
    pub side: u8,
    pub start_frame: u32,
    pub end_frame: u32,
    /// 期間中に失った自分の HP
    pub hp_lost: f32,
    /// 期間中に相手へ与えた HP。
    #[serde(default)]
    pub hp_dealt: f32,
    #[serde(default)]
    pub cause: BurnoutCause,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BurnoutCause {
    SelfInitiated,
    ForcedByGuard,
    Mixed,
    #[default]
    Unknown,
}
