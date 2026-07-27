use super::super::threats::{CompoundThreat, ProjectileThreat, TeleportEvent};
use super::*;

/// イベント層の出力一式。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchEvents {
    pub rounds: Vec<RoundInfo>,
    pub damage: Vec<DamageEvent>,
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
    /// ガード崩れ / 被圧被弾（メーターが読めない場合は空）
    pub guard_breaks: Vec<GuardBreakEvent>,
    /// 不利フレーム中のボタン暴れ（メーターが読めない場合は空）
    pub presses_while_minus: Vec<MinusPressEvent>,
    /// 不利フレーム後の回答偏重を測る分母。入力を直接観測できた機会だけ。
    #[serde(default)]
    pub minus_situations: Vec<MinusSituationEvent>,
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
    /// 入力セグメント（[0]=P1, [1]=P2）
    pub segments: [Vec<InputSegment>; 2],
    /// クリーニング済み HP 系列（[0]=P1, [1]=P2、ラウンド内単調非増加）
    #[serde(skip)]
    pub hp: [Vec<f32>; 2],
}
