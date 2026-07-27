use super::model::LowReturn;
use crate::advice::{COMBO_GAP, LOW_RETURN_DROP};
use crate::match_events::{MatchEvents, PunishChance};

fn continuous_hit_count(events: &MatchEvents, own: u8, punish_frame: u32) -> u32 {
    let mut hit_frames: Vec<_> = events
        .contacts
        .iter()
        .filter(|contact| {
            contact.attacker == own
                && contact.hit
                && contact.frame + 5 >= punish_frame
                && contact.frame <= punish_frame + 90
        })
        .map(|contact| contact.frame)
        .collect();
    hit_frames.sort_unstable();
    let mut count = 0;
    let mut last = None;
    for frame in hit_frames {
        if last.is_some_and(|last| frame > last + COMBO_GAP) {
            break;
        }
        count += 1;
        last = Some(frame);
    }
    count
}

pub(super) fn low_return(
    events: &MatchEvents,
    own: u8,
    punish: &PunishChance,
) -> Option<LowReturn> {
    let _hit_count = continuous_hit_count(events, own, punish.frame);
    let drop = events
        .damage
        .iter()
        .filter(|damage| {
            damage.victim == 3 - own
                && damage.start_frame + 5 >= punish.frame
                && damage.start_frame <= punish.frame + 120
        })
        .map(|damage| damage.drop)
        .fold(0.0_f32, f32::max);
    (drop > 0.0 && drop < LOW_RETURN_DROP).then(|| LowReturn {
        frame: punish.frame,
        round_no: punish.round_no,
        drop,
        input: punish.pressed.clone(),
    })
}
