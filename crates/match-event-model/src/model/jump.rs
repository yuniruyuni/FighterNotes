/// ジャンプの結果（ジャンプした側の視点）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum JumpOutcome {
    /// 飛びが通り攻撃がヒットした（相手の HP が減った）
    LandedHit,
    /// 空中で迎撃された（自分の HP が減った）
    GotHit,
    /// 被弾窓には入ったが、入力に対応する新しい離陸を第一段で確認できない。
    /// 空間二段目で空中を確認できた場合だけ GotHit へ昇格する。
    UnverifiedHit,
    /// 上入力後に被弾したが、接触時点は空中ではなかった。
    /// 「ジャンプを落とされた」助言には使わない。
    GroundedHit,
    /// ジャンプ予備動作（地上 4F）中に狩られた。対空されたのではなく
    /// 「ガードすべき場面で動いた」被弾（暴れ系の材料）
    PreJumpClipped,
    /// どちらの HP も動かなかった（様子見・スカし・ガードされた）
    Neutral,
}

/// 離陸時の相手との相対方向。第一段では斜め入力の向きを断定できないため
/// `Unknown` とし、候補窓の位置観測後に `Forward` / `Backward` へ確定する。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JumpDirection {
    #[default]
    Unknown,
    Neutral,
    Forward,
    Backward,
}

fn jump_takeoff_confirmed_default() -> bool {
    true
}

/// ジャンプイベント。
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct JumpEvent {
    pub side: u8,
    pub frame: u32,
    pub outcome: JumpOutcome,
    /// 入力履歴で観測した絶対方向（U / UR / UL）。
    #[serde(default)]
    pub input_dir: String,
    /// 相手との相対方向。空間二段目を通す前の斜め入力は Unknown。
    #[serde(default)]
    pub direction: JumpDirection,
    /// このジャンプへ排他的に帰属したヒットコンタクト。
    #[serde(default)]
    pub contact_frame: Option<u32>,
    /// 入力近傍から始まる未使用のメーターラン、または空間二段目で離陸を
    /// 確認できた候補。false の候補はユーザー向け対空助言に使わない。
    #[serde(default = "jump_takeoff_confirmed_default")]
    pub takeoff_confirmed: bool,
    /// ジャンプ結果を帰属できる終端。通常キャラは固定物理上限、長滞空を
    /// 明示したキャラだけメーターの移動系ラン終端 + マージンまで延長する。
    #[serde(default)]
    pub air_end: u32,
    pub round_no: u32,
}
