use super::support::*;

#[test]
fn legacy_card_without_classification_deserializes_conservatively() {
    let card: AdviceCard = serde_json::from_value(serde_json::json!({
        "id": "legacy",
        "title": "旧カード",
        "severity": 0.1,
        "description": "説明",
        "practice": "練習",
        "evidence": []
    }))
    .expect("ruleset v3 のカードを読める");

    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.confidence, EventConfidence::Medium);
}
