use frame_meter::{BrightClass, CellState, RowObs, CELL_COUNT};
use meter_tracker::MeterTracker;

/// 全セル empty・V=0 の合成観測を作る
fn empty_obs() -> RowObs {
    RowObs::empty()
}

/// 指定セルまでの20セルを新鮮なstunとして持つ合成観測を作る。
fn stun_obs_with_edge(fresh_edge: i32) -> RowObs {
    let mut obs = empty_obs();
    for back in 0..20usize {
        let cell = (fresh_edge as usize + CELL_COUNT - back) % CELL_COUNT;
        obs.states[cell] = CellState::Stun;
        obs.v[cell] = 252.0;
        obs.bright[cell] = BrightClass::Fresh;
    }
    obs.fresh_edge = fresh_edge;
    obs
}

fn mixed_state_for_game_frame(game_frame: usize) -> CellState {
    match game_frame {
        0..=26 => CellState::Counter,
        27..=31 => CellState::Active,
        32..=52 => CellState::PunishCounter,
        53..=55 => CellState::Counter,
        56..=58 => CellState::Active,
        _ => CellState::Stun,
    }
}

fn mixed_obs_with_edge(fresh_edge: i32) -> RowObs {
    let mut obs = empty_obs();
    for cell in 0..=fresh_edge as usize {
        obs.states[cell] = mixed_state_for_game_frame(cell);
        obs.v[cell] = 252.0;
        obs.bright[cell] = BrightClass::Fresh;
    }
    obs.fresh_edge = fresh_edge;
    obs
}

fn left_state_at_video_frame(tracker: &MeterTracker, video_frame: i64) -> Option<&str> {
    let &(segment_id, game_frame) = tracker.video_map.get(&video_frame)?;
    tracker
        .left
        .segments
        .iter()
        .find(|segment| segment.segment_id == segment_id)?
        .entries
        .iter()
        .find(|entry| entry.game_frame == game_frame)
        .map(|entry| entry.state.as_str())
}

#[test]
fn test_tracker_creates_segment_and_entries() {
    let mut tracker = MeterTracker::new();

    for video_frame in 0..15i64 {
        tracker.update(
            video_frame,
            stun_obs_with_edge(video_frame as i32),
            empty_obs(),
        );
    }
    tracker.finish();

    assert_eq!(tracker.left.segments.len(), 1);
    let left_entries = &tracker.left.segments[0].entries;
    assert_eq!(left_entries.len(), 15);
    assert_eq!(
        left_entries
            .iter()
            .map(|entry| (entry.game_frame, entry.state.as_str(), entry.confidence))
            .collect::<Vec<_>>(),
        (0..15)
            .map(|game_frame| {
                let confidence = match game_frame {
                    0..=11 => 1.0,
                    12..=13 => 0.9,
                    _ => 0.5,
                };
                (game_frame, "stun", confidence)
            })
            .collect::<Vec<_>>()
    );
}

#[test]
fn test_tracker_no_segment_before_stable_edge() {
    let mut tracker = MeterTracker::new();

    // 最初の1フレームだけ: まだセグメントが開かない
    tracker.update(0, stun_obs_with_edge(5), empty_obs());

    // セグメントはまだ開いていないはず
    // （2フレーム連続整合が必要なため）
    // → セグメントはまだあっても1フレーム観測で確定エントリはないはず
    tracker.finish();
    // finish後でも、2フレーム整合なしならセグメントなし
    assert!(
        tracker.left.segments.is_empty() || tracker.left.segments[0].entries.is_empty(),
        "should not have confirmed entries without 2 stable frames"
    );
}

#[test]
fn test_tracker_all_blackish_closes_segment() {
    let mut tracker = MeterTracker::new();

    for video_frame in 0..10i64 {
        tracker.update(
            video_frame,
            stun_obs_with_edge(video_frame as i32),
            empty_obs(),
        );
    }
    for video_frame in 10..15i64 {
        tracker.update(video_frame, empty_obs(), empty_obs());
    }
    for video_frame in 15..20i64 {
        tracker.update(
            video_frame,
            stun_obs_with_edge((video_frame - 15) as i32),
            empty_obs(),
        );
    }

    tracker.finish();
    assert_eq!(tracker.left.segments.len(), 2);
    assert_eq!(
        tracker
            .left
            .segments
            .iter()
            .map(|segment| segment.segment_id)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn test_tracker_state_string_values() {
    let mut tracker = MeterTracker::new();

    // stun 状態で観測
    let mut obs = empty_obs();
    for i in 0..10usize {
        obs.states[i] = CellState::Stun;
        obs.v[i] = 220.0;
        obs.bright[i] = BrightClass::Fresh;
    }
    obs.fresh_edge = 9;

    tracker.update(0, obs.clone(), obs.clone());
    tracker.update(1, obs.clone(), obs.clone());

    for vf in 2..30i64 {
        let mut o = obs.clone();
        let fe = (9 + vf as i32).min(79);
        for i in 0..=fe as usize {
            o.states[i] = CellState::Stun;
            o.v[i] = 220.0;
            o.bright[i] = BrightClass::Fresh;
        }
        o.fresh_edge = fe;
        tracker.update(vf, o.clone(), o.clone());
    }

    tracker.finish();

    if let Some(seg) = tracker.left.segments.first() {
        for entry in &seg.entries {
            // state は snake_case であるべき
            assert!(
                matches!(
                    entry.state.as_str(),
                    "stun"
                        | "unknown"
                        | "empty"
                        | "other"
                        | "counter"
                        | "punish_counter"
                        | "motion_recovery"
                        | "active"
                        | "projectile_active"
                        | "parry"
                        | "inv_full"
                        | "inv_strike"
                        | "inv_proj"
                ),
                "unexpected state string: {}",
                entry.state
            );
        }
    }
}

#[test]
fn test_circ_delta_wraps_correctly() {
    let mut tracker = MeterTracker::new();
    for video_frame in 0..100i64 {
        tracker.update(
            video_frame,
            stun_obs_with_edge((video_frame as usize % CELL_COUNT) as i32),
            empty_obs(),
        );
    }
    tracker.finish();

    assert_eq!(tracker.left.segments.len(), 1);
    let entries = &tracker.left.segments[0].entries;
    assert_eq!(entries.len(), 100);
    assert_eq!(entries.last().map(|entry| entry.game_frame), Some(99));
    assert!(entries
        .windows(2)
        .all(|window| window[0].game_frame < window[1].game_frame));
}

#[test]
fn test_video_map_contains_frame_refs() {
    let mut tracker = MeterTracker::new();

    for video_frame in 0..20i64 {
        tracker.update(
            video_frame,
            stun_obs_with_edge(video_frame as i32),
            empty_obs(),
        );
    }
    tracker.finish();

    assert_eq!(tracker.left.segments.len(), 1);
    for video_frame in 0..20i64 {
        assert_eq!(tracker.video_map[&video_frame], (0, video_frame));
    }
}

#[test]
fn test_tracker_hitstop_holds_game_frame() {
    let mut tracker = MeterTracker::new();
    let edges = [
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 9, 9, 9, 9, 9, 9, 9, 10, 11, 12, 13, 14,
    ];
    for (video_frame, edge) in edges.into_iter().enumerate() {
        tracker.update(video_frame as i64, stun_obs_with_edge(edge), empty_obs());
    }
    tracker.finish();

    for video_frame in 10..=16i64 {
        assert_eq!(tracker.video_map[&video_frame].1, 9);
    }
}

#[test]
fn test_tracker_f1104_state_transitions_without_video_fixture() {
    let mut tracker = MeterTracker::new();

    for video_frame in 1033..1040i64 {
        tracker.update(video_frame, empty_obs(), empty_obs());
    }

    // 元fixtureから時系列のfresh edgeだけを保存し、cell stateは意味的な区間として合成する。
    let edge_sequence = [
        0, 1, 3, 4, 5, 6, 6, 7, 9, 10, 11, 12, 13, 14, 14, 16, 17, 17, 19, 19, 20, 22, 23, 24, 24,
        25, 27, 28, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 29, 30, 30, 32, 33, 34, 35, 35,
        37, 38, 39, 40, 41, 42, 42, 44, 45, 45, 47, 47, 48, 49, 50, 51, 53, 54, 54, 55, 56, 57, 58,
        59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 59, 60, 60, 61, 62, 62, 62, 62, 62,
    ];
    for (offset, edge) in edge_sequence.into_iter().enumerate() {
        tracker.update(1040 + offset as i64, mixed_obs_with_edge(edge), empty_obs());
    }
    tracker.finish();

    assert_eq!(tracker.left.segments.len(), 1);
    assert_eq!(tracker.left.segments[0].entries.len(), 63);
    assert_eq!(left_state_at_video_frame(&tracker, 1040), Some("counter"));
    assert_eq!(left_state_at_video_frame(&tracker, 1066), Some("active"));
    assert_eq!(
        left_state_at_video_frame(&tracker, 1083),
        Some("punish_counter")
    );
    assert_eq!(left_state_at_video_frame(&tracker, 1104), Some("counter"));
    assert_eq!(left_state_at_video_frame(&tracker, 1108), Some("active"));
    assert_eq!(left_state_at_video_frame(&tracker, 1111), Some("stun"));
    assert_eq!(tracker.video_map[&1104].1, 53);
    assert_eq!(tracker.video_map[&1111].1, 59);
    assert_eq!(tracker.video_map[&1130].1, 62);

    let held_game_frames: Vec<i64> = (1068..=1080)
        .map(|video_frame| tracker.video_map[&video_frame].1)
        .collect();
    assert!(
        held_game_frames.iter().all(|game_frame| *game_frame == 29),
        "hitstop中にgame frameが進んだ: {held_game_frames:?}"
    );

    let mapped_game_frames: Vec<i64> = (1040..=1130)
        .filter_map(|video_frame| tracker.video_map.get(&video_frame).map(|entry| entry.1))
        .collect();
    assert_eq!(mapped_game_frames.len(), 91);
    assert!(
        mapped_game_frames
            .windows(2)
            .all(|window| window[0] <= window[1]),
        "game frameが単調増加していない: {mapped_game_frames:?}"
    );
}
