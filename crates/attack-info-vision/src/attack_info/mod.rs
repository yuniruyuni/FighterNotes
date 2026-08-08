//! トレーニング/リプレイ中央の攻撃情報表示を構造化して読み取る。
//!
//! 数字は入力履歴と同じ統計モデルを共用し、属性は固定4種類の多数決
//! グリフマスクで分類する。認識不能時は推測せず `None` を返す。

mod model;
mod reader;
mod sequences;
mod templates;
mod tracker;

pub use model::{
    AttackAttribute, AttackInfoFrameInspection, AttackInfoObservation, AttackInfoRoi,
    AttackInfoRois, AttackInfoSide, AttackInfoSideInspection, AttackInfoSideRois, AttackSequence,
    AttackSequenceStep,
};
pub use reader::{read_attack_info, read_attack_info_from_meter_strip};
pub use sequences::build_attack_sequences;
pub use tracker::AttackInfoTracker;

pub fn attack_info_debug_json(rgba: &[u8], width: u32, height: u32) -> String {
    serde_json::to_string(&read_attack_info(rgba, width, height))
        .unwrap_or_else(|error| format!(r#"{{"error":"{error}"}}"#))
}
