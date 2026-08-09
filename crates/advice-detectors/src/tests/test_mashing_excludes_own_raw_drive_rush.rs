use super::support::*;
use crate::match_events::{DriveRushEvent, DriveRushOutcome, EventConfidence, MatchEvents};

#[test]
fn own_raw_drive_rush_stopped_by_a_hit_is_not_called_defensive_mashing() {
    let mut events = one_mash_candidate();
    events.drive_rushes.push(rush(1, DriveRushOutcome::Stopped));

    assert!(
        detect_mashing(&[], &events, 1, 0).is_none(),
        "攻めるために出した自分の生ラッシュは守勢の暴れに帰属しない"
    );
}

#[test]
fn pressing_into_an_opponents_raw_drive_rush_can_still_be_defensive_mashing() {
    let mut events = one_mash_candidate();
    events.drive_rushes.push(rush(2, DriveRushOutcome::Hit));

    assert!(
        detect_mashing(&[], &events, 1, 0).is_some(),
        "相手の生ラッシュに対するボタンは従来どおり検討対象に残す"
    );
}

fn one_mash_candidate() -> MatchEvents {
    let mut events = basic_mashing_events();
    events.damage.retain(|damage| damage.start_frame <= 1000);
    events.segments[0].retain(|segment| segment.start_frame <= 990);
    events
}

fn rush(side: u8, outcome: DriveRushOutcome) -> DriveRushEvent {
    DriveRushEvent {
        side,
        frame: 970,
        raw: true,
        outcome,
        contact_frame: Some(999),
        damage: 0.12,
        confidence: EventConfidence::Medium,
        round_no: 1,
    }
}
