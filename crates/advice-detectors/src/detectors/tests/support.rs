use crate::match_events::{MatchEvents, RoundInfo};

pub fn empty_events() -> MatchEvents {
    MatchEvents {
        rounds: vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 5_999,
            winner: Some(2),
            p1_hp_end: 0.0,
            p2_hp_end: 0.5,
        }],
        damage: vec![],
        attack_evidence: Default::default(),
        jumps: vec![],
        throws: vec![],
        throw_actions: vec![],
        drive_impacts: vec![],
        drive_rushes: vec![],
        burnouts: vec![],
        contacts: vec![],
        punishes: vec![],
        reversals: vec![],
        super_arts: vec![],
        guard_breaks: vec![],
        presses_while_minus: vec![],
        minus_situations: vec![],
        advantage_situations: vec![],
        knockdowns: vec![],
        whiffs: vec![],
        projectiles: vec![],
        teleports: vec![],
        compound_threats: vec![],
        meter_state: [vec![], vec![]],
        meter_confidence: [vec![], vec![]],
        meter_game_frame: [vec![], vec![]],
        spatial_coverage: Default::default(),
        corner_spans: vec![],
        input_coverage: Default::default(),
        segments: [vec![], vec![]],
        hp: [vec![1.0; 6_000], vec![1.0; 6_000]],
    }
}

/// 提示するカードとして成立しているか。
///
/// 文面や根拠が欠けたカードは、利用者に空欄を見せることになる。指摘の
/// 内容が正しくても、読めなければ何も伝わらない。
pub fn assert_usable(card: &crate::AdviceCard) {
    assert!(!card.id.is_empty(), "id が空");
    assert!(!card.title.is_empty(), "見出しが空");
    assert!(!card.description.is_empty(), "説明が空");
    assert!(!card.practice.is_empty(), "練習方法が空");
    assert!(!card.evidence.is_empty(), "根拠のクリップが無い");
    for clip in &card.evidence {
        assert!(!clip.label.is_empty(), "クリップの見出しが空: {clip:?}");
    }
}
