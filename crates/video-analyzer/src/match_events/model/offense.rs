/// 投げ入力イベント。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThrowEvent {
    pub thrower: u8,
    pub frame: u32,
    /// 相手の HP が直後に減った（投げが通った）
    pub connected: bool,
    pub round_no: u32,
}

/// 複数の知覚証拠を突き合わせたイベントの信頼度。
/// 原因を強く断定するカードは原則 `High`、一部の証拠が欠ける場合は
/// `Medium` として表示側にも明示する。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventConfidence {
    #[default]
    Low,
    Medium,
    High,
}

/// 投げ入力を実際の行動結果まで追跡した分類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThrowOutcome {
    Hit,
    Teched,
    ExecutedWhiff,
    Unconfirmed,
}

/// 投げに至った接近方法。位置解析で確定できたものだけ具体化する。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThrowApproach {
    ForwardDash,
    DriveRush,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ThrowActionEvent {
    pub thrower: u8,
    pub input_frame: u32,
    #[serde(default)]
    pub startup_frame: Option<u32>,
    #[serde(default)]
    pub active_frame: Option<u32>,
    pub outcome: ThrowOutcome,
    #[serde(default)]
    pub damage: f32,
    #[serde(default)]
    pub approach: ThrowApproach,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveImpactOutcome {
    Hit,
    Blocked,
    Parried,
    Countered,
    Whiffed,
    Unconfirmed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveImpactEvent {
    pub side: u8,
    pub input_frame: u32,
    #[serde(default)]
    pub active_frame: Option<u32>,
    #[serde(default)]
    pub contact_frame: Option<u32>,
    pub outcome: DriveImpactOutcome,
    #[serde(default)]
    pub damage: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DriveRushOutcome {
    Hit,
    Blocked,
    Stopped,
    NoContact,
    Unconfirmed,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveRushEvent {
    pub side: u8,
    pub frame: u32,
    /// true = 通常技キャンセルではなくパリィ始動の生ラッシュ候補。
    pub raw: bool,
    pub outcome: DriveRushOutcome,
    #[serde(default)]
    pub contact_frame: Option<u32>,
    #[serde(default)]
    pub damage: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}
