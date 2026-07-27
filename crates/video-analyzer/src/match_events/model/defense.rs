use super::EventConfidence;

/// 無敵技（DP/SA リバーサル）を撃ってガード/空振りし、後隙を狩られた場面。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReversalEvent {
    pub side: u8,
    pub frame: u32,
    /// 狩られて失った HP
    pub drop: f32,
    /// true = ガードされた / false = 空振り（どちらも被弾したもののみ記録）
    pub blocked: bool,
    #[serde(default)]
    pub confidence: EventConfidence,
    pub round_no: u32,
}

/// ガード入力崩れ。
///
/// ブロックしていた（back / down-back のガード方向 + block コンタクト）のに、
/// 途中で入力がガード方向から外れ（例: ↘→↗）、その非ガード状態のときに
/// 打撃を喰らって STUN + HP 減少した場面。入力履歴・メーター・HP の 3 点一致。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GuardBreakEvent {
    pub side: u8,
    pub frame: u32,
    pub drop: f32,
    /// ブロック時に握っていたガード方向（"DR" / "R" 等）
    pub guard_dir: String,
    /// 被弾時に切り替わっていた非ガード方向（"UR" / "N" 等）
    pub broke_to: String,
    pub round_no: u32,
}

/// 不利フレーム中のボタン暴れ。
///
/// 相手の攻撃をガードして不利（相手が先に動ける）を背負った状態で
/// 攻撃ボタンを押した場面。狩られなかった押しも癖として全件記録する。
/// 弾ガード（projectile 接触）は距離があるため対象外。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MinusPressEvent {
    pub side: u8,
    /// ボタン押下フレーム
    pub frame: u32,
    /// 実測の不利幅（MINUS_PRESS_THRESHOLD 以上）
    pub minus_frames: u32,
    /// 押したボタンのバッジラベル（"弱" 等、複数は "+" 連結）
    pub pressed: String,
    #[serde(default)]
    pub action_kind: DefensiveActionKind,
    pub outcome: MinusPressOutcome,
    /// CounterHit で失った HP（それ以外は 0）
    pub drop: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    #[serde(default)]
    pub source_contact_frame: u32,
    pub round_no: u32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DefensiveActionKind {
    #[default]
    Strike,
    Throw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum MinusPressOutcome {
    /// 押した技が潰されて被弾した
    CounterHit,
    /// 割り込みが通った（リスクはあったが勝った）
    Won,
    /// 相手が攻めず無事に済んだ
    GotAway,
}

/// ガード後に 1F 以上不利になった、確認済みの判断機会。
///
/// `presses_while_minus` は最速打撃・最速投げを実行した場面だけを持つため、
/// それだけでは「何回の機会のうち何回その回答を選んだか」を計算できない。
/// このイベントは、直接観測された入力と同一 meter epoch を確認できた全機会を
/// 分母として残す。`fastest_action == None` はガード継続・移動・無敵技などを
/// 一括した「最速打撃／投げ以外」であり、個別の行動までは断定しない。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MinusSituationEvent {
    pub side: u8,
    /// ガード硬直が解け、自分が行動可能になったフレーム。
    pub frame: u32,
    pub minus_frames: u32,
    #[serde(default)]
    pub fastest_action: Option<DefensiveActionKind>,
    #[serde(default)]
    pub action_frame: Option<u32>,
    #[serde(default)]
    pub pressed: String,
    #[serde(default)]
    pub outcome: Option<MinusPressOutcome>,
    #[serde(default)]
    pub drop: f32,
    #[serde(default)]
    pub confidence: EventConfidence,
    #[serde(default)]
    pub source_contact_frame: u32,
    pub round_no: u32,
}

/// メーター由来の接触イベント（攻撃がヒット/ガードされた瞬間）。
///
/// ヒットストップ中は両者のフレームメーターが同一ゲームフレームで
/// 数フレーム停止する。停止中に片側が active（赤）/ projectile_active、
/// もう片側が stun（黄）なら攻撃が接触している（f8996 実測:
/// 両者 dwell 10 で停止、P1=active / P2=stun）。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ContactEvent {
    /// 停止開始ビデオフレーム（≈ ヒットの瞬間）
    pub frame: u32,
    pub attacker: u8,
    pub victim: u8,
    /// true = ヒット（HP が減った）/ false = ガード（ブロックストップ）
    pub hit: bool,
    /// true = 弾（攻撃側の停止状態が projectile_active）。
    /// 遠距離の弾ガードを密着の固めと区別するために使う
    #[serde(default)]
    pub projectile: bool,
    pub round_no: u32,
}
