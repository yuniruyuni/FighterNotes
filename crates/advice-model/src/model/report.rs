use super::*;

/// 解析アドバイスレポート（JSON 出力の最上位構造）。
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AdviceReport {
    /// 集計互換性を判断する検出ルールの世代。
    #[serde(default)]
    pub ruleset_version: u32,
    /// 同じ ruleset でも配布物を特定できる解析器ビルド ID。
    #[serde(default)]
    pub analyzer_build_id: String,
    pub total_frames: u32,
    pub rounds_detected: u32,
    pub damage_taken_events: Vec<DamageTakenEvent>,
    #[serde(default)]
    pub damage_breakdown: DamageBreakdown,
    pub weaknesses: Vec<Weakness>,
    pub practice_items: Vec<String>,
    pub summary: String,
    /// 指摘カード（原因診断 → 事実確認 → 統計、同種は確度・severity 順）
    pub cards: Vec<AdviceCard>,
    /// 候補はあったが、必要証拠のcoverage不足で抑制したカード。
    #[serde(default)]
    pub suppressed_cards: Vec<SuppressedAdviceCard>,
    pub round_summaries: Vec<RoundSummary>,
    /// 同定できた相手の技ごとの、触られ方と回答の収支。同定できた接触
    /// だけの下限値で、触られた回数の多い順。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub opponent_move_stats: Vec<OpponentMoveStat>,
    /// 自分の技の使用分布。同定できた実行だけの下限値で、回数の多い順。
    /// 流派があるため処方には使わず、偏りを眺める統計に留める。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub own_move_usage: Vec<OwnMoveUsage>,
    /// 入力読み取りが無いパイプラインでは None
    pub input_stats: Option<InputStats>,
    #[serde(default)]
    pub tactic_stats: TacticStats,
    #[serde(default)]
    pub coverage: AnalysisCoverage,
    #[serde(default)]
    pub analysis_warnings: Vec<String>,
}
