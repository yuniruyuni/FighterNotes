/// ラウンド情報。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoundInfo {
    pub round_no: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    /// 勝者（1|2）。判定不能（動画途中で切れた等）は None
    pub winner: Option<u8>,
    pub p1_hp_end: f32,
    pub p2_hp_end: f32,
}

/// ダメージシーケンス（連続 HP 減少のまとまり）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DamageEvent {
    /// 被弾側（1|2）
    pub victim: u8,
    pub start_frame: u32,
    pub end_frame: u32,
    /// 被弾直前の演出フリーズ（SA 暗転・投げ演出等でメーターが長時間
    /// 両者停止するスパン）の開始フレーム。フリーズが無ければ start_frame
    /// と同値。クリップの「演出前から再生」アンカーに使う
    #[serde(default)]
    pub pre_freeze_frame: u32,
    pub hp_before: f32,
    pub hp_after: f32,
    pub drop: f32,
    pub round_no: u32,
}
