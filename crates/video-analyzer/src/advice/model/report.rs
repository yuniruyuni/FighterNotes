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
    /// 入力読み取りが無いパイプラインでは None
    pub input_stats: Option<InputStats>,
    #[serde(default)]
    pub tactic_stats: TacticStats,
    #[serde(default)]
    pub coverage: AnalysisCoverage,
    #[serde(default)]
    pub analysis_warnings: Vec<String>,
}
