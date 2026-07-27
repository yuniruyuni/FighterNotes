use crate::palette::{nearest_palette, state_quality, PaletteName};
use crate::CellState;

use super::assert_close;

#[test]
fn palette_categories_and_colored_families_are_exhaustive() {
    for &name in PaletteName::all() {
        let colored = name.as_colored_entry();
        assert_eq!(name.state_family(), colored.clone().map(|entry| entry.0));
        assert_eq!(
            name.is_emptyish(),
            matches!(name, PaletteName::Black | PaletteName::Gap)
        );
        assert_eq!(
            name.is_whiteish(),
            matches!(
                name,
                PaletteName::White | PaletteName::WhiteDim | PaletteName::LabelBox
            )
        );
        assert_eq!(
            name.is_grayish(),
            matches!(name, PaletteName::Gray | PaletteName::GrayDim)
        );
        assert_eq!(
            name.is_stripe_pink(),
            matches!(name, PaletteName::StripePink | PaletteName::StripePinkDim)
        );
        assert_eq!(
            name.is_stripe_orange(),
            matches!(
                name,
                PaletteName::StripeOrange | PaletteName::StripeOrangeDim
            )
        );
    }
}

#[test]
fn nearest_palette_keeps_first_entry_when_distances_tie() {
    let midpoint = [127.748_64, 175.900_99, 16.650_03];
    let (name, distance) = nearest_palette(midpoint);

    assert_eq!(name, PaletteName::Counter);
    assert_close(distance, 31.122_257);
}

#[test]
fn state_quality_is_nearest_distance_within_requested_family() {
    let counter = PaletteName::Counter.color();
    let a = [counter[0] + 3.0, counter[1], counter[2]];
    let b = [counter[0], counter[1] + 4.0, counter[2]];

    assert_close(state_quality(&CellState::Counter, a, b), 3.0);
    assert_close(state_quality(&CellState::Empty, a, b), 0.0);
}
