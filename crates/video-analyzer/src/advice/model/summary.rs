/// ラウンドごとのサマリー。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RoundSummary {
    pub round_no: u32,
    pub start_frame: u32,
    pub end_frame: u32,
    /// 自分が勝ったか（判定不能は None）
    pub won: Option<bool>,
    pub own_hp_end: f32,
    pub opp_hp_end: f32,
    /// 自分が失った HP 合計
    pub own_hp_lost: f32,
    pub opp_hp_lost: f32,
    pub own_hits_taken: u32,
    /// 開幕 3 秒以内に被弾した
    pub early_hit: bool,
    pub own_burnouts: u32,
    /// ラウンド境界・勝敗をどこまで確定できたか（"high" / "medium"）。
    #[serde(default)]
    pub detection_confidence: String,
}

/// レポート内の数値が、動画のどの範囲を母集団にしているかを明示する。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AnalysisCoverage {
    pub match_frames: u32,
    pub analyzed_match_frames: u32,
    pub input_segments: u32,
    pub analyzed_input_segments: u32,
    pub attack_damage_events: u32,
    pub attack_damage_linked: u32,
    pub attack_damage_consistent: u32,
    pub attack_damage_mismatched: u32,
}

/// 入力習慣の統計（自分側）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InputStats {
    /// 試合中の入力セグメント総数
    pub total_inputs: u32,
    pub minutes: f32,
    pub jumps: u32,
    pub jumps_per_min: f32,
    pub jump_got_hit: u32,
    pub jump_landed: u32,
    pub throw_attempts: u32,
    pub throw_hits: u32,
    /// 攻撃ボタンを含む入力の数
    pub button_presses: u32,
    pub auto_presses: u32,
    /// AUTO / ボタン押下の比率（Modern の AUTO 依存度）
    pub auto_ratio: f32,
    pub di_presses: u32,
    /// しゃがみ方向（D/DL/DR）を入れていた時間比率
    pub crouch_ratio: f32,
}

/// 指摘の有無とは独立した、戦術ごとの遭遇数と結果。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct TacticStats {
    pub anti_air_opportunities: u32,
    pub anti_air_successes: u32,
    pub jump_ins_allowed: u32,
    pub di_faced: u32,
    pub di_returned: u32,
    pub di_blocked: u32,
    pub di_parried: u32,
    pub di_hit: u32,
    pub di_avoided: u32,
    pub di_unconfirmed: u32,
    pub raw_drive_rushes_faced: u32,
    pub raw_drive_rushes_defended: u32,
    pub raw_drive_rushes_hit: u32,
    pub raw_drive_rushes_unconfirmed: u32,
    pub dash_throws_faced: u32,
    pub throw_whiffs: u32,
    /// 入力を直接観測できた、ガード後 1F 以上不利の判断機会。
    #[serde(default)]
    pub minus_defense_opportunities: u32,
    pub fastest_strike_challenges: u32,
    pub fastest_strike_losses: u32,
    pub fastest_throw_challenges: u32,
    pub fastest_throw_losses: u32,
    pub burnout_count: u32,
    pub burnout_seconds: f32,
    pub burnout_hp_lost: f32,
    pub burnout_hp_dealt: f32,
    pub burnout_self_initiated: u32,
    pub burnout_forced: u32,
    pub burnout_mixed: u32,
    pub burnout_unknown: u32,
    pub sa1_used: u32,
    pub sa2_used: u32,
    pub sa3_used: u32,
    pub ca_used: u32,
    pub super_hits: u32,
    pub super_blocked: u32,
    pub super_no_immediate_contact: u32,
    pub super_punished: u32,
    pub super_kos: u32,
    pub super_combo_uses: u32,
    pub super_punish_uses: u32,
    pub super_reversal_uses: u32,
    pub super_neutral_uses: u32,
    /// ゲーム内中央表示まで帰属できた自分のSA/CAヒット。
    pub super_damage_samples: u32,
    /// SA/CAを含むコンボ全体の表示ダメージ合計。
    pub super_reported_combo_damage: u32,
    /// SA/CA投入直前の累積値から増えた表示ダメージ合計。
    pub super_reported_marginal_damage: u32,
    /// 投入時の表示補正率が50%以下で、KOしなかった使用。
    pub super_low_scaling_uses: u32,
    pub opponent_sa1_used: u32,
    pub opponent_sa2_used: u32,
    pub opponent_sa3_used: u32,
    pub opponent_ca_used: u32,
    pub opponent_super_hits: u32,
    pub opponent_super_blocked: u32,
    pub opponent_super_no_immediate_contact: u32,
    pub opponent_super_punished: u32,
    pub opponent_super_kos: u32,
    pub super_gauge_end: f32,
    pub opponent_super_gauge_end: f32,
}
