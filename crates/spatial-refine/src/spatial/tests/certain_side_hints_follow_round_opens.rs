use super::*;
use crate::match_events::RoundInfo;

/// 側が確定している区間は round 開始から 60 フレームで、round の終わりと
/// window の境界で切り詰める。範囲外の round があっても走査は続く。
#[test]
fn certain_side_hints_follow_round_opens() {
    let mut events = empty_events();
    events.punishes.push(PunishChance {
        frame: 200,
        side: 1,
        advantage: 4,
        outcome: PunishOutcome::Missed,
        origin: PunishOrigin::BlockedMove,
        recovery_start_frame: 196,
        recovery_end_frame: 203,
        source_contact_frame: Some(195),
        attack_start_frame: None,
        attack_active_frame: None,
        reachability: PunishReachability::Unknown,
        punished_drop: 0.0,
        pressed: String::new(),
        round_no: 1,
    });
    let round = |round_no: u32, start_frame: u32, end_frame: u32| RoundInfo {
        round_no,
        start_frame,
        end_frame,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    };
    // rounds[0] (round_no 1) は window の round 境界クリップに使われる。
    // 追加の round は注釈だけに効く。window は 159..208。
    // 先頭の追加: 完全に window より後(走査が打ち切られないことの確認)。
    // 次: 確定末尾 99+60=159 が開始ちょうどへ届く。98 なら届かない。
    // 次: round が短く、確定末尾が round 終端 170 で切れる。
    // 次: 開始 208 が window 末尾ちょうど。209 は届かない。
    events.rounds.extend([
        round(2, 300, 400),
        round(3, 99, 400),
        round(4, 98, 158),
        round(5, 150, 170),
        round(6, 208, 400),
        round(7, 209, 400),
    ]);

    let windows = spatial_candidate_windows(&events);
    assert_eq!(windows.len(), 1);
    let hints: Vec<(u32, u32)> = windows[0]
        .certain_side_hints
        .iter()
        .map(|range| (range.start_frame, range.end_frame))
        .collect();
    assert_eq!(hints, [(159, 159), (159, 170), (208, 208)]);
}
