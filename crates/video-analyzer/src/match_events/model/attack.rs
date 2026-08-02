use super::EventConfidence;
use crate::attack_info::{AttackAttribute, AttackSequence};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackDamageConsistency {
    Consistent,
    Mismatch,
    #[default]
    Unverified,
}

/// HP被弾列へ帰属できた、ゲーム内攻撃情報表示の集約証拠。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DamageAttackEvidence {
    pub victim: u8,
    pub attacker: u8,
    pub damage_start_frame: u32,
    pub sequence_start_frame: u32,
    pub sequence_end_frame: u32,
    /// 同じHP被弾列に含まれたゲーム内コンボ表示の合計。
    pub combo_damage: u32,
    pub sequence_count: u32,
    pub final_scaling_percent: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub starter_attribute: Option<AttackAttribute>,
    pub final_attribute: AttackAttribute,
    pub complete: bool,
    #[serde(default)]
    pub recovered_from_max: bool,
    pub confidence: EventConfidence,
    #[serde(default)]
    pub hp_consistency: AttackDamageConsistency,
    /// `AttackEvidence::sequences`への内部参照。レポートJSONには出さない。
    #[serde(default, skip_serializing)]
    pub sequence_indices: Vec<u32>,
}

impl DamageAttackEvidence {
    pub fn exact_damage_is_reliable(&self) -> bool {
        self.complete
            && !self.recovered_from_max
            && self.confidence != EventConfidence::Low
            && self.hp_consistency != AttackDamageConsistency::Mismatch
    }

    /// 断定的な表示値とcoverageの分子に使える、完全かつHP整合済みの証拠。
    pub fn exact_damage_is_strictly_reliable(&self) -> bool {
        self.complete
            && !self.recovered_from_max
            && self.confidence == EventConfidence::High
            && self.hp_consistency == AttackDamageConsistency::Consistent
    }
}

/// SA/CA使用へ帰属できたゲーム内コンボ表示。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct SuperArtAttackEvidence {
    pub side: u8,
    pub super_frame: u32,
    pub combo_damage: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marginal_damage: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entry_scaling_percent: Option<u32>,
    pub final_scaling_percent: u32,
    pub confidence: EventConfidence,
}

/// イベント層へ保持する攻撃情報一式。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AttackEvidence {
    #[serde(default)]
    pub sequences: Vec<AttackSequence>,
    #[serde(default)]
    pub damage: Vec<DamageAttackEvidence>,
    #[serde(default)]
    pub super_arts: Vec<SuperArtAttackEvidence>,
}
