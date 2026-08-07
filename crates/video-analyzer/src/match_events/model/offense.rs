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
    InterruptedByInvincible,
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

/// 有利のうちに開始した攻め継続の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PressureFollowUp {
    Strike,
    Throw,
}

/// 有利フレームを取った側が、その有利をどう使ったか。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvantageOutcome {
    /// 相手が動けるようになるまでに次の攻撃を開始した。
    Continued,
    /// 攻撃を開始せず、続けて相手の攻撃をガード／被弾する側へ回った。
    TurnLost,
    /// 攻撃を開始しなかったが、相手も攻めてこず仕切り直しになった。
    Reset,
}

/// ガードさせて有利フレームを取った側の、攻め継続に関する判断機会。
///
/// 有利幅は `MinusSituationEvent` と同じ接触・同じ meter epoch から測る
/// （守備側の不利幅がそのまま攻撃側の有利幅になる）。
/// `action_frame == None` は有利のうちに攻撃を開始しなかったことだけを示し、
/// 前進・様子見・位置調整・ゲージ回復のどれであるかまでは断定しない。
/// このため単発の `TurnLost` を癖とは扱わず、反復と偏りが揃った場合だけ
/// 原因診断へ上げる。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdvantageSituationEvent {
    /// 有利を取った側（直前の攻撃をガードさせた側）。
    pub side: u8,
    /// 有利側が次の行動を開始できるようになったフレーム。
    pub frame: u32,
    /// 実測の有利フレーム数（`ADVANTAGE_THRESHOLD` 以上）。
    pub plus_frames: u32,
    /// 有利のうちに開始した攻撃の種類。入力へ紐付かない場合は None。
    #[serde(default)]
    pub follow_up: Option<PressureFollowUp>,
    /// 攻撃の発生開始フレーム。None は攻撃を開始しなかったことを示す。
    #[serde(default)]
    pub action_frame: Option<u32>,
    #[serde(default)]
    pub pressed: String,
    pub outcome: AdvantageOutcome,
    /// `TurnLost` の後、結果窓のうちに失った HP（それ以外は 0）。
    #[serde(default)]
    pub drop: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    #[serde(default)]
    pub source_contact_frame: u32,
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
