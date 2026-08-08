use super::super::*;
use super::support::empty_events;
use crate::match_events::{EventConfidence, MatchEvents, WhiffEvent, WhiffOutcome};

/// 被ダメージが直接の結果である指摘は、挙げた場面で失った HP を公開する。
/// 利用者が「どれから直すか」を決めるための値なので、severity の相対値では
/// なく実測 HP でなければならない。
#[test]
fn damage_backed_cards_publish_the_hp_they_cost() {
    let events = MatchEvents {
        whiffs: vec![
            WhiffEvent {
                side: 1,
                frame: 100,
                end_frame: 108,
                outcome: WhiffOutcome::Punished,
                drop: 0.2,
                punished_frame: Some(115),
                confidence: EventConfidence::High,
                round_no: 1,
            },
            WhiffEvent {
                side: 1,
                frame: 400,
                end_frame: 408,
                outcome: WhiffOutcome::Punished,
                drop: 0.15,
                punished_frame: Some(415),
                confidence: EventConfidence::High,
                round_no: 1,
            },
        ],
        ..empty_events()
    };

    let card = detect_whiff_punished(&events, 1).expect("card");

    let hp_lost = card.hp_lost.expect("damage backed card reports hp");
    assert!((hp_lost - 0.35).abs() < 1e-5);
    // severity は件数の重みを含むためHPとは一致しない。取り違えないこと。
    assert!(card.severity > hp_lost);
}

/// 損失が機会費用である指摘は HP へ換算しない。0 と書くと「損害なし」に
/// 読め、未設定と区別できなくなる。
#[test]
fn opportunity_cost_cards_do_not_claim_hp() {
    let events = empty_events();

    for card in [
        detect_low_conversion(&events, 1),
        detect_punish_missed(&events, 1, None),
        detect_low_scaling_super(&events, 1),
        detect_throw_loop(&events, 2),
    ]
    .into_iter()
    .flatten()
    {
        assert_eq!(card.hp_lost, None, "{} claimed hp", card.id);
    }
}
