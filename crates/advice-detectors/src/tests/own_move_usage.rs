//! 自分の技の使用分布に対する契約。
//!
//! フレームメーターの Startup 連続表示と入力表示の二重整合で同定できた
//! 実行だけを数え、どの回数も下限として扱えることを固定する。結果には
//! 紐づけない。

use super::support::*;
use crate::detectors::build_own_move_usage;
use crate::match_events::{InputEvidence, InputSegment, MeterState};

/// 自分(1P)の技の実測列と入力表示を敷く。`startup` フレームの Startup
/// 連続表示の直後に Active を 1 フレーム置く。
fn plant_own(ev: &mut MatchEvents, start: usize, startup: usize, dir: &str, badge: &str) {
    for frame in start..start + startup {
        ev.meter_state[0][frame] = MeterState::Startup;
    }
    ev.meter_state[0][start + startup] = MeterState::Active;
    ev.segments[0].push(InputSegment {
        start_frame: start as u32,
        end_frame: start as u32 + 3,
        dir: dir.to_string(),
        badges: vec![badge.to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
}

fn events_with_meter(len: usize) -> MatchEvents {
    let mut ev = empty_events();
    ev.meter_state[0] = vec![MeterState::Free; len];
    ev.meter_confidence[0] = vec![1.0; len];
    ev
}

#[test]
fn own_move_usage_tallies_identified_executions() {
    let mut ev = events_with_meter(2_000);

    // ケンの 2MK(発生 7)を 2 回、2HP(発生 8)と 2LP(発生 4)を 1 回ずつ。
    plant_own(&mut ev, 200, 7, "D", "中K");
    plant_own(&mut ev, 600, 7, "D", "中K");
    plant_own(&mut ev, 400, 8, "D", "強P");
    plant_own(&mut ev, 800, 4, "D", "弱P");
    // 入力表示の無い Startup 表示(歩きからの読み違い等)は数えない。
    for frame in 1_000..1_007 {
        ev.meter_state[0][frame] = MeterState::Startup;
    }

    let usage = build_own_move_usage(&ev, 1, Some("KEN"));
    let rows: Vec<(&str, u32)> = usage
        .iter()
        .map(|entry| (entry.name.as_str(), entry.uses))
        .collect();
    assert_eq!(
        rows,
        vec![("2MK", 2), ("2HP", 1), ("2LP", 1)],
        "回数の多い順、同数は記譜の辞書順"
    );
}

#[test]
fn own_move_usage_requires_a_nearby_direct_input() {
    // 押下(= Startup 表示の開始)と入力セグメントのずれは PRESS_SLACK
    // (4F)まで許す。境界の内外で数えるかが変わる。
    for (offset, expected) in [(4_i64, 1_u32), (5, 0)] {
        let mut ev = events_with_meter(2_000);
        for frame in 200..207 {
            ev.meter_state[0][frame] = MeterState::Startup;
        }
        ev.meter_state[0][207] = MeterState::Active;
        // セグメントが押下推定より遅く始まる側の境界。
        ev.segments[0].push(InputSegment {
            start_frame: (200 + offset) as u32,
            end_frame: (200 + offset) as u32 + 3,
            dir: "D".to_string(),
            badges: vec!["中K".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        });
        let usage = build_own_move_usage(&ev, 1, Some("KEN"));
        assert_eq!(
            usage.iter().map(|entry| entry.uses).sum::<u32>(),
            expected,
            "start 側のずれ {offset}F"
        );
    }
    // セグメントが押下推定より先に終わる側の境界。
    for (end_gap, expected) in [(4_i64, 1_u32), (5, 0)] {
        let mut ev = events_with_meter(2_000);
        for frame in 200..207 {
            ev.meter_state[0][frame] = MeterState::Startup;
        }
        ev.meter_state[0][207] = MeterState::Active;
        ev.segments[0].push(InputSegment {
            start_frame: (200 - end_gap - 3) as u32,
            end_frame: (200 - end_gap) as u32,
            dir: "D".to_string(),
            badges: vec!["中K".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        });
        let usage = build_own_move_usage(&ev, 1, Some("KEN"));
        assert_eq!(
            usage.iter().map(|entry| entry.uses).sum::<u32>(),
            expected,
            "end 側のずれ {end_gap}F"
        );
    }
}

#[test]
fn own_move_usage_uses_the_latest_matching_segment() {
    let mut ev = events_with_meter(2_000);
    for frame in 200..207 {
        ev.meter_state[0][frame] = MeterState::Startup;
    }
    ev.meter_state[0][207] = MeterState::Active;
    // 古い入力(2HP 相当)より、押下に近い新しい入力(2MK)を採用する。
    for (start, badge) in [(197_u32, "強P"), (199, "中K")] {
        ev.segments[0].push(InputSegment {
            start_frame: start,
            end_frame: start + 3,
            dir: "D".to_string(),
            badges: vec![badge.to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        });
    }
    let usage = build_own_move_usage(&ev, 1, Some("KEN"));
    assert_eq!(usage.len(), 1);
    assert_eq!(usage[0].name, "2MK");
}

#[test]
fn own_move_usage_skips_unreliable_and_out_of_round_runs() {
    // 読み取り信頼度の足りないフレームを含む実行は実測として使わない。
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 7, "D", "中K");
    ev.meter_confidence[0][203] = 0.49;
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());
    // 境界: 信頼度がしきい値ちょうどなら使う。
    ev.meter_confidence[0][203] = 0.5;
    assert_eq!(build_own_move_usage(&ev, 1, Some("KEN")).len(), 1);

    // round 確定外(round は 0..=5999)の表示は数えない。
    let mut ev = events_with_meter(7_000);
    plant_own(&mut ev, 6_100, 7, "D", "中K");
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());

    // キャラ未指定なら同定できないので空。
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 7, "D", "中K");
    assert!(build_own_move_usage(&ev, 1, None).is_empty());
}

#[test]
fn own_move_usage_ignores_indirect_or_unmatched_inputs() {
    // 直接観測の無い(全補修の)セグメントでは同定しない。
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 7, "D", "中K");
    ev.segments[0][0].evidence = InputEvidence {
        observed_frames: 0,
        repaired_frames: 4,
    };
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());

    // バッジの無い(移動だけの)セグメントでは同定しない。
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 7, "D", "中K");
    ev.segments[0][0].badges.clear();
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());

    // 実測発生が記譜と合わない実行は数えない(2MK は発生 7、±1 まで)。
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 10, "D", "中K");
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());
}

/// 却下された実行(信頼度不足・入力なし・発生不一致)の後の実行も
/// 走査は続き、数えられる。
#[test]
fn own_move_usage_scans_past_rejected_runs() {
    let mut ev = events_with_meter(2_000);
    // 信頼度不足。
    plant_own(&mut ev, 200, 7, "D", "中K");
    ev.meter_confidence[0][203] = 0.0;
    // 入力表示なし。
    for frame in 400..407 {
        ev.meter_state[0][frame] = MeterState::Startup;
    }
    // 実測発生が記譜と合わない。
    plant_own(&mut ev, 600, 12, "D", "中K");
    // これは数える。
    plant_own(&mut ev, 800, 7, "D", "中K");

    let usage = build_own_move_usage(&ev, 1, Some("KEN"));
    assert_eq!(usage.len(), 1, "却下された実行で走査が止まっている");
    assert_eq!((usage[0].name.as_str(), usage[0].uses), ("2MK", 1));
}

/// round 間の表示は数えないが、次 round の実行まで走査は続く。
#[test]
fn own_move_usage_counts_runs_after_a_between_rounds_gap() {
    let mut ev = events_with_meter(9_000);
    ev.rounds.push(crate::match_events::RoundInfo {
        round_no: 2,
        start_frame: 7_000,
        end_frame: 8_999,
        winner: Some(1),
        p1_hp_end: 0.5,
        p2_hp_end: 0.0,
    });
    plant_own(&mut ev, 6_100, 7, "D", "中K");
    plant_own(&mut ev, 7_100, 7, "D", "中K");

    let usage = build_own_move_usage(&ev, 1, Some("KEN"));
    assert_eq!(
        usage.iter().map(|entry| entry.uses).sum::<u32>(),
        1,
        "round 間の実行で走査が止まるか、余計に数えている"
    );
}

/// 実測列の末尾まで Startup が続く実行も、区間の切り出しは配列内に
/// 収まり、通常どおり同定できる。
#[test]
fn own_move_usage_handles_a_run_reaching_the_end_of_the_meter() {
    let mut ev = events_with_meter(1_000);
    for frame in 993..1_000 {
        ev.meter_state[0][frame] = MeterState::Startup;
    }
    ev.segments[0].push(InputSegment {
        start_frame: 993,
        end_frame: 996,
        dir: "D".to_string(),
        badges: vec!["中K".to_string()],
        auto: false,
        throw: false,
        evidence: Default::default(),
    });
    let usage = build_own_move_usage(&ev, 1, Some("KEN"));
    assert_eq!((usage[0].name.as_str(), usage[0].uses), ("2MK", 1));
}

/// 信頼度の検査は実行区間だけに行う。直後(Active)の信頼度は問わない。
#[test]
fn own_move_usage_checks_reliability_only_within_the_run() {
    let mut ev = events_with_meter(2_000);
    plant_own(&mut ev, 200, 7, "D", "中K");
    ev.meter_confidence[0][207] = 0.0;
    assert_eq!(build_own_move_usage(&ev, 1, Some("KEN")).len(), 1);
    // 区間の先頭の信頼度は問う。
    ev.meter_confidence[0][200] = 0.0;
    assert!(build_own_move_usage(&ev, 1, Some("KEN")).is_empty());
}
