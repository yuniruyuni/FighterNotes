use crate::advice::{
    MASH_INV_LOOKBACK, MASH_METER_CONFIDENCE, MASH_PROJECTILE_WINDOW, MASH_STARTUP_LAG,
    MASH_STARTUP_LEAD,
};
use crate::match_events::{DamageEvent, InputSegment, MatchEvents, MeterState};

pub(super) fn confirm_execution(
    events: &MatchEvents,
    own_index: usize,
    press: &InputSegment,
    damage: &DamageEvent,
) -> Option<bool> {
    if events.meter_state[0].is_empty() {
        return Some(false);
    }
    let state = &events.meter_state[own_index];
    let confidence = &events.meter_confidence[own_index];
    let reliable = |frame: usize| {
        confidence.is_empty()
            || confidence
                .get(frame)
                .is_some_and(|value| *value >= MASH_METER_CONFIDENCE)
    };
    let hit_frame = damage.start_frame as usize;
    let in_move_at_hit = (hit_frame.saturating_sub(3)..=hit_frame).any(|frame| {
        reliable(frame)
            && matches!(
                state.get(frame),
                Some(MeterState::Startup | MeterState::Active | MeterState::Recovery)
            )
    });
    if !in_move_at_hit {
        return None;
    }
    let startup_start = press.start_frame.saturating_sub(MASH_STARTUP_LEAD) as usize;
    let startup_end = press
        .end_frame
        .saturating_add(MASH_STARTUP_LAG)
        .min(damage.start_frame) as usize;
    (startup_start..=startup_end)
        .any(|frame| reliable(frame) && state.get(frame) == Some(&MeterState::Startup))
        .then_some(true)
}

pub(super) fn is_neutral_or_counterplay(
    events: &MatchEvents,
    own: u8,
    own_index: usize,
    press: &InputSegment,
    damage: &DamageEvent,
) -> bool {
    if events.meter_state[0].is_empty() {
        return false;
    }
    let opponent_state = &events.meter_state[2 - own as usize];
    let press_frame = press.start_frame as usize;
    if opponent_state.get(press_frame) == Some(&MeterState::Recovery) {
        return true;
    }
    let own_state = &events.meter_state[own_index];
    let projectile_end = (press_frame + MASH_PROJECTILE_WINDOW).min(own_state.len());
    if press_frame < projectile_end
        && own_state[press_frame..projectile_end].contains(&MeterState::ProjectileActive)
    {
        return true;
    }
    let damage_frame = (damage.start_frame as usize).min(opponent_state.len());
    let invincible_start = damage_frame.saturating_sub(MASH_INV_LOOKBACK);
    opponent_state[invincible_start..damage_frame].contains(&MeterState::Invincible)
}
