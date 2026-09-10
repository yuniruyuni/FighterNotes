use super::support::*;
use crate::match_events::{JumpDirection, JumpEvent, JumpOutcome, RoundInfo};

fn jump(frame: u32, side: u8, outcome: JumpOutcome, contact_height: Option<f32>) -> JumpEvent {
    JumpEvent {
        side,
        frame,
        outcome,
        input_dir: "UL".to_string(),
        direction: JumpDirection::Forward,
        contact_frame: Some(frame + 20),
        takeoff_confirmed: true,
        air_end: frame + 44,
        contact_height,
        round_no: 1,
    }
}

/// 接触高さの内訳は、高さを観測できたジャンプだけを数える下限値。
/// 境界(身長の 5/4)ちょうどは「高い位置」に入り、未観測は high にも
/// deep にも入らないので、合計は迎撃・飛び込みの総数に届かない。
#[test]
fn jump_contact_heights_are_lower_bounds() {
    let mut events = empty_events();
    events.rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 5_999,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    events.jumps = vec![
        // 相手(P2)のジャンプを迎撃した = 自分の対空。境界ちょうどは高め。
        jump(100, 2, JumpOutcome::GotHit, Some(1.25)),
        jump(200, 2, JumpOutcome::GotHit, Some(1.0)),
        jump(300, 2, JumpOutcome::GotHit, Some(0.75)),
        // 高さ未観測の迎撃は内訳に入らない(それでも成功数には入る)。
        jump(400, 2, JumpOutcome::GotHit, None),
        // 相手の通った飛び込みは自分の対空の高さではない。
        jump(500, 2, JumpOutcome::LandedHit, Some(2.0)),
        // 自分(P1)の通った飛び込み。境界ちょうどはこちらでも早当て。
        jump(600, 1, JumpOutcome::LandedHit, Some(1.5)),
        jump(650, 1, JumpOutcome::LandedHit, Some(1.25)),
        jump(700, 1, JumpOutcome::LandedHit, Some(0.5)),
        jump(800, 1, JumpOutcome::LandedHit, None),
        // 自分の落とされたジャンプは当て高さの分子ではない。
        jump(900, 1, JumpOutcome::GotHit, Some(1.5)),
        // 後ろジャンプと確定 round 外は数えない。
        JumpEvent {
            direction: JumpDirection::Backward,
            ..jump(1_000, 1, JumpOutcome::LandedHit, Some(1.5))
        },
        jump(6_500, 1, JumpOutcome::LandedHit, Some(1.5)),
    ];

    let stats = build_tactic_stats(&[], &events, 1, 2);
    assert_eq!(stats.anti_air_successes, 4);
    assert_eq!(stats.anti_air_contacts_high, 1, "1.25 は境界ちょうどで高め");
    assert_eq!(stats.anti_air_contacts_deep, 2);
    assert_eq!(
        stats.own_jump_contacts_high, 2,
        "1.25 は境界ちょうどで早当て"
    );
    assert_eq!(stats.own_jump_contacts_deep, 1);
}
