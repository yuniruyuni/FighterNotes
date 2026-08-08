use super::{InputSegment, TELEPORT_INPUT_LOOKBACK};

pub fn is_dhalsim(character: Option<&str>) -> bool {
    character.is_some_and(|name| name.eq_ignore_ascii_case("DHALSIM"))
}

fn is_punch_badge(label: &str) -> bool {
    matches!(label, "弱P" | "中P" | "強P")
}

fn is_kick_badge(label: &str) -> bool {
    matches!(label, "弱K" | "中K" | "強K")
}

fn is_teleport_button_chord(segment: &InputSegment) -> bool {
    if segment.throw || segment.auto {
        return false;
    }
    let punches = segment
        .badges
        .iter()
        .filter(|badge| is_punch_badge(badge))
        .count();
    let kicks = segment
        .badges
        .iter()
        .filter(|badge| is_kick_badge(badge))
        .count();
    punches >= 2 || kicks >= 2
}

pub fn teleport_input(segments: &[InputSegment], inv_start: u32) -> Option<&InputSegment> {
    let mut candidates: Vec<&InputSegment> = segments
        .iter()
        .filter(|segment| {
            segment.start_frame <= inv_start.saturating_add(3)
                && segment.start_frame.saturating_add(TELEPORT_INPUT_LOOKBACK) >= inv_start
                && is_teleport_button_chord(segment)
        })
        .collect();
    candidates.sort_by_key(|segment| segment.start_frame);
    let mut cluster_start = *candidates.last()?;
    for candidate in candidates.iter().rev().skip(1) {
        if candidate.end_frame.saturating_add(2) < cluster_start.start_frame {
            break;
        }
        cluster_start = *candidate;
    }
    Some(cluster_start)
}
