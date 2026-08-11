use crate::rescue::dominant_color_family;
use crate::CellState;

#[test]
fn dominant_family_ignores_unassigned_pixels_and_averages_members() {
    let pixels = [
        [146, 201, 19],
        [130, 162, 49],
        [110, 151, 14],
        [146, 201, 19],
        [255, 0, 255],
        [255, 0, 255],
    ];

    assert_eq!(
        dominant_color_family(&pixels),
        Some((CellState::Counter, [133.0, 178.75, 25.25]))
    );
    assert_eq!(dominant_color_family(&[]), None);
}

#[test]
fn dominant_family_fraction_threshold_is_inclusive() {
    let mut accepted = vec![[255, 0, 255]; 20];
    accepted[..7].fill([93, 20, 176]);
    let mut rejected = accepted.clone();
    rejected[6] = [255, 0, 255];

    assert_eq!(
        dominant_color_family(&accepted),
        Some((CellState::Active, [93.0, 20.0, 176.0]))
    );
    assert_eq!(dominant_color_family(&rejected), None);
}

#[test]
fn palette_assignment_distance_is_inclusive() {
    assert_eq!(
        dominant_color_family(&[[191, 201, 19]; 3]),
        Some((CellState::Counter, [191.0, 201.0, 19.0]))
    );
    assert_eq!(dominant_color_family(&[[192, 201, 19]; 3]), None);
}

#[test]
fn dominant_family_keeps_first_palette_entry_when_distances_tie() {
    assert_eq!(
        dominant_color_family(&[[141, 123, 32]; 3]),
        Some((CellState::PunishCounter, [141.0, 123.0, 32.0]))
    );
}

#[test]
fn every_rescuable_color_family_round_trips() {
    for (pixel, expected) in [
        ([146, 201, 19], CellState::Counter),
        ([180, 112, 15], CellState::PunishCounter),
        ([237, 255, 88], CellState::MotionRecovery),
        ([93, 20, 176], CellState::Active),
        ([18, 127, 186], CellState::ProjectileActive),
        ([87, 17, 65], CellState::Parry),
        ([55, 255, 247], CellState::Stun),
    ] {
        assert_eq!(
            dominant_color_family(&[pixel; 3]),
            Some((
                expected,
                [pixel[0] as f32, pixel[1] as f32, pixel[2] as f32]
            ))
        );
    }
}
