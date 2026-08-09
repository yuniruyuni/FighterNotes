use crate::match_events::EventConfidence;

/// 弱点項目（互換出力。カードから生成）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Weakness {
    pub category: String,
    pub description: String,
    pub frequency: u32,
}

/// 証拠クリップ（該当場面へのジャンプ先）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EvidenceClip {
    pub frame: u32,
    pub label: String,
    /// 区間クリップの終端フレーム（None = frame 単点。UI は ±固定窓で再生）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_frame: Option<u32>,
}

/// 指摘カード。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdviceKind {
    /// 複数の証拠を結び、改善可能な原因まで帰属できた指摘。
    Diagnosis,
    /// 原因は断定せず、見直すべき事実を列挙するカード。
    #[default]
    Observation,
    /// 試合全体の集計値を示すカード。
    Statistic,
}

fn default_advice_confidence() -> EventConfidence {
    EventConfidence::Medium
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AdviceCard {
    /// 安定 ID（"anti_air" 等）
    pub id: String,
    #[serde(default)]
    pub kind: AdviceKind,
    #[serde(default = "default_advice_confidence")]
    pub confidence: EventConfidence,
    pub title: String,
    /// 表示ソートキー。HP 量に件数の重みを足した相対値で、HP そのものではない。
    pub severity: f32,
    /// この指摘が挙げた場面で実際に失った HP の合計。
    ///
    /// 「どれから直すべきか」を利用者が判断するための値なので、被ダメージが
    /// 直接の結果である指摘にだけ入れる。確反の取りこぼしのように損失が
    /// 機会費用であるものは、被ダメージへ換算せず `None` のままにする。
    #[serde(default)]
    pub hp_lost: Option<f32>,
    pub description: String,
    /// 練習メニュー提案
    pub practice: String,
    pub evidence: Vec<EvidenceClip>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRequirement {
    OwnHp,
    OpponentHp,
    OwnDrive,
    OpponentDrive,
    OwnSuper,
    OpponentSuper,
    OwnInput,
    OpponentInput,
    FrameMeter,
    Contacts,
    Punishes,
    Spatial,
    OwnAttackInfo,
    OpponentAttackInfo,
}

/// 候補自体は検出したが、必要証拠のcoverage不足で公開カードから除外した記録。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SuppressedAdviceCard {
    pub id: String,
    pub title: String,
    pub missing_requirements: Vec<EvidenceRequirement>,
}
