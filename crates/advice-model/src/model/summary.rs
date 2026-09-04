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
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceAvailability {
    Available,
    #[default]
    Unavailable,
    NotApplicable,
}

impl EvidenceAvailability {
    pub fn is_available(self) -> bool {
        self == Self::Available
    }
}

/// 閾値と依存関係を解析器側で解決した、表示・カード共通の可用性契約。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AnalysisAvailability {
    pub own_hp: EvidenceAvailability,
    pub opponent_hp: EvidenceAvailability,
    pub own_drive: EvidenceAvailability,
    pub opponent_drive: EvidenceAvailability,
    pub own_super: EvidenceAvailability,
    pub opponent_super: EvidenceAvailability,
    pub own_input: EvidenceAvailability,
    pub opponent_input: EvidenceAvailability,
    pub own_meter: EvidenceAvailability,
    pub opponent_meter: EvidenceAvailability,
    pub contacts: EvidenceAvailability,
    pub punishes: EvidenceAvailability,
    pub spatial: EvidenceAvailability,
    pub own_attack_info: EvidenceAvailability,
    pub opponent_attack_info: EvidenceAvailability,
}

impl AnalysisAvailability {
    /// 「改善点なし」と結論できるだけの知覚層が揃っているか。
    /// 機会自体が無い `NotApplicable` は欠測ではないため許容する。
    pub fn supports_no_findings_claim(&self) -> bool {
        ![
            self.own_hp,
            self.opponent_hp,
            self.own_drive,
            self.opponent_drive,
            self.own_super,
            self.opponent_super,
            self.own_input,
            self.opponent_input,
            self.own_meter,
            self.opponent_meter,
            self.contacts,
            self.punishes,
            self.spatial,
            self.own_attack_info,
            self.opponent_attack_info,
        ]
        .contains(&EvidenceAvailability::Unavailable)
    }
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct AnalysisCoverage {
    pub match_frames: u32,
    pub analyzed_match_frames: u32,
    pub input_segments: u32,
    pub analyzed_input_segments: u32,
    /// 検出器別の読取率に共通して使う、確定ラウンド内の試合フレーム数。
    pub detector_match_frames: u32,
    pub own_hp_reliable_frames: u32,
    pub opponent_hp_reliable_frames: u32,
    pub own_drive_reliable_frames: u32,
    pub opponent_drive_reliable_frames: u32,
    pub own_super_reliable_frames: u32,
    pub opponent_super_reliable_frames: u32,
    pub own_super_end_reliable: bool,
    pub opponent_super_end_reliable: bool,
    pub own_input_observed_frames: u32,
    pub opponent_input_observed_frames: u32,
    pub own_input_repaired_frames: u32,
    pub opponent_input_repaired_frames: u32,
    pub own_meter_mapped_frames: u32,
    pub opponent_meter_mapped_frames: u32,
    /// 空間解析は全試合ではなく、意味イベントから選んだ候補区間だけが分母。
    pub spatial_candidate_frames: u32,
    pub spatial_sampled_frames: u32,
    pub spatial_usable_frames: u32,
    pub own_spatial_observed_frames: u32,
    pub opponent_spatial_observed_frames: u32,
    pub attack_damage_events: u32,
    pub attack_damage_linked: u32,
    pub attack_damage_consistent: u32,
    pub attack_damage_mismatched: u32,
    pub attack_damage_unverified: u32,
    /// 自分の攻撃（相手側HP減少）に対する中央攻撃表示の母数と厳格利用可能数。
    pub own_attack_damage_events: u32,
    pub own_attack_damage_usable: u32,
    /// 相手の攻撃（自分側HP減少）に対する中央攻撃表示の母数と厳格利用可能数。
    pub opponent_attack_damage_events: u32,
    pub opponent_attack_damage_usable: u32,
    /// None はruleset v8以前の保存済みレポートだけを表す。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub availability: Option<AnalysisAvailability>,
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
    /// 判断機会の数と、そこで最も多かった回答が占める割合（百分率）。
    /// 回答が読まれているかを測る。機会が少ないうちは意味を持たないため、
    /// 割合だけでなく機会数も一緒に持つ。
    #[serde(default)]
    pub disadvantage_decisions: u32,
    #[serde(default)]
    pub disadvantage_top_option_percent: u32,
    #[serde(default)]
    pub advantage_decisions: u32,
    #[serde(default)]
    pub advantage_top_option_percent: u32,
    #[serde(default)]
    pub okizeme_decisions: u32,
    #[serde(default)]
    pub okizeme_top_option_percent: u32,
    /// 自分が画面端を背負って受けた被弾。空間解析が端を確認できた場面に
    /// 限るため下限値で、0 は「端で被弾しなかった」を意味しない。
    #[serde(default)]
    pub cornered_hits_taken: u32,
    /// 相手を画面端に追い込んで与えた被弾。同じく下限値。
    #[serde(default)]
    pub cornered_hits_dealt: u32,
    /// 自分が守る側になった投げ。抜けと被投げの分母。
    /// 相手が届かない位置で振った投げ（空振り）は、守る機会ではないので含めない。
    #[serde(default)]
    pub throws_faced: u32,
    /// 抜けられずに投げられた数。
    #[serde(default)]
    pub throws_taken: u32,
    /// 投げ抜けが成立した数。
    #[serde(default)]
    pub throws_teched: u32,
    /// 無敵技で投げを潰した数。抜けとは別の解決手段なので分けて数える。
    #[serde(default)]
    pub throws_reversal_escaped: u32,
    /// 自分が取ったダウンと、その起き上がりへの攻め。
    #[serde(default)]
    pub knockdowns_scored: u32,
    /// 起き上がりに攻撃判定を重ねられた数（持続当て）。
    #[serde(default)]
    pub okizeme_meaty: u32,
    /// 重ねてはいないが、起き上がり直後に攻めを始めた数。
    #[serde(default)]
    pub okizeme_pressured: u32,
    /// 何も始めず仕切り直しになった数。距離を取る選択も含むため、
    /// これだけでは失敗として扱わない。
    #[serde(default)]
    pub okizeme_neutral: u32,
    /// 自分が取られたダウン。起き攻めを受けた回数の分母。
    #[serde(default)]
    pub knockdowns_taken: u32,
    /// そのうち起き上がりに攻撃判定を重ねられた数。
    #[serde(default)]
    pub okizeme_faced_meaty: u32,
    /// 自分の Drive Impact の結果内訳。相手の DI を受けた側の `di_*` とは別。
    #[serde(default)]
    pub own_di_used: u32,
    #[serde(default)]
    pub own_di_hit: u32,
    #[serde(default)]
    pub own_di_blocked: u32,
    #[serde(default)]
    pub own_di_parried: u32,
    /// 相手の DI で返された数。
    #[serde(default)]
    pub own_di_countered: u32,
    #[serde(default)]
    pub own_di_whiffed: u32,
    #[serde(default)]
    pub own_di_unconfirmed: u32,
    /// 自分の生 Drive Rush の結果内訳。
    #[serde(default)]
    pub own_raw_drive_rushes: u32,
    #[serde(default)]
    pub own_raw_drive_rush_hits: u32,
    #[serde(default)]
    pub own_raw_drive_rush_defended: u32,
    /// 確定済みゲージ系列から実測した消費量（1.0 = ゲージ全量）。
    /// SF6 の本数を仮定せず、行動前後の実際の減少だけを積む。
    #[serde(default)]
    pub drive_spent_on_impacts: f32,
    #[serde(default)]
    pub drive_spent_on_rushes: f32,
    /// 上の消費に紐づけて確認できた与ダメージ。1本あたりの効率の分子。
    #[serde(default)]
    pub drive_damage_from_impacts: f32,
    #[serde(default)]
    pub drive_damage_from_rushes: f32,
    /// 消費量を実測できた行動の数。0 のとき効率を表示しないための分母。
    #[serde(default)]
    pub drive_spend_samples: u32,
    /// 接触しなかった自分の攻撃判定（投げ・DI・無敵技・弾を除く）。
    #[serde(default)]
    pub whiffs: u32,
    /// そのうち硬直を狩られた数。
    #[serde(default)]
    pub whiffs_punished: u32,
    /// 相手の空振り。差し返しの分母。
    #[serde(default)]
    pub opponent_whiffs: u32,
    /// 相手の空振りを狩れた数。差し返しの分子。
    #[serde(default)]
    pub opponent_whiffs_punished: u32,
    /// 入力を直接観測できた、ガードさせて有利を取った側の判断機会。
    #[serde(default)]
    pub advantage_opportunities: u32,
    /// 有利のうちに次の攻撃を開始した機会。
    #[serde(default)]
    pub advantage_continued: u32,
    /// 有利のうちに攻撃を開始しなかった機会。前進・様子見・回復を含む。
    #[serde(default)]
    pub advantage_abandoned: u32,
    /// 攻撃を開始せず、続けて相手の攻撃を受ける側へ回った機会。
    #[serde(default)]
    pub advantage_turns_lost: u32,
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
    /// 全有効ラウンドで自分側のSA使用を完全集計できる観測被覆がある。
    /// falseでも検出済み件数は下限として利用できる。
    pub super_art_stats_complete: bool,
    /// 全有効ラウンドで相手側のSA使用を完全集計できる観測被覆がある。
    /// falseでも検出済み件数は下限として利用できる。
    pub opponent_super_art_stats_complete: bool,
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
