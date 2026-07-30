use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackAttribute {
    Upper,
    Middle,
    Lower,
    Throw,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackInfoSide {
    pub last_damage: u32,
    pub scaling_percent: u32,
    pub combo_damage: u32,
    pub max_combo_damage: u32,
    pub attribute: AttackAttribute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackInfoObservation {
    pub frame_index: u32,
    pub p1: AttackInfoSide,
    pub p2: AttackInfoSide,
}

/// 中央表示の累積値変化から復元した、片側の1コンボ分の攻撃証拠。
///
/// `observation_count` は認識できた表示更新数であり、実際のヒット数ではない。
/// 高速な多段技では中間表示が描画・認識されないことがある。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackSequenceStep {
    pub frame_index: u32,
    pub last_damage: u32,
    pub combo_damage: u32,
    pub scaling_percent: u32,
    pub attribute: AttackAttribute,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttackSequence {
    /// 攻撃側（1|2）。
    pub attacker: u8,
    pub start_frame: u32,
    /// 最後に正の累積ダメージを確認したフレーム。
    pub end_frame: u32,
    /// 0表示または次コンボで終了を確認したフレーム。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub closed_frame: Option<u32>,
    pub combo_damage: u32,
    pub last_damage: u32,
    pub final_scaling_percent: u32,
    /// 最初に読めた値が初段（last == combo）だった場合だけ確定する。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starter_attribute: Option<AttackAttribute>,
    pub final_attribute: AttackAttribute,
    pub observation_count: u32,
    #[serde(default)]
    pub steps: Vec<AttackSequenceStep>,
    pub complete: bool,
    /// 短時間しか出なかった最終値を、次の表示に残った最大値から補完した。
    #[serde(default)]
    pub recovered_from_max: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct AttackInfoRoi {
    pub x1: u32,
    pub x2: u32,
    pub y1: u32,
    pub y2: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttackInfoSideInspection {
    #[serde(flatten)]
    pub value: AttackInfoSide,
    pub numeric_score: u32,
    pub attribute_score: u32,
    pub attribute_margin: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttackInfoSideRois {
    pub numeric: AttackInfoRoi,
    pub attribute: AttackInfoRoi,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttackInfoRois {
    pub p1: AttackInfoSideRois,
    pub p2: AttackInfoSideRois,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AttackInfoFrameInspection {
    pub p1: Option<AttackInfoSideInspection>,
    pub p2: Option<AttackInfoSideInspection>,
    pub rois: AttackInfoRois,
}
