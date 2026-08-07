use super::super::threats::{CompoundThreat, ProjectileThreat, TeleportEvent};
use super::*;

/// 候補区間だけを復号する空間解析パスの実行状況。
///
/// `candidate_frames` は重複を統合した候補区間の総フレーム数、
/// `sampled_frames` は実際に空間観測を受け取れた一意なフレーム数。
/// side 別の値は、補間ではなくそのフレームで人物を直接観測できた数。
#[derive(Debug, Clone, Copy, Default)]
pub struct SpatialCoverage {
    pub candidate_frames: u32,
    pub sampled_frames: u32,
    /// 両者を十分な信頼度で追跡でき、距離を利用できる一意なフレーム数。
    pub usable_frames: u32,
    pub p1_observed_frames: u32,
    pub p2_observed_frames: u32,
}

/// 入力確定層をフレーム単位で数えたcoverage。
///
/// segmentへ畳んだ後ではround境界をまたぐ区間内の内訳を復元できないため、
/// production pipelineでは`TrackedInput`から直接集計して保持する。
#[derive(Debug, Clone, Copy, Default)]
pub struct InputCoverage {
    pub measured: bool,
    pub p1_observed_frames: u32,
    pub p2_observed_frames: u32,
    pub p1_repaired_frames: u32,
    pub p2_repaired_frames: u32,
}

/// イベント層の出力一式。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchEvents {
    pub rounds: Vec<RoundInfo>,
    pub damage: Vec<DamageEvent>,
    /// 中央のゲーム内攻撃情報表示を、HP被弾列へ時間・攻撃側で帰属した証拠。
    #[serde(default)]
    pub attack_evidence: AttackEvidence,
    pub jumps: Vec<JumpEvent>,
    pub throws: Vec<ThrowEvent>,
    #[serde(default)]
    pub throw_actions: Vec<ThrowActionEvent>,
    #[serde(default)]
    pub drive_impacts: Vec<DriveImpactEvent>,
    #[serde(default)]
    pub drive_rushes: Vec<DriveRushEvent>,
    pub burnouts: Vec<BurnoutPeriod>,
    /// メーター由来の接触イベント（メーターが読めない場合は空）
    pub contacts: Vec<ContactEvent>,
    /// 確定反撃の機会と結果（メーターが読めない場合は空）
    pub punishes: Vec<PunishChance>,
    /// 無敵技ぶっぱ被弾（メーターが読めない場合は空）
    pub reversals: Vec<ReversalEvent>,
    /// SA ゲージ低下から確定した SA1/2/3/CA の使用。
    #[serde(default)]
    pub super_arts: Vec<SuperArtEvent>,
    /// ガード崩れ / 被圧被弾（メーターが読めない場合は空）
    pub guard_breaks: Vec<GuardBreakEvent>,
    /// 不利フレーム中のボタン暴れ（メーターが読めない場合は空）
    pub presses_while_minus: Vec<MinusPressEvent>,
    /// 不利フレーム後の回答偏重を測る分母。入力を直接観測できた機会だけ。
    #[serde(default)]
    pub minus_situations: Vec<MinusSituationEvent>,
    /// ガードさせて有利を取った側の攻め継続。`minus_situations` と同じ接触から
    /// 測るため、機会の分母も入力を直接観測できたものだけになる。
    #[serde(default)]
    pub advantage_situations: Vec<AdvantageSituationEvent>,
    /// キャラクター行動から独立して残る飛び道具
    #[serde(default)]
    pub projectiles: Vec<ProjectileThreat>,
    /// キャラクター固有のテレポート/位置入れ替え
    #[serde(default)]
    pub teleports: Vec<TeleportEvent>,
    /// 弾とテレポート攻撃など、到達時間が重なる複数脅威
    #[serde(default)]
    pub compound_threats: Vec<CompoundThreat>,
    /// フレームごとのメーター状態（[0]=P1, [1]=P2。メーター無しなら空）
    #[serde(skip)]
    pub meter_state: [Vec<MeterState>; 2],
    /// フレームごとの非 Free メーター状態の読取信頼度（0.0..=1.0）。
    #[serde(skip)]
    pub meter_confidence: [Vec<f32>; 2],
    /// フレームメーターのゲーム内フレーム番号。溜め時間等でヒットストップを除く。
    #[serde(skip)]
    pub meter_game_frame: [Vec<i64>; 2],
    /// 候補区間に限定した空間解析パスのcoverage。
    #[serde(skip)]
    pub spatial_coverage: SpatialCoverage,
    /// 確定ラウンド内の入力履歴を、segment化前のフレーム列から数えたcoverage。
    #[serde(skip)]
    pub input_coverage: InputCoverage,
    /// 入力セグメント（[0]=P1, [1]=P2）
    pub segments: [Vec<InputSegment>; 2],
    /// クリーニング済み HP 系列（[0]=P1, [1]=P2、ラウンド内単調非増加）
    #[serde(skip)]
    pub hp: [Vec<f32>; 2],
}

impl MatchEvents {
    pub fn attack_evidence_for_damage(
        &self,
        damage: &DamageEvent,
    ) -> Option<&DamageAttackEvidence> {
        self.attack_evidence.damage.iter().find(|evidence| {
            evidence.victim == damage.victim && evidence.damage_start_frame == damage.start_frame
        })
    }

    pub fn attack_evidence_for_super(
        &self,
        super_art: &SuperArtEvent,
    ) -> Option<&SuperArtAttackEvidence> {
        self.attack_evidence.super_arts.iter().find(|evidence| {
            evidence.side == super_art.side && evidence.super_frame == super_art.frame
        })
    }

    /// SA/CA自身へ結び付いた中央表示と、その対象HP被弾列がともに厳格条件を
    /// 満たす場合だけ返す。別サイドや別被弾の良好な表示で補完しない。
    pub fn reliable_attack_evidence_for_super(
        &self,
        super_art: &SuperArtEvent,
    ) -> Option<&SuperArtAttackEvidence> {
        let super_evidence = self.attack_evidence_for_super(super_art)?;
        if super_evidence.confidence != EventConfidence::High {
            return None;
        }
        let linked = self
            .attack_evidence
            .damage
            .iter()
            .filter(|evidence| evidence.victim == 3 - super_art.side)
            .filter_map(|evidence| {
                let damage = self.damage.iter().find(|damage| {
                    damage.victim == evidence.victim
                        && damage.start_frame == evidence.damage_start_frame
                        && damage.round_no == super_art.round_no
                })?;
                let in_result_window = damage.start_frame >= super_art.frame.saturating_sub(10)
                    && damage.start_frame <= super_art.frame.saturating_add(360);
                let freeze_distance = damage.pre_freeze_frame.abs_diff(super_art.frame);
                (in_result_window || freeze_distance <= 30).then_some((evidence, freeze_distance))
            })
            .min_by_key(|(_, distance)| *distance)
            .map(|(evidence, _)| evidence)?;
        linked
            .exact_damage_is_strictly_reliable()
            .then_some(super_evidence)
    }
}
