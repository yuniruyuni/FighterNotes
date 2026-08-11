use crate::{fresh_color_edge, BrightClass, CellState, CELL_COUNT};

fn row() -> (Vec<f32>, Vec<CellState>, Vec<BrightClass>) {
    (
        vec![0.0; CELL_COUNT],
        vec![CellState::Empty; CELL_COUNT],
        vec![BrightClass::None_; CELL_COUNT],
    )
}

#[test]
fn front_gap_uses_half_current_value_when_that_exceeds_empty_limit() {
    let (mut values, mut states, mut bright) = row();
    values[5] = 200.0;
    values[6] = 80.0;
    values[7] = 180.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Fresh;

    assert_eq!(
        fresh_color_edge(&values, &states, &bright),
        5,
        "80 is below half of the current value"
    );
}

#[test]
fn front_gap_limit_is_strict() {
    let (mut values, mut states, mut bright) = row();
    values[5] = 110.0;
    values[6] = 55.0;
    values[7] = 200.0;
    states[5] = CellState::Counter;
    bright[5] = BrightClass::Fresh;

    assert_eq!(fresh_color_edge(&values, &states, &bright), -1);
}

#[test]
fn freshness_requires_a_colored_state_and_valid_brightness_context() {
    let (mut values, mut states, mut bright) = row();
    values[5] = 200.0;
    bright[5] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&values, &states, &bright), -1);

    states[5] = CellState::Counter;
    bright[5] = BrightClass::Low;
    assert_eq!(fresh_color_edge(&values, &states, &bright), -1);

    states[0] = CellState::Counter;
    bright[0] = BrightClass::Low;
    assert_eq!(fresh_color_edge(&values, &states, &bright), -1);
}

#[test]
fn final_striped_and_carried_low_cells_can_be_edges() {
    let (mut values, mut states, mut bright) = row();
    values[CELL_COUNT - 1] = 220.0;
    states[CELL_COUNT - 1] = CellState::InvStrike;
    bright[CELL_COUNT - 1] = BrightClass::Fresh;
    assert_eq!(fresh_color_edge(&values, &states, &bright), 79);

    let (mut values, mut states, mut bright) = row();
    values[4] = 200.0;
    values[5] = 200.0;
    states[4] = CellState::Counter;
    states[5] = CellState::Counter;
    bright[4] = BrightClass::Fresh;
    bright[5] = BrightClass::Low;
    assert_eq!(fresh_color_edge(&values, &states, &bright), 5);
}
