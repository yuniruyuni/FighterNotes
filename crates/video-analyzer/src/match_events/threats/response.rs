use super::*;

pub(super) fn response_in_window(
    defender: usize,
    start: u32,
    end: u32,
    parry_runs: &[Vec<StateRun>; 2],
    inv_runs: &[Vec<StateRun>; 2],
) -> Option<DefenseResponse> {
    let mk = |run: &StateRun, kind| DefenseResponse {
        side: defender as u8 + 1,
        kind,
        start_frame: run.start,
        end_frame: run.end,
    };
    parry_runs[defender]
        .iter()
        .find(|run| run.end >= start && run.start <= end)
        .map(|run| mk(run, DefenseResponseKind::Parry))
        .or_else(|| {
            inv_runs[defender]
                .iter()
                .find(|run| run.end >= start && run.start <= end)
                .map(|run| mk(run, DefenseResponseKind::Invincible))
        })
}

pub(super) fn damage_assigned_to_contact<'a>(
    contact: &ContactEvent,
    contacts: &[ContactEvent],
    damage: &'a [DamageEvent],
) -> Option<&'a DamageEvent> {
    damage.iter().find(|event| {
        if event.victim != contact.victim
            || event.start_frame + 5 < contact.frame
            || event.start_frame > contact.frame.saturating_add(THREAT_DAMAGE_WINDOW)
        {
            return false;
        }
        // HP updates may lag a contact. If a later contact from the same
        // attacker is closer to that HP transition, assign the damage there.
        let nearest = contacts
            .iter()
            .filter(|candidate| {
                candidate.attacker == contact.attacker
                    && candidate.victim == contact.victim
                    && candidate.frame <= event.start_frame.saturating_add(5)
                    && event.start_frame <= candidate.frame.saturating_add(THREAT_DAMAGE_WINDOW)
            })
            .max_by_key(|candidate| candidate.frame);
        nearest.is_some_and(|nearest| nearest.frame == contact.frame)
    })
}
