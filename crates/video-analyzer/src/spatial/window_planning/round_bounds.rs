use crate::match_events::RoundInfo;

#[derive(Clone, Copy)]
pub(super) struct RoundBounds {
    pub(super) start: u32,
    pub(super) end: u32,
}

pub(super) fn for_round(rounds: &[RoundInfo], round_no: u32) -> RoundBounds {
    rounds
        .iter()
        .find(|round| round.round_no == round_no)
        .map_or(
            RoundBounds {
                start: 0,
                end: u32::MAX,
            },
            |round| RoundBounds {
                start: round.start_frame,
                end: round.end_frame,
            },
        )
}
