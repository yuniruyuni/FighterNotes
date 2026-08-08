use super::ATTACK_LOOKBACK;
use crate::frame_features::FrameFeatures;
use crate::match_events::{MatchEvents, MeterState};

#[derive(Debug, Clone, Copy)]
pub struct StartupObservation {
    pub frame: u32,
    pub startup: u32,
}

pub fn frame_index(features: &[FrameFeatures], frame: u32) -> Option<usize> {
    features
        .binary_search_by_key(&frame, |feature| feature.frame_index)
        .ok()
}

pub fn startup_observation(
    features: &[FrameFeatures],
    events: &MatchEvents,
    attacker: u8,
    contact_frame: u32,
) -> Option<StartupObservation> {
    let side = attacker as usize - 1;
    let states = &events.meter_state[side];
    let contact_index = frame_index(features, contact_frame)?;
    if contact_index >= states.len() {
        return None;
    }
    let search_start = contact_index.saturating_sub(ATTACK_LOOKBACK);
    let last_startup = (search_start..=contact_index)
        .rev()
        .find(|&index| states[index] == MeterState::Startup)?;
    if states[last_startup..=contact_index].iter().any(|state| {
        !matches!(
            state,
            MeterState::Startup | MeterState::Active | MeterState::Invincible
        )
    }) {
        return None;
    }
    let mut first_startup = last_startup;
    while first_startup > search_start && states[first_startup - 1] == MeterState::Startup {
        first_startup -= 1;
    }

    let raw_startup = contact_index.saturating_sub(first_startup) as u32 + 1;
    let startup = events.meter_game_frame[side]
        .get(first_startup)
        .zip(events.meter_game_frame[side].get(contact_index))
        .and_then(|(&start, &contact)| {
            (start >= 0 && contact >= start)
                .then(|| u32::try_from(contact - start).ok())
                .flatten()
        })
        .map_or(raw_startup, |game_frames| game_frames + 1);
    Some(StartupObservation {
        frame: features[first_startup].frame_index,
        startup,
    })
}
