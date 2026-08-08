use super::*;

const ACTION_LOOKBACK: usize = 45;
const CONTACT_WINDOW: u32 = 120;
const RESULT_WINDOW: u32 = 360;
const PUNISH_WINDOW: u32 = 90;

pub(crate) struct SuperArtInputs<'a> {
    pub(crate) features: &'a [FrameFeatures],
    pub(crate) meter_state: &'a [Vec<MeterState>; 2],
    pub(crate) contacts: &'a [ContactEvent],
    pub(crate) damage: &'a [DamageEvent],
    pub(crate) punishes: &'a [PunishChance],
    pub(crate) rounds: &'a [RoundInfo],
    pub(crate) freeze_spans: &'a [(u32, u32)],
}

#[derive(Clone, Copy)]
struct ActionAnchor {
    frame: u32,
    evidence: bool,
}

pub(crate) fn extract_super_arts(inputs: SuperArtInputs<'_>) -> Vec<SuperArtEvent> {
    let SuperArtInputs {
        features,
        meter_state,
        contacts,
        damage,
        punishes,
        rounds,
        freeze_spans,
    } = inputs;
    let mut events = Vec::new();
    for side_index in 0..2 {
        let mut previous: Option<(usize, f32, bool)> = None;
        for (index, feature) in features.iter().enumerate() {
            let (value, uncertain, ca_ready) = gauge(feature, side_index);
            if !feature.is_match_screen || uncertain {
                continue;
            }
            let Some((previous_index, previous_value, previous_ca)) = previous else {
                previous = Some((index, value, ca_ready));
                continue;
            };
            let before_level = stock_level(previous_value);
            let after_level = stock_level(value);
            if after_level >= before_level
                || previous_value - value < crate::frame_features::MIN_SUPER_SPEND_DROP
            {
                previous = Some((index, value, ca_ready));
                continue;
            }

            let level = (before_level - after_level).clamp(1, 3);
            let anchor = action_anchor(
                features,
                meter_state.get(side_index).map_or(&[], Vec::as_slice),
                freeze_spans,
                previous_index,
                index,
            );
            let drop_frame = feature.frame_index;
            let round_at = round_of(rounds, anchor.frame).or_else(|| round_of(rounds, drop_frame));
            if let Some(round_no) = round_at {
                events.push(build_event(
                    side_index,
                    level,
                    previous_value,
                    value,
                    previous_ca,
                    drop_frame,
                    anchor,
                    round_no,
                    meter_state,
                    contacts,
                    damage,
                    punishes,
                    rounds,
                    features,
                ));
            }
            previous = Some((index, value, ca_ready));
        }
    }
    events.sort_by_key(|event| event.frame);
    events
}

#[allow(clippy::too_many_arguments)]
fn build_event(
    side_index: usize,
    level: u8,
    gauge_before: f32,
    gauge_after: f32,
    ca_ready: bool,
    gauge_drop_frame: u32,
    anchor: ActionAnchor,
    round_no: u32,
    meter_state: &[Vec<MeterState>; 2],
    contacts: &[ContactEvent],
    damage: &[DamageEvent],
    punishes: &[PunishChance],
    rounds: &[RoundInfo],
    features: &[FrameFeatures],
) -> SuperArtEvent {
    let side = side_index as u8 + 1;
    let opponent = 3 - side;
    let result_window_end = rounds
        .iter()
        .find(|round| round.round_no == round_no)
        .map_or(anchor.frame.saturating_add(RESULT_WINDOW), |round| {
            round
                .end_frame
                .min(anchor.frame.saturating_add(RESULT_WINDOW))
        });
    let contact_window_end = result_window_end.min(anchor.frame.saturating_add(CONTACT_WINDOW));
    let contact = contacts
        .iter()
        .filter(|contact| {
            contact.attacker == side
                && contact.round_no == round_no
                && contact.frame >= anchor.frame
                && contact.frame <= contact_window_end
        })
        .min_by_key(|contact| contact.frame);
    let attributable_damage: f32 = damage
        .iter()
        .filter(|event| {
            event.victim == opponent
                && event.round_no == round_no
                && event.pre_freeze_frame >= anchor.frame.saturating_sub(10)
                && event.pre_freeze_frame <= anchor.frame.saturating_add(30)
                && event.start_frame <= result_window_end
        })
        .map(|event| event.drop)
        .sum();
    let outcome = match contact {
        Some(contact) if contact.hit => SuperArtOutcome::Hit,
        Some(_) => SuperArtOutcome::Blocked,
        None if attributable_damage > 0.0 => SuperArtOutcome::Hit,
        None if anchor.evidence => SuperArtOutcome::NoImmediateContact,
        None => SuperArtOutcome::Unconfirmed,
    };
    let context = classify_context(
        side,
        side_index,
        anchor.frame,
        features,
        meter_state,
        contacts,
        punishes,
    );
    let response_frame = contact.map_or_else(
        || {
            recovery_start(
                features,
                &meter_state[side_index],
                anchor.frame,
                result_window_end,
            )
        },
        |contact| Some(contact.frame),
    );
    let punished_damage = if outcome == SuperArtOutcome::Hit {
        0.0
    } else {
        response_frame.map_or(0.0, |response| {
            damage
                .iter()
                .filter(|event| {
                    event.victim == side
                        && event.round_no == round_no
                        && event.start_frame >= response
                        && event.start_frame <= response.saturating_add(PUNISH_WINDOW)
                })
                .map(|event| event.drop)
                .fold(0.0, f32::max)
        })
    };
    let ko = outcome == SuperArtOutcome::Hit
        && rounds.iter().any(|round| {
            round.round_no == round_no
                && round.winner == Some(side)
                && round.end_frame <= result_window_end.saturating_add(30)
        });

    SuperArtEvent {
        side,
        frame: anchor.frame,
        gauge_drop_frame,
        level,
        critical_art: level == 3 && ca_ready,
        gauge_before,
        gauge_after,
        context,
        outcome,
        contact_frame: contact.map(|contact| contact.frame),
        damage: attributable_damage,
        ko,
        punished: punished_damage > 0.0,
        punished_damage,
        confidence: if anchor.evidence {
            EventConfidence::High
        } else {
            EventConfidence::Medium
        },
        round_no,
    }
}

fn action_anchor(
    features: &[FrameFeatures],
    states: &[MeterState],
    freeze_spans: &[(u32, u32)],
    previous_index: usize,
    current_index: usize,
) -> ActionAnchor {
    let previous_frame = features[previous_index].frame_index;
    let current_frame = features[current_index].frame_index;
    let freeze = freeze_spans
        .iter()
        .copied()
        .filter(|&(start, end)| end >= previous_frame.saturating_sub(5) && start <= current_frame)
        .max_by_key(|&(start, end)| (end - start, start));
    if let Some((start, _)) = freeze {
        return ActionAnchor {
            frame: start,
            evidence: true,
        };
    }
    if states.is_empty() {
        return ActionAnchor {
            frame: current_frame,
            evidence: false,
        };
    }

    let start = previous_index.saturating_sub(ACTION_LOOKBACK);
    let end = current_index.min(states.len().saturating_sub(1));
    for target in [MeterState::Invincible, MeterState::Startup] {
        if let Some(index) = (start..=end).rev().find(|&index| {
            states[index] == target
                && (index == 0 || states.get(index - 1).copied() != Some(target))
        }) {
            return ActionAnchor {
                frame: features[index].frame_index,
                evidence: true,
            };
        }
    }
    ActionAnchor {
        frame: current_frame,
        evidence: false,
    }
}

fn classify_context(
    side: u8,
    side_index: usize,
    frame: u32,
    features: &[FrameFeatures],
    meter_state: &[Vec<MeterState>; 2],
    contacts: &[ContactEvent],
    punishes: &[PunishChance],
) -> SuperArtContext {
    if punishes.iter().any(|punish| {
        punish.side == side
            && punish.outcome == PunishOutcome::Success
            && punish.frame <= frame.saturating_add(12)
            && punish.frame.saturating_add(45) >= frame
    }) {
        return SuperArtContext::Punish;
    }
    if contacts.iter().any(|contact| {
        contact.attacker == side
            && contact.hit
            && contact.frame < frame
            && contact.frame.saturating_add(90) >= frame
    }) {
        return SuperArtContext::Combo;
    }
    let defensive_meter = meter_state
        .get(side_index)
        .filter(|states| !states.is_empty())
        .is_some_and(|states| {
            let index = idx_of(features, frame);
            let start = index.saturating_sub(60);
            states
                .get(start..index.min(states.len()))
                .is_some_and(|span| span.contains(&MeterState::Stun))
        });
    let defensive_contact = contacts.iter().any(|contact| {
        contact.victim == side
            && contact.frame <= frame
            && contact.frame.saturating_add(90) >= frame
    });
    if defensive_meter || defensive_contact {
        SuperArtContext::DefensiveReversal
    } else if meter_state
        .get(side_index)
        .is_some_and(|states| !states.is_empty())
    {
        SuperArtContext::Neutral
    } else {
        SuperArtContext::Unknown
    }
}

fn recovery_start(
    features: &[FrameFeatures],
    states: &[MeterState],
    from: u32,
    to: u32,
) -> Option<u32> {
    if states.is_empty() {
        return None;
    }
    let start = idx_of(features, from);
    let end = idx_of(features, to).min(states.len().saturating_sub(1));
    (start..=end)
        .find(|&index| states[index] == MeterState::Recovery)
        .map(|index| features[index].frame_index)
}

fn stock_level(value: f32) -> u8 {
    value.clamp(0.0, 3.0).floor() as u8
}

fn gauge(feature: &FrameFeatures, side_index: usize) -> (f32, bool, bool) {
    if side_index == 0 {
        (
            feature.left_super_value,
            feature.left_super_uncertain,
            feature.left_ca_ready,
        )
    } else {
        (
            feature.right_super_value,
            feature.right_super_uncertain,
            feature.right_ca_ready,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::match_events::tests::support::feat;

    #[test]
    fn gauge_drop_and_meter_action_create_a_level_two_hit() {
        let features = features_with_spend(2.7, 0.7);
        let mut states = [vec![MeterState::Free; 100], vec![MeterState::Free; 100]];
        states[0][45..50].fill(MeterState::Invincible);
        let contacts = vec![ContactEvent {
            frame: 70,
            attacker: 1,
            victim: 2,
            hit: true,
            projectile: false,
            round_no: 1,
        }];
        let damage = vec![DamageEvent {
            victim: 2,
            start_frame: 70,
            pre_freeze_frame: 45,
            end_frame: 80,
            hp_before: 1.0,
            hp_after: 0.8,
            drop: 0.2,
            round_no: 1,
        }];
        let events = extract_super_arts(SuperArtInputs {
            features: &features,
            meter_state: &states,
            contacts: &contacts,
            damage: &damage,
            punishes: &[],
            rounds: &rounds(),
            freeze_spans: &[],
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, 2);
        assert_eq!(events[0].frame, 45);
        assert_eq!(events[0].outcome, SuperArtOutcome::Hit);
        assert_eq!(events[0].damage, 0.2);
        assert_eq!(events[0].confidence, EventConfidence::High);
        assert!(!events[0].critical_art);
    }

    #[test]
    fn ca_without_immediate_contact_is_not_called_a_whiff() {
        let mut features = features_with_spend(3.0, 0.2);
        for feature in &mut features[..50] {
            feature.left_ca_ready = true;
        }
        let states = [vec![MeterState::Free; 300], vec![MeterState::Free; 300]];
        let later_unrelated_contact = [ContactEvent {
            frame: 250,
            attacker: 1,
            victim: 2,
            hit: true,
            projectile: false,
            round_no: 1,
        }];
        let events = extract_super_arts(SuperArtInputs {
            features: &features,
            meter_state: &states,
            contacts: &later_unrelated_contact,
            damage: &[],
            punishes: &[],
            rounds: &rounds(),
            freeze_spans: &[(42, 60)],
        });

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].level, 3);
        assert!(events[0].critical_art);
        assert_eq!(events[0].frame, 42);
        assert_eq!(events[0].outcome, SuperArtOutcome::NoImmediateContact);
    }

    #[test]
    fn stock_boundary_jitter_does_not_create_a_super_event() {
        let features = features_with_spend(3.0, 2.94);
        let states = [vec![MeterState::Free; 300], vec![MeterState::Free; 300]];
        let events = extract_super_arts(SuperArtInputs {
            features: &features,
            meter_state: &states,
            contacts: &[],
            damage: &[],
            punishes: &[],
            rounds: &rounds(),
            freeze_spans: &[(42, 60)],
        });
        assert!(events.is_empty());
    }

    fn features_with_spend(before: f32, after: f32) -> Vec<FrameFeatures> {
        (0..300)
            .map(|index| {
                let mut feature = feat(index, 1.0, 1.0);
                feature.is_match_screen = true;
                feature.left_super_value = if index < 50 { before } else { after };
                feature.left_super_uncertain = false;
                feature
            })
            .collect()
    }

    fn rounds() -> Vec<RoundInfo> {
        vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 299,
            winner: None,
            p1_hp_end: 1.0,
            p2_hp_end: 1.0,
        }]
    }
}
