use frame_meter::{CellState, RowObs, CELL_COUNT};

use crate::calibration::{
    LABEL_DIGIT_MIN, READ_DIM_CONF, READ_EARLY_CONF, READ_FADE_CONF, READ_FRESH_CONF,
};

use super::{insert_read, shared_pair, MeterTracker};

fn columns_with<F>(mut value: F) -> Vec<f32>
where
    F: FnMut(usize) -> f32,
{
    (0..CELL_COUNT).map(&mut value).collect()
}

#[test]
fn record_maps_wrapped_cell_and_assigns_read_confidences() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(159);
    let mut left = RowObs::empty();
    let mut right = RowObs::empty();
    left.states[79] = CellState::Active;
    right.states[79] = CellState::Stun;
    left.states[78] = CellState::Counter;
    left.states[77] = CellState::Parry;
    left.states[76] = CellState::PunishCounter;

    tracker.record(10, &left, &right, true, false);
    tracker.record(12, &left, &right, false, false);

    assert_eq!(tracker.video_map[&10], (0, 159));
    assert_eq!(tracker.dwell[&159], [10, 12]);
    assert_eq!(tracker.reads["left"][&159].0, "active");
    assert_eq!(tracker.reads["right"][&159].0, "stun");
    assert_eq!(tracker.reads["left"][&159].1, READ_FADE_CONF);
    assert_eq!(tracker.reads["left"][&158].1, READ_EARLY_CONF);
    assert_eq!(tracker.reads["left"][&157].1, READ_EARLY_CONF);
    assert_eq!(tracker.reads["left"][&156].1, READ_FRESH_CONF);
}

#[test]
fn advanced_record_resolves_other_or_rescued_state_with_matching_previous_side() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(5);
    let mut current_left = RowObs::empty();
    current_left.states[5] = CellState::Other;
    let mut correlations = vec![[-1.0f32; 10]; CELL_COUNT];
    correlations[5][4] = LABEL_DIGIT_MIN as f32;
    current_left.digit_corr = Some(correlations);
    current_left.cols = Some(columns_with(|cell| cell as f32));
    current_left.cols_w = 1;
    let mut previous_left = RowObs::empty();
    previous_left.cols = Some(columns_with(|cell| (cell + 1) as f32));
    previous_left.cols_w = 1;
    let mut previous_right = RowObs::empty();
    previous_right.cols = Some(columns_with(|cell| (cell * 40 + 20) as f32));
    previous_right.cols_w = 1;

    let mut current_right = RowObs::empty();
    current_right.states[5] = CellState::Counter;
    current_right.rescued[5] = true;
    current_right.cols = Some(columns_with(|cell| (cell * 40) as f32));
    current_right.cols_w = 1;
    tracker.previous = Some(shared_pair(previous_left, previous_right));

    tracker.record(10, &current_left, &current_right, true, true);

    assert_eq!(tracker.reads["left"][&5].0, "empty");
    assert!(
        !tracker.reads["left"][&5].2,
        "解決後のセルは数字覆いにしない"
    );
    assert_eq!(tracker.reads["right"][&5].0, "other");
}

#[test]
fn record_reads_previous_lap_dim_cells_only_within_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(80);
    let mut observation = RowObs::empty();
    observation.states[0] = CellState::Active;
    observation.states[78] = CellState::Counter;
    observation.states[79] = CellState::Stun;

    tracker.record(10, &observation, &observation, true, false);

    assert_eq!(tracker.reads["left"][&80].0, "active");
    assert_eq!(tracker.reads["left"][&78].1, READ_DIM_CONF);
    assert_eq!(tracker.reads["left"][&79].1, READ_DIM_CONF);
    assert!(!tracker.reads["left"][&79].2);
    assert!(!tracker.reads["left"].contains_key(&77));
}

#[test]
fn backfill_includes_the_farthest_cell_and_frame_zero() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(12);
    let mut observation = RowObs::empty();
    observation.states[0] = CellState::PunishCounter;

    tracker.record(10, &observation, &observation, true, false);

    assert_eq!(tracker.reads["left"][&0].0, "punish_counter");
}

#[test]
fn store_read_keeps_a_clear_existing_read_over_a_covered_replacement() {
    let mut tracker = MeterTracker::new();
    tracker.store_read("left", 5, "active".to_string(), 0.8, false);
    tracker.store_read("left", 5, "stun".to_string(), 1.0, true);

    assert_eq!(
        tracker.reads["left"][&5],
        ("active".to_string(), 0.8, false)
    );
}

#[test]
fn record_reads_dim_cells_at_later_lap_and_exact_window_boundary() {
    for absolute in [90, 160] {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(absolute);
        let mut observation = RowObs::empty();
        observation.states[78] = CellState::Counter;
        observation.states[79] = CellState::Stun;

        tracker.record(10, &observation, &observation, true, false);

        let previous_lap = absolute / CELL_COUNT as i64 - 1;
        assert_eq!(
            tracker.reads["left"][&(previous_lap * CELL_COUNT as i64 + 78)].1,
            READ_DIM_CONF
        );
        assert_eq!(
            tracker.reads["left"][&(previous_lap * CELL_COUNT as i64 + 79)].1,
            READ_DIM_CONF
        );
    }
}

#[test]
fn record_finalizes_entries_older_than_read_window() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(20);
    insert_read(&mut tracker, "left", 5, "stun", 1.0, false);
    tracker.dwell.insert(5, [2, 3]);

    tracker.record(20, &RowObs::empty(), &RowObs::empty(), false, false);

    assert!(!tracker.reads["left"].contains_key(&5));
    assert_eq!(tracker.left.segments[0].entries[0].game_frame, 5);
}

/// 数字が重なって色が読めないセルの観測。
fn covered_at(cell: usize) -> RowObs {
    let mut observation = RowObs::empty();
    let mut correlations = vec![[-1.0f32; 10]; CELL_COUNT];
    correlations[cell][4] = LABEL_DIGIT_MIN as f32;
    observation.digit_corr = Some(correlations);
    observation
}

/// 数字に覆われたセルは、覆われていたことを記録に残す。後段はこれを
/// 見て、色の読みをどこまで信じるか決める。
#[test]
fn a_cell_hidden_behind_a_number_is_recorded_as_covered() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(40);
    let mut observation = covered_at(40);
    observation.states[40] = CellState::Active;

    tracker.record(10, &observation, &observation, true, false);

    assert!(tracker.reads["left"][&40].2, "覆いを記録していない");
    assert!(
        !tracker.reads["left"][&39].2,
        "覆われていないセルまで印を付けている"
    );
}

/// 遡って埋めるセルにも、そのセルごとの覆いを記録する。
#[test]
fn the_backfilled_cells_carry_their_own_covering() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(40);
    let mut observation = covered_at(37);
    observation.states[37] = CellState::Counter;

    tracker.record(10, &observation, &observation, true, false);

    assert!(tracker.reads["left"][&37].2, "遡った先の覆いを落としている");
    assert!(!tracker.reads["left"][&38].2);
}

/// 覆われたセルの色は、直前のフレームの並びから引き当てる。ただし
/// メーターが進んだと分かっている場合だけ。止まっている間は引き当て
/// ようがない。
#[test]
fn a_hidden_cell_is_only_resolved_when_the_meter_advanced() {
    let build = |advanced: bool| {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(5);
        let mut observation = RowObs::empty();
        observation.states[5] = CellState::Other;
        observation.cols = Some(columns_with(|cell| cell as f32));
        observation.cols_w = 1;
        let mut previous = RowObs::empty();
        previous.cols = Some(columns_with(|cell| (cell + 1) as f32));
        previous.cols_w = 1;
        tracker.previous = Some(shared_pair(previous.clone(), previous));
        tracker.record(10, &observation, &observation, true, advanced);
        tracker.reads["left"][&5].0.clone()
    };

    assert_eq!(build(false), "other", "止まっている間に引き当てている");
    assert_ne!(build(true), "other", "進んだのに引き当てていない");
}

/// 票が割れたフレームは色を記録しない。時刻の対応だけ残す。
#[test]
fn a_frame_without_a_vote_records_only_the_timing() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(40);
    let mut observation = RowObs::empty();
    observation.states[40] = CellState::Active;

    tracker.record(10, &observation, &observation, false, false);

    assert_eq!(tracker.video_map[&10], (0, 40), "時刻の対応まで捨てている");
    assert!(
        !tracker.reads["left"].contains_key(&40),
        "票の割れたフレームの色を採っている"
    );
}

/// 遡って埋めるのは周回の先頭まで。前の周回のセルまで書き換えない。
#[test]
fn the_backfill_stops_at_the_start_of_the_lap() {
    let mut tracker = MeterTracker::new();
    // 2 周目の 6 セル目。遡れるのは周回の頭までの 5 セル。
    tracker.open_segment(CELL_COUNT as i64 + 5);
    let mut observation = RowObs::empty();
    observation.states.fill(CellState::Active);
    // 前の周回の読み直しが混ざらないよう、暗い位置は無表示にしておく。
    observation.states[78] = CellState::Empty;
    observation.states[79] = CellState::Empty;

    tracker.record(10, &observation, &observation, true, false);

    let lap_start = CELL_COUNT as i64;
    assert!(tracker.reads["left"].contains_key(&lap_start));
    assert!(
        !tracker.reads["left"].contains_key(&(lap_start - 1)),
        "前の周回まで遡って埋めている"
    );
}

/// 最初の周回では、まだ前の周回が無い。暗い表示の読み直しをしない。
#[test]
fn the_first_lap_has_no_previous_lap_to_re_read() {
    let mut tracker = MeterTracker::new();
    tracker.open_segment(0);
    let mut observation = RowObs::empty();
    observation.states[78] = CellState::Counter;
    observation.states[79] = CellState::Stun;

    tracker.record(10, &observation, &observation, true, false);

    assert!(tracker.reads["left"].keys().all(|absolute| *absolute >= 0));
}

/// 周回の頭を過ぎてしまえば、前の周回の暗い表示は読み直さない。既に
/// 確定させた区間へ後から書き戻さないための線引き。
#[test]
fn the_dim_re_read_only_happens_near_the_start_of_a_lap() {
    let reads_previous_lap = |cell: i64| {
        let mut tracker = MeterTracker::new();
        tracker.open_segment(CELL_COUNT as i64 + cell);
        let mut observation = RowObs::empty();
        observation.states[79] = CellState::Stun;
        tracker.record(10, &observation, &observation, true, false);
        tracker.reads["left"].contains_key(&79)
    };

    assert!(reads_previous_lap(11), "周回の頭で読み直していない");
    assert!(!reads_previous_lap(12), "周回の頭を過ぎても読み直している");
}
