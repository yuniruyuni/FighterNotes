//! 複合連係へのパリィと、大きなリードからの逆転に対するテスト。
//!
//! どちらも断定しない指摘。飛び道具とテレポートが重なる状況でパリィを
//! 短く離したのは、投げを警戒した判断かもしれない。リードを守れなかった
//! のは、どの選択が悪かったのかまでこの情報からは決まらない。
//!
//! 断定しない指摘ほど、どの場面を並べるかが全てになる。関係ない場面が
//! 混ざると、見返す時間が無駄になる。

use super::super::*;
use super::support::{assert_usable, empty_events};
use crate::match_events::{
    CompoundThreat, DefenseResponse, DefenseResponseKind, MatchEvents, ThreatOutcome,
};
use crate::{AdviceKind, RoundSummary};

// ── 飛び道具とテレポートの複合連係 ───────────────────────────────────────

/// 飛び道具をパリィしたが、その先の打撃で被弾した場面。
fn threat_hit(projectile_start: u32) -> CompoundThreat {
    let followup = projectile_start + 60;
    CompoundThreat {
        attacker: 2,
        defender: 1,
        projectile_start_frame: projectile_start,
        teleport_frame: projectile_start + 30,
        followup_attack_frame: followup,
        followup_contact_frame: Some(followup + 10),
        projectile_response: Some(DefenseResponse {
            side: 1,
            kind: DefenseResponseKind::Parry,
            start_frame: projectile_start + 5,
            end_frame: followup - 5,
        }),
        followup_response: None,
        outcome: ThreatOutcome::Hit,
        damage: 0.18,
        round_no: 1,
        confidence: 1.0,
    }
}

fn events_with_threats(threats: Vec<CompoundThreat>) -> MatchEvents {
    MatchEvents {
        compound_threats: threats,
        ..empty_events()
    }
}

/// 複合連係が無ければ何も出さない。
#[test]
fn nothing_is_reported_without_a_compound_threat() {
    assert!(detect_layered_defense(&empty_events(), 1).is_none());
}

/// 受け切れていれば指摘しない。
#[test]
fn a_threat_that_was_defended_is_not_reported() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].outcome = ThreatOutcome::Defended;

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// 後段にも回答していれば、パリィを短く離した話ではない。
#[test]
fn answering_the_second_hit_is_not_letting_go_early() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].followup_response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Parry,
        start_frame: 155,
        end_frame: 175,
    });

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// パリィを後段まで維持できていれば、離した話ではない。
#[test]
fn holding_the_parry_through_the_second_hit_is_not_reported() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].projectile_response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Parry,
        start_frame: 105,
        end_frame: 165,
    });

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// パリィ以外の回答は、この指摘の対象ではない。ガードで受けたのなら
/// パリィの離し方の話にならない。
#[test]
fn a_response_other_than_a_parry_is_not_this_card() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].projectile_response = Some(DefenseResponse {
        side: 1,
        kind: DefenseResponseKind::Invincible,
        start_frame: 105,
        end_frame: 155,
    });

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// HP が減っていなければ被弾した場面ではない。
#[test]
fn a_threat_that_cost_nothing_is_not_reported() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].damage = 0.0;

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// 相手が守っていた連係は自分の話ではない。
#[test]
fn a_threat_the_opponent_defended_is_not_yours() {
    let mut events = events_with_threats(vec![threat_hit(100)]);
    events.compound_threats[0].defender = 2;

    assert!(detect_layered_defense(&events, 1).is_none());
}

/// 一度きりは、投げを警戒して離した読み合いかもしれない。
#[test]
fn a_single_short_parry_stays_an_observation() {
    let events = events_with_threats(vec![threat_hit(100)]);

    let card = detect_layered_defense(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert_eq!(card.hp_lost, Some(0.18));
    assert!((card.severity - 0.20).abs() < 1e-6);
}

/// 繰り返していれば、防御回答の見直し候補。
#[test]
fn repeating_the_short_parry_becomes_a_diagnosis() {
    let events = events_with_threats(vec![threat_hit(100), threat_hit(1000)]);

    let card = detect_layered_defense(&events, 1).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Diagnosis);
    assert!((card.hp_lost.expect("損失がある") - 0.36).abs() < 1e-6);
    assert!((card.severity - 0.40).abs() < 1e-6);
}

/// 同じ状況が何回あったかを分母に出す。受け切れている回もあるなら、
/// 毎回同じ離し方をしているわけではない。
#[test]
fn the_number_of_such_situations_is_reported() {
    let mut events = events_with_threats(vec![threat_hit(100), threat_hit(1000)]);
    let mut defended = threat_hit(2000);
    defended.outcome = ThreatOutcome::Defended;
    events.compound_threats.push(defended);

    let card = detect_layered_defense(&events, 1).expect("提示される");

    assert!(
        card.description.contains("を 3 回確認"),
        "同じ状況の回数を出していない: {}",
        card.description
    );
    assert!(
        card.description.contains("被弾した場面が 2 回"),
        "被弾の回数を出していない: {}",
        card.description
    );
}

/// クリップは飛び道具から後段の被弾まで。飛び道具を映さないと、
/// なぜパリィを入力したのかが分からない。
#[test]
fn the_clip_starts_at_the_projectile() {
    let events = events_with_threats(vec![threat_hit(100)]);

    let card = detect_layered_defense(&events, 1).expect("提示される");

    assert_eq!(card.evidence[0].frame, 100, "飛び道具から始まっていない");
    assert!(
        card.evidence[0].end_frame.expect("終わりがある") > 160,
        "後段の被弾まで映していない"
    );
}

/// 一度きりと繰り返しで文面を書き分ける。
#[test]
fn the_layered_wording_changes_when_it_repeats() {
    let once = events_with_threats(vec![threat_hit(100)]);
    let twice = events_with_threats(vec![threat_hit(100), threat_hit(1000)]);

    let once = detect_layered_defense(&once, 1).expect("提示される");
    let twice = detect_layered_defense(&twice, 1).expect("提示される");

    assert_eq!(once.id, twice.id);
    assert_ne!(once.title, twice.title, "見出しを書き分けていない");
    assert_ne!(
        once.description, twice.description,
        "説明を書き分けていない"
    );
    assert_ne!(once.practice, twice.practice, "練習方法を書き分けていない");
}

// ── 大きなリードからの逆転 ───────────────────────────────────────────────

/// 一ラウンド分の HP 推移。自分は `own` から `own_end` へ、相手は
/// `opponent` から `opponent_end` へ、まっすぐ減る。
fn round_with(own: (f32, f32), opponent: (f32, f32)) -> MatchEvents {
    let mut events = empty_events();
    let length = 600usize;
    let ramp = |from: f32, to: f32| -> Vec<f32> {
        (0..length)
            .map(|frame| from + (to - from) * frame as f32 / (length - 1) as f32)
            .collect()
    };
    events.hp = [ramp(own.0, own.1), ramp(opponent.0, opponent.1)];
    events.rounds[0].start_frame = 0;
    events.rounds[0].end_frame = length as u32 - 1;
    events
}

fn lost_round() -> Vec<RoundSummary> {
    vec![RoundSummary {
        round_no: 1,
        start_frame: 0,
        end_frame: 599,
        won: Some(false),
        own_hp_end: 0.0,
        opp_hp_end: 0.4,
        own_hp_lost: 1.0,
        opp_hp_lost: 0.6,
        own_hits_taken: 5,
        early_hit: false,
        own_burnouts: 0,
        detection_confidence: "high".to_string(),
    }]
}

/// 勝ったラウンドは並べない。
#[test]
fn a_round_that_was_won_is_not_listed() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));
    let mut rounds = lost_round();
    rounds[0].won = Some(true);

    assert!(detect_lead_loss(&events, &rounds, 0).is_none());
}

/// 勝敗が判らないラウンドも並べない。
#[test]
fn a_round_with_an_unknown_result_is_not_listed() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));
    let mut rounds = lost_round();
    rounds[0].won = None;

    assert!(detect_lead_loss(&events, &rounds, 0).is_none());
}

/// リードが小さいまま落としたラウンドは、逆転ではなく競り負け。
#[test]
fn losing_without_a_large_lead_is_not_a_reversal() {
    let large = round_with((1.0, 0.0), (0.70, 0.40));
    let small = round_with((1.0, 0.0), (0.71, 0.40));

    assert!(
        detect_lead_loss(&large, &lost_round(), 0).is_some(),
        "閾値ちょうどのリードを落としている"
    );
    assert!(
        detect_lead_loss(&small, &lost_round(), 0).is_none(),
        "小さいリードを逆転と呼んでいる"
    );
}

/// 逆転された事実だけを並べる。どの選択が悪かったかは決めない。
#[test]
fn the_card_lists_the_rounds_without_judging_them() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));

    let card = detect_lead_loss(&events, &lost_round(), 0).expect("提示される");

    assert_usable(&card);
    assert_eq!(card.kind, AdviceKind::Observation);
    assert!((card.severity - 0.15).abs() < 1e-6);
    assert!(
        card.description.contains("断定"),
        "決められないことを決めている: {}",
        card.description
    );
}

/// 逆転されたラウンド数を重みにそのまま反映する。
#[test]
fn two_lost_leads_have_twice_the_base_weight() {
    let mut events = round_with((1.0, 0.0), (0.6, 0.4));
    let own_round = events.hp[0].clone();
    let opponent_round = events.hp[1].clone();
    events.hp[0].extend_from_slice(&own_round);
    events.hp[1].extend_from_slice(&opponent_round);
    let second_round = crate::match_events::RoundInfo {
        round_no: 2,
        start_frame: 600,
        end_frame: 1199,
        ..events.rounds[0].clone()
    };
    events.rounds.push(second_round);

    let mut rounds = lost_round();
    rounds.push(RoundSummary {
        round_no: 2,
        start_frame: 600,
        end_frame: 1199,
        ..rounds[0].clone()
    });

    let card = detect_lead_loss(&events, &rounds, 0).expect("提示される");

    assert_eq!(card.evidence.len(), 2);
    assert!((card.severity - 0.30).abs() < 1e-6);
}

/// クリップは最大リードの時点から、逆転された時点まで。手前から
/// 始めると、何が起きて逆転したのかが映らない。
#[test]
fn the_clip_runs_from_the_peak_lead_to_the_flip() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));

    let card = detect_lead_loss(&events, &lost_round(), 0).expect("提示される");
    let clip = &card.evidence[0];

    assert_eq!(clip.frame, 0, "最大リードの時点から始まっていない");
    let flipped = clip.end_frame.expect("終わりがある");
    assert!(flipped > 0 && flipped < 599, "逆転の時点で終わっていない");
    assert!(
        events.hp[1][flipped as usize] > events.hp[0][flipped as usize],
        "まだ逆転していない時点で切っている"
    );
}

/// 最大リードが複数回あれば、最後のものを起点にする。早い方を選ぶと、
/// 立て直した後にまた離した場面が映らない。
#[test]
fn the_latest_peak_is_the_one_used() {
    let mut events = round_with((1.0, 0.0), (0.6, 0.4));
    // 序盤と中盤で同じ差になるよう、相手側を一度戻す。
    for frame in 0..200usize {
        events.hp[0][frame] = 1.0;
        events.hp[1][frame] = 0.6;
    }
    for frame in 200..300usize {
        events.hp[0][frame] = 0.9;
        events.hp[1][frame] = 0.6;
    }
    for frame in 300..350usize {
        events.hp[0][frame] = 1.0;
        events.hp[1][frame] = 0.6;
    }

    let card = detect_lead_loss(&events, &lost_round(), 0).expect("提示される");

    assert_eq!(
        card.evidence[0].frame, 349,
        "最後の最大リードを選んでいない"
    );
}

/// ラウンド終端で初めて最大リードになった場合も、その終端を最大値と
/// クリップ起点の両方へ含める。
#[test]
fn a_peak_on_the_rounds_end_frame_is_included() {
    let mut events = round_with((1.0, 0.0), (0.6, 0.4));
    events.hp = [vec![0.5, 0.5, 1.0], vec![0.5, 0.5, 0.0]];
    events.rounds[0].end_frame = 2;
    let mut rounds = lost_round();
    rounds[0].end_frame = 2;

    let card = detect_lead_loss(&events, &rounds, 0).expect("終端の最大リードを検出する");

    assert_eq!(card.evidence[0].frame, 2);
    assert_eq!(card.evidence[0].end_frame, Some(2));
}

/// HP が同値になっただけでは逆転ではない。相手 HP が上回る最初の
/// フレームまで走査を続ける。
#[test]
fn tied_health_before_the_flip_is_not_the_flip() {
    let mut events = round_with((1.0, 0.0), (0.6, 0.4));
    events.hp = [vec![1.0, 0.5, 0.4], vec![0.6, 0.5, 0.6]];
    events.rounds[0].end_frame = 2;
    let mut rounds = lost_round();
    rounds[0].end_frame = 2;

    let card = detect_lead_loss(&events, &rounds, 0).expect("提示される");

    assert_eq!(card.evidence[0].frame, 0);
    assert_eq!(card.evidence[0].end_frame, Some(2));
}

/// 失った HP は、最大リードから逆転までに減った分。
#[test]
fn the_health_lost_is_measured_from_the_peak() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));

    let card = detect_lead_loss(&events, &lost_round(), 0).expect("提示される");
    let clip = &card.evidence[0];
    let expected = events.hp[0][clip.frame as usize]
        - events.hp[0][clip.end_frame.expect("終わりがある") as usize];

    assert!(
        (card.hp_lost.expect("損失がある") - expected).abs() < 1e-5,
        "最大リード以降の減り方と合っていない"
    );
}

/// 記録に無いラウンドは並べない。
#[test]
fn a_round_without_a_recorded_range_is_skipped() {
    let events = round_with((1.0, 0.0), (0.6, 0.4));
    let mut rounds = lost_round();
    rounds[0].round_no = 9;

    assert!(detect_lead_loss(&events, &rounds, 0).is_none());
}

/// どちらの側の HP を見るかは渡された添字で決まる。取り違えると、
/// リードしていたのが誰かが入れ替わる。
#[test]
fn the_index_decides_whose_lead_is_measured() {
    let mut events = round_with((1.0, 0.0), (0.6, 0.4));

    assert!(
        detect_lead_loss(&events, &lost_round(), 0).is_some(),
        "リードしていた側を見ていない"
    );

    // 同じ試合を、二人を入れ替えて記録した場合。
    events.hp.swap(0, 1);

    let swapped = detect_lead_loss(&events, &lost_round(), 1).expect("入れ替えた側を見ていない");
    let original = detect_lead_loss(&round_with((1.0, 0.0), (0.6, 0.4)), &lost_round(), 0)
        .expect("提示される");

    assert_eq!(
        swapped.evidence[0].frame, original.evidence[0].frame,
        "添字で見る側が切り替わっていない"
    );
    assert_eq!(swapped.hp_lost, original.hp_lost);
}
