use super::support::*;

// ─── fresh_color_edge ────────────────────────────────────────────────────────

fn empty_row() -> (Vec<f32>, Vec<CellState>, Vec<BrightClass>) {
    (
        vec![0.0; CELL_COUNT],
        vec![CellState::Empty; CELL_COUNT],
        vec![BrightClass::None_; CELL_COUNT],
    )
}

#[test]
fn edge_all_empty_returns_minus_one() {
    let (v, states, bright) = empty_row();
    assert_eq!(fresh_color_edge(&v, &states, &bright), -1);
}

#[test]
fn edge_single_fresh_mid_with_gap() {
    let (mut v, mut states, mut bright) = empty_row();
    // cell 5: Counter Fresh, v=200; cell 6: v=0（gap）
    v[5] = 200.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&v, &states, &bright), 5);
}

#[test]
fn edge_rightmost_wins() {
    let (mut v, mut states, mut bright) = empty_row();
    // cell 5 と 10 の両方が Counter Fresh, 10 の後ろはゼロ（gap）
    v[5] = 200.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Fresh;
    v[6] = 200.0; // 5 の後に連続 → has_front_gap(5) = false
    v[7] = 200.0;
    v[10] = 200.0;
    states[10] = CellState::Counter;
    bright[10] = BrightClass::Fresh;
    // v[11], v[12] は 0 → gap
    assert_eq!(fresh_color_edge(&v, &states, &bright), 10);
}

#[test]
fn edge_at_last_cell() {
    // cell 79 は次セルがないため常に has_front_gap = true
    let (mut v, mut states, mut bright) = empty_row();
    v[79] = 200.0;
    states[79] = CellState::Counter;
    bright[79] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&v, &states, &bright), 79);
}

#[test]
fn edge_low_with_prev_fresh_counts() {
    // cell 4 = Fresh, cell 5 = Low → 5 は「前が Fresh」ルールで is_fresh
    let (mut v, mut states, mut bright) = empty_row();
    v[4] = 200.0;
    states[4] = CellState::Counter;
    bright[4] = BrightClass::Fresh;
    v[5] = 200.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Low;
    // v[6] = 0 → gap after 5
    assert_eq!(fresh_color_edge(&v, &states, &bright), 5);
}

#[test]
fn edge_no_gap_not_returned() {
    // cell 5 が Fresh だが後続セルも明るく gap なし → -1
    let (mut v, mut states, mut bright) = empty_row();
    v[5] = 200.0;
    v[6] = 200.0;
    v[7] = 200.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&v, &states, &bright), -1);
}

#[test]
fn edge_stripe_inv_full_fresh_with_gap() {
    // InvFull は stripe 系なので wf >= STRIPE_WF_MIN → Fresh と判定される
    let (mut v, mut states, mut bright) = empty_row();
    v[20] = 220.0;
    states[20] = CellState::InvFull;
    bright[20] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&v, &states, &bright), 20);
}
