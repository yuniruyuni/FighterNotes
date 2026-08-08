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

/// リード喪失の損失は、最大リード時点から逆転までに自分が失った HP。
/// 引き算の向きを取り違えると符号が反転する。
#[test]
fn lead_loss_reports_the_hp_given_up_between_peak_and_flip() {
    use crate::advice::RoundSummary;
    use crate::match_events::RoundInfo;

    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 99,
        winner: Some(2),
        p1_hp_end: 0.1,
        p2_hp_end: 0.6,
    }];
    // 自分は 1.0 から 0.1 へ、相手は 0.5 のまま。ピークは frame 0。
    let own: Vec<f32> = (0..100)
        .map(|frame| if frame < 50 { 1.0 } else { 0.1 })
        .collect();
    events.hp = [own, vec![0.5; 100]];

    let summaries = vec![RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: 99,
        won: Some(false),
        own_hp_end: 0.1,
        opp_hp_end: 0.5,
        own_hp_lost: 0.9,
        opp_hp_lost: 0.5,
        own_hits_taken: 1,
        early_hit: false,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    }];

    let card = detect_lead_loss(&events, &summaries, 0).expect("card");

    let hp_lost = card.hp_lost.expect("lead loss reports hp");
    assert!((hp_lost - 0.9).abs() < 1e-5);
}
