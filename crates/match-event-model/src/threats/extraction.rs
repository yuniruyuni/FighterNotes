use super::*;

/// Extract persistent projectiles, character teleports, and their overlap.
///
/// Teleport recognition is intentionally character-gated. An arbitrary
/// `inv -> active` signature is also produced by DPs and supers; treating it as
/// a teleport would recreate the same semantic error this layer is meant to
/// remove. Additional characters can be added through explicit move profiles.
pub fn extract_threats(
    inputs: ThreatInputs<'_>,
) -> (
    Vec<ProjectileThreat>,
    Vec<TeleportEvent>,
    Vec<CompoundThreat>,
) {
    let ThreatInputs {
        features,
        timelines,
        meter_state,
        segments,
        jumps,
        contacts,
        damage,
        rounds,
        characters,
    } = inputs;
    let projectile_runs = [
        state_runs(timelines[0], |state| state == "projectile_active")
            .into_iter()
            .filter(|run| run.distinct_game_frames >= PROJECTILE_MIN_GAME_FRAMES)
            .collect::<Vec<_>>(),
        state_runs(timelines[1], |state| state == "projectile_active")
            .into_iter()
            .filter(|run| run.distinct_game_frames >= PROJECTILE_MIN_GAME_FRAMES)
            .collect::<Vec<_>>(),
    ];
    let inv_runs = [
        state_runs(timelines[0], |state| state.starts_with("inv_")),
        state_runs(timelines[1], |state| state.starts_with("inv_")),
    ];
    let parry_runs = [
        state_runs(timelines[0], |state| state == "parry"),
        state_runs(timelines[1], |state| state == "parry"),
    ];
    let stun_runs = [
        state_runs(timelines[0], |state| state == "stun"),
        state_runs(timelines[1], |state| state == "stun"),
    ];
    let active_runs = [
        state_runs(timelines[0], |state| state == "active"),
        state_runs(timelines[1], |state| state == "active"),
    ];

    let mut projectiles = Vec::new();
    for (owner, runs) in projectile_runs.iter().enumerate() {
        for run in runs {
            let Some(round_no) = round_of(rounds, run.start) else {
                continue;
            };
            projectiles.push(ProjectileThreat {
                owner: owner as u8 + 1,
                observed_start_frame: run.start,
                observed_end_frame: run.end,
                threat_end_frame: run.end.saturating_add(PROJECTILE_CARRY_WINDOW),
                contact_frame: None,
                round_no,
                confidence: 0.75,
            });
        }
    }

    // A parry immediately followed by defender stun is a stronger projectile
    // contact signal than a fixed carry window because meter evidence may end
    // before the projectile reaches the defender.
    for projectile in &mut projectiles {
        let defender = 2usize - projectile.owner as usize;
        let response = response_in_window(
            defender,
            projectile.observed_end_frame.saturating_sub(4),
            projectile
                .observed_end_frame
                .saturating_add(PROJECTILE_CONTACT_WINDOW),
            &parry_runs,
            &inv_runs,
        );
        let contact = response.as_ref().and_then(|response| {
            (response.kind == DefenseResponseKind::Parry)
                .then(|| {
                    stun_runs[defender].iter().find(|run| {
                        run.start >= response.end_frame
                            && run.start <= response.end_frame.saturating_add(2)
                    })
                })
                .flatten()
        });
        if let Some(contact) = contact {
            projectile.contact_frame = Some(contact.start);
            projectile.threat_end_frame = contact.start;
            projectile.confidence = 0.9;
        }
    }

    let mut teleports = Vec::new();
    for attacker in 0..2usize {
        if !is_dhalsim(characters[attacker]) {
            continue;
        }
        let defender = 1 - attacker;
        for inv in &inv_runs[attacker] {
            if inv.end.saturating_sub(inv.start).saturating_add(1) > TELEPORT_INV_MAX {
                continue;
            }
            let Some(input) = teleport_input(&segments[attacker], inv.start) else {
                continue;
            };
            let Some(round_no) = round_of(rounds, inv.start) else {
                continue;
            };
            let followup = active_runs[attacker]
                .iter()
                .find(|run| {
                    run.start >= inv.end
                        && run.start <= inv.end.saturating_add(TELEPORT_FOLLOWUP_WINDOW)
                })
                .map(|run| run.start);
            let projectile = projectiles.iter().find(|projectile| {
                projectile.owner == attacker as u8 + 1
                    && projectile.round_no == round_no
                    && projectile.observed_start_frame <= inv.start
                    && projectile.threat_end_frame >= inv.start
            });
            let before = idx_of(features, input.start_frame.saturating_sub(1));
            let defender_actionable = meter_state[defender]
                .get(before)
                .is_some_and(|state| matches!(state, MeterState::Free | MeterState::Parry));
            let airborne = jumps.iter().any(|jump| {
                jump.side == attacker as u8 + 1
                    && jump.takeoff_confirmed
                    && jump.frame <= input.start_frame
                    && jump.air_end >= inv.start
            });
            let followup_run = followup.and_then(|attack_frame| {
                active_runs[attacker]
                    .iter()
                    .find(|run| run.start == attack_frame)
            });
            let followup_contact = followup_run.and_then(|run| {
                contacts.iter().find(|contact| {
                    contact.attacker == attacker as u8 + 1
                        && contact.victim == defender as u8 + 1
                        && contact.frame >= run.start
                        && contact.frame <= run.end
                })
            });
            let hit = followup_contact
                .and_then(|contact| damage_assigned_to_contact(contact, contacts, damage));
            let outcome = match (followup, followup_contact, hit) {
                (_, Some(_), Some(_)) => ThreatOutcome::Hit,
                (_, Some(_), None) => ThreatOutcome::Defended,
                (Some(_), None, _) => ThreatOutcome::Whiffed,
                (None, _, _) => ThreatOutcome::Unknown,
            };
            let response = followup_contact.and_then(|contact| {
                response_in_window(
                    defender,
                    contact.frame.saturating_sub(2),
                    contact.frame.saturating_add(2),
                    &parry_runs,
                    &inv_runs,
                )
                .or_else(|| {
                    (outcome == ThreatOutcome::Defended).then_some(DefenseResponse {
                        side: defender as u8 + 1,
                        kind: DefenseResponseKind::Guard,
                        start_frame: contact.frame,
                        end_frame: contact.frame,
                    })
                })
            });
            let context = if followup.is_none() {
                TeleportContext::MovementOnly
            } else if !defender_actionable {
                TeleportContext::DefenderUnavailable
            } else if projectile.is_some() {
                TeleportContext::ProjectileCovered
            } else {
                TeleportContext::NakedAttack
            };
            teleports.push(TeleportEvent {
                attacker: attacker as u8 + 1,
                defender: defender as u8 + 1,
                input_frame: input.start_frame,
                inv_start_frame: inv.start,
                inv_end_frame: inv.end,
                followup_attack_frame: followup,
                followup_contact_frame: followup_contact.map(|contact| contact.frame),
                airborne,
                defender_actionable,
                context,
                response,
                outcome,
                damage: hit.map_or(0.0, |event| event.drop),
                dp_reachability: DpReachability::Unknown,
                round_no,
                confidence: 0.9,
            });
        }
    }
    teleports.sort_by_key(|event| event.input_frame);

    let compounds = teleports
        .iter()
        .filter_map(|teleport| {
            if teleport.context != TeleportContext::ProjectileCovered {
                return None;
            }
            let followup_attack_frame = teleport.followup_attack_frame?;
            let projectile = projectiles.iter().find(|projectile| {
                projectile.owner == teleport.attacker
                    && projectile.round_no == teleport.round_no
                    && projectile.observed_start_frame <= teleport.inv_start_frame
                    && projectile.threat_end_frame >= teleport.inv_start_frame
            })?;
            let projectile_response = response_in_window(
                teleport.defender as usize - 1,
                projectile.observed_end_frame.saturating_sub(4),
                projectile.contact_frame.unwrap_or(teleport.inv_end_frame),
                &parry_runs,
                &inv_runs,
            );
            // Without a visual projectile track, a parry/invincible response is
            // the confirmation that the old projectile actually reached this
            // setup. Recent projectile meter evidence alone remains tentative.
            projectile_response.as_ref()?;
            let projectile_contact = projectile.contact_frame?;
            if projectile_contact < teleport.input_frame
                || projectile_contact > followup_attack_frame
            {
                return None;
            }
            Some(CompoundThreat {
                attacker: teleport.attacker,
                defender: teleport.defender,
                projectile_start_frame: projectile.observed_start_frame,
                teleport_frame: teleport.input_frame,
                followup_attack_frame,
                followup_contact_frame: teleport.followup_contact_frame,
                projectile_response,
                followup_response: teleport.response.clone(),
                outcome: teleport.outcome,
                damage: teleport.damage,
                round_no: teleport.round_no,
                confidence: teleport.confidence.min(projectile.confidence),
            })
        })
        .collect();

    (projectiles, teleports, compounds)
}
