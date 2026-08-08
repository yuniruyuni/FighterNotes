use super::model::{AttackInfoFrameInspection, AttackInfoObservation, AttackInfoSide};

const CONFIRM_FRAMES: u32 = 3;
const RESET_CONFIRM_FRAMES: u32 = 20;

#[derive(Debug, Clone)]
struct PendingSide {
    first_frame: u32,
    last_frame: u32,
    count: u32,
    value: AttackInfoSide,
}

#[derive(Debug, Default)]
struct SideTracker {
    pending: Option<PendingSide>,
    confirmed: Option<AttackInfoSide>,
}

#[derive(Debug)]
struct SideUpdate {
    frame_index: u32,
    value: AttackInfoSide,
}

impl SideTracker {
    fn observe(&mut self, frame_index: u32, mut value: AttackInfoSide) -> Option<SideUpdate> {
        if let Some(previous) = &self.confirmed {
            repair_leading_damage_digits(previous, &mut value);
            if !is_coherent_change(previous, &value) {
                self.pending = None;
                return None;
            }
        }

        let same_pending = self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.last_frame + 1 == frame_index && pending.value == value);
        if same_pending {
            let pending = self.pending.as_mut().expect("pending checked");
            pending.last_frame = frame_index;
            pending.count += 1;
        } else {
            if let Some(update) = self.promote_arithmetic_bridge(&value) {
                self.pending = Some(PendingSide {
                    first_frame: frame_index,
                    last_frame: frame_index,
                    count: 1,
                    value,
                });
                return Some(update);
            }
            self.pending = Some(PendingSide {
                first_frame: frame_index,
                last_frame: frame_index,
                count: 1,
                value,
            });
        }

        let pending = self.pending.as_ref().expect("pending initialized");
        let required_frames = self.confirmed.as_ref().map_or(CONFIRM_FRAMES, |previous| {
            if is_combo_reset(previous, &pending.value) {
                RESET_CONFIRM_FRAMES
            } else {
                CONFIRM_FRAMES
            }
        });
        if pending.count < required_frames
            || self
                .confirmed
                .as_ref()
                .is_some_and(|previous| *previous == pending.value)
        {
            return None;
        }
        Some(self.promote_pending())
    }

    fn miss(&mut self) -> Option<SideUpdate> {
        let pending = self.pending.take()?;
        if !is_self_authenticating_record(self.confirmed.as_ref(), &pending.value) {
            return None;
        }
        self.confirmed = Some(pending.value.clone());
        Some(SideUpdate {
            frame_index: pending.first_frame,
            value: pending.value,
        })
    }

    fn promote_arithmetic_bridge(&mut self, next: &AttackInfoSide) -> Option<SideUpdate> {
        let pending = self.pending.as_ref()?;
        if pending.count >= CONFIRM_FRAMES
            || !is_arithmetic_bridge(self.confirmed.as_ref(), &pending.value, next)
        {
            return None;
        }
        Some(self.promote_pending())
    }

    fn promote_pending(&mut self) -> SideUpdate {
        let pending = self.pending.take().expect("pending initialized");
        self.confirmed = Some(pending.value.clone());
        SideUpdate {
            frame_index: pending.first_frame,
            value: pending.value,
        }
    }
}

#[derive(Debug, Default)]
pub struct AttackInfoTracker {
    sides: [SideTracker; 2],
    pub observations: Vec<AttackInfoObservation>,
}

impl AttackInfoTracker {
    pub fn observe(&mut self, frame_index: u32, inspection: &AttackInfoFrameInspection) {
        let p1 = inspection.p1.as_ref().map(|side| side.value.clone());
        let p2 = inspection.p2.as_ref().map(|side| side.value.clone());
        self.observe_side(0, frame_index, p1);
        self.observe_side(1, frame_index, p2);
    }

    fn observe_side(&mut self, side: usize, frame_index: u32, value: Option<AttackInfoSide>) {
        let update = match value {
            Some(value) => self.sides[side].observe(frame_index, value),
            None => self.sides[side].miss(),
        };
        let Some(update) = update else {
            return;
        };
        debug_assert_eq!(self.sides[side].confirmed.as_ref(), Some(&update.value));
        let (Some(p1), Some(p2)) = (
            self.sides[0].confirmed.clone(),
            self.sides[1].confirmed.clone(),
        ) else {
            return;
        };
        let observation = AttackInfoObservation {
            frame_index: update.frame_index,
            p1,
            p2,
        };
        if self
            .observations
            .last()
            .is_some_and(|previous| previous.frame_index == observation.frame_index)
        {
            *self.observations.last_mut().expect("last checked") = observation;
        } else {
            self.observations.push(observation);
        }
    }
}

fn is_combo_reset(previous: &AttackInfoSide, current: &AttackInfoSide) -> bool {
    previous.combo_damage > 0 && current.combo_damage == 0
}

fn is_arithmetic_bridge(
    previous: Option<&AttackInfoSide>,
    candidate: &AttackInfoSide,
    next: &AttackInfoSide,
) -> bool {
    if candidate.combo_damage == 0 || next.combo_damage <= candidate.combo_damage {
        return false;
    }
    let candidate_starts_sequence = previous.is_none_or(|previous| {
        previous.combo_damage == 0
            || candidate.combo_damage < previous.combo_damage
            || candidate.scaling_percent > previous.scaling_percent
    });
    let candidate_delta_matches = if candidate_starts_sequence {
        candidate.combo_damage == candidate.last_damage
    } else {
        previous.is_some_and(|previous| {
            candidate.combo_damage > previous.combo_damage
                && candidate.combo_damage - previous.combo_damage == candidate.last_damage
        })
    };
    candidate_delta_matches
        && next.combo_damage - candidate.combo_damage == next.last_damage
        && next.scaling_percent <= candidate.scaling_percent
        && next.max_combo_damage >= candidate.max_combo_damage
}

fn is_self_authenticating_record(
    previous: Option<&AttackInfoSide>,
    candidate: &AttackInfoSide,
) -> bool {
    previous.is_some_and(|previous| {
        candidate.combo_damage > previous.combo_damage
            && candidate.combo_damage == candidate.max_combo_damage
            && candidate.max_combo_damage > previous.max_combo_damage
            && candidate.scaling_percent <= previous.scaling_percent
    })
}

fn repair_leading_damage_digits(previous: &AttackInfoSide, current: &mut AttackInfoSide) {
    let continued_combo_damage = (previous.combo_damage > 0
        && current.combo_damage > previous.combo_damage
        && current.scaling_percent <= previous.scaling_percent)
        .then(|| current.combo_damage - previous.combo_damage);
    let reset_damage = (current.combo_damage < previous.combo_damage
        && current.scaling_percent > previous.scaling_percent)
        .then_some(current.combo_damage);
    let Some(expected) = continued_combo_damage.or(reset_damage) else {
        return;
    };
    if is_strict_decimal_suffix(expected, current.last_damage) {
        current.last_damage = expected;
    }
}

fn is_strict_decimal_suffix(value: u32, suffix: u32) -> bool {
    if suffix == 0 || suffix >= value {
        return false;
    }
    let mut magnitude = 10u32;
    while magnitude <= suffix {
        magnitude = magnitude.saturating_mul(10);
    }
    value % magnitude == suffix
}

fn is_coherent_change(previous: &AttackInfoSide, current: &AttackInfoSide) -> bool {
    let same_hit_metadata = current.last_damage == previous.last_damage
        && current.scaling_percent == previous.scaling_percent
        && current.max_combo_damage == previous.max_combo_damage
        && current.attribute == previous.attribute;
    if same_hit_metadata
        && current.combo_damage < previous.combo_damage
        && current.combo_damage != current.last_damage
    {
        // 先頭桁だけが背景へ溶けた "2660 -> 660" 型の誤読を棄却する。
        return false;
    }
    if previous.combo_damage > 0
        && current.combo_damage > 0
        && current.combo_damage < previous.combo_damage
        && current.scaling_percent <= previous.scaling_percent
        && current.combo_damage != current.last_damage
    {
        // 継続中に補正率が下がりながら累積値だけ減ることはない。
        return false;
    }
    if current.last_damage != previous.last_damage
        && current.combo_damage == previous.combo_damage
        && current.scaling_percent == previous.scaling_percent
        && current.max_combo_damage == previous.max_combo_damage
        && current.attribute == previous.attribute
    {
        // 新しい攻撃なら累積値も更新される。単独変化は桁欠落か描画途中。
        return false;
    }
    if current.scaling_percent != previous.scaling_percent
        && current.last_damage == previous.last_damage
        && current.combo_damage == previous.combo_damage
        && current.max_combo_damage == previous.max_combo_damage
        && current.attribute == previous.attribute
    {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attack_info::{
        AttackAttribute, AttackInfoRoi, AttackInfoRois, AttackInfoSideInspection,
        AttackInfoSideRois,
    };

    fn side(damage: u32) -> AttackInfoSideInspection {
        AttackInfoSideInspection {
            value: AttackInfoSide {
                last_damage: damage,
                scaling_percent: 100,
                combo_damage: damage,
                max_combo_damage: damage,
                attribute: AttackAttribute::Upper,
            },
            numeric_score: 0,
            attribute_score: 0,
            attribute_margin: 100,
        }
    }

    fn inspection_sides(
        p1: Option<AttackInfoSideInspection>,
        p2: Option<AttackInfoSideInspection>,
    ) -> AttackInfoFrameInspection {
        let empty_roi = AttackInfoRoi {
            x1: 0,
            x2: 0,
            y1: 0,
            y2: 0,
        };
        AttackInfoFrameInspection {
            p1,
            p2,
            rois: AttackInfoRois {
                p1: AttackInfoSideRois {
                    numeric: empty_roi,
                    attribute: empty_roi,
                },
                p2: AttackInfoSideRois {
                    numeric: empty_roi,
                    attribute: empty_roi,
                },
            },
        }
    }

    fn inspection(damage: u32) -> AttackInfoFrameInspection {
        inspection_sides(Some(side(damage)), Some(side(damage)))
    }

    #[test]
    fn requires_three_consecutive_frames_and_keeps_change_origin() {
        let mut tracker = AttackInfoTracker::default();
        tracker.observe(10, &inspection(100));
        assert!(tracker.observations.is_empty());
        tracker.observe(11, &inspection(100));
        assert!(tracker.observations.is_empty());
        tracker.observe(12, &inspection(100));
        assert_eq!(tracker.observations[0].frame_index, 10);

        tracker.observe(13, &inspection(300));
        tracker.observe(14, &inspection(300));
        assert_eq!(tracker.observations.len(), 1);
        tracker.observe(15, &inspection(300));
        assert_eq!(tracker.observations.len(), 2);
        assert_eq!(tracker.observations[1].frame_index, 13);
    }

    #[test]
    fn confirms_one_side_while_the_other_is_temporarily_unreadable() {
        let mut tracker = AttackInfoTracker::default();
        for frame in 10..13 {
            tracker.observe(frame, &inspection(100));
        }
        for frame in 13..16 {
            tracker.observe(frame, &inspection_sides(Some(side(200)), None));
        }
        assert_eq!(tracker.observations.len(), 2);
        assert_eq!(tracker.observations[1].frame_index, 13);
        assert_eq!(tracker.observations[1].p1.combo_damage, 200);
        assert_eq!(tracker.observations[1].p2.combo_damage, 100);
    }

    #[test]
    fn keeps_a_short_first_hit_when_the_next_total_proves_it() {
        let mut tracker = AttackInfoTracker::default();
        for frame in 10..13 {
            tracker.observe(frame, &inspection(0));
        }
        let mut first = side(600);
        first.value.attribute = AttackAttribute::Lower;
        tracker.observe(13, &inspection_sides(Some(first), Some(side(0))));
        let mut second = side(544);
        second.value.combo_damage = 1144;
        second.value.max_combo_damage = 1144;
        second.value.scaling_percent = 68;
        tracker.observe(14, &inspection_sides(Some(second.clone()), Some(side(0))));
        tracker.observe(15, &inspection_sides(Some(second.clone()), Some(side(0))));
        tracker.observe(16, &inspection_sides(Some(second), Some(side(0))));

        assert_eq!(tracker.observations[1].frame_index, 13);
        assert_eq!(tracker.observations[1].p1.combo_damage, 600);
        assert_eq!(tracker.observations[1].p1.attribute, AttackAttribute::Lower);
        assert_eq!(tracker.observations[2].p1.combo_damage, 1144);
    }

    #[test]
    fn keeps_a_single_frame_new_record_when_the_panel_becomes_unreadable() {
        let mut tracker = AttackInfoTracker::default();
        let mut previous = side(204);
        previous.value.scaling_percent = 51;
        previous.value.combo_damage = 1855;
        previous.value.max_combo_damage = 1855;
        for frame in 10..13 {
            tracker.observe(
                frame,
                &inspection_sides(Some(previous.clone()), Some(side(0))),
            );
        }
        let mut record = side(252);
        record.value.scaling_percent = 42;
        record.value.combo_damage = 2401;
        record.value.max_combo_damage = 2401;
        tracker.observe(13, &inspection_sides(Some(record), Some(side(0))));
        tracker.observe(14, &inspection_sides(None, Some(side(0))));

        assert_eq!(tracker.observations.last().unwrap().frame_index, 13);
        assert_eq!(tracker.observations.last().unwrap().p1.combo_damage, 2401);
    }

    #[test]
    fn rejects_isolated_leading_digit_loss_but_allows_combo_reset() {
        let previous = AttackInfoSide {
            last_damage: 165,
            scaling_percent: 55,
            combo_damage: 2660,
            max_combo_damage: 2660,
            attribute: AttackAttribute::Upper,
        };
        assert!(!is_coherent_change(
            &previous,
            &AttackInfoSide {
                combo_damage: 660,
                ..previous.clone()
            }
        ));
        assert!(!is_coherent_change(
            &previous,
            &AttackInfoSide {
                last_damage: 65,
                ..previous.clone()
            }
        ));
        assert!(!is_coherent_change(
            &AttackInfoSide {
                last_damage: 204,
                scaling_percent: 68,
                combo_damage: 804,
                max_combo_damage: 2660,
                attribute: AttackAttribute::Upper,
            },
            &AttackInfoSide {
                last_damage: 4,
                scaling_percent: 59,
                combo_damage: 8,
                max_combo_damage: 2660,
                attribute: AttackAttribute::Upper,
            }
        ));
        assert!(is_coherent_change(
            &previous,
            &AttackInfoSide {
                last_damage: 600,
                scaling_percent: 100,
                combo_damage: 600,
                attribute: AttackAttribute::Lower,
                ..previous
            }
        ));
    }

    #[test]
    fn repairs_only_arithmetically_proven_damage_prefixes() {
        let previous = AttackInfoSide {
            last_damage: 637,
            scaling_percent: 75,
            combo_damage: 2397,
            max_combo_damage: 2397,
            attribute: AttackAttribute::Upper,
        };
        let mut continued = AttackInfoSide {
            last_damage: 7,
            combo_damage: 2584,
            max_combo_damage: 2584,
            ..previous.clone()
        };
        repair_leading_damage_digits(&previous, &mut continued);
        assert_eq!(continued.last_damage, 187);

        let mut unrelated = AttackInfoSide {
            last_damage: 80,
            combo_damage: 2827,
            ..previous.clone()
        };
        repair_leading_damage_digits(&previous, &mut unrelated);
        assert_eq!(unrelated.last_damage, 80);

        let mut reset = AttackInfoSide {
            last_damage: 60,
            scaling_percent: 100,
            combo_damage: 960,
            ..previous.clone()
        };
        repair_leading_damage_digits(&previous, &mut reset);
        assert_eq!(reset.last_damage, 960);
    }

    #[test]
    fn short_zero_reset_is_held_but_a_sustained_reset_is_confirmed() {
        let mut tracker = AttackInfoTracker::default();
        for frame in 10..13 {
            tracker.observe(frame, &inspection(100));
        }
        let mut reset = inspection(0);
        reset.p1.as_mut().unwrap().value.max_combo_damage = 100;
        reset.p2.as_mut().unwrap().value.max_combo_damage = 100;
        for frame in 13..13 + RESET_CONFIRM_FRAMES - 1 {
            tracker.observe(frame, &reset);
        }
        assert_eq!(tracker.observations.len(), 1);
        tracker.observe(13 + RESET_CONFIRM_FRAMES - 1, &reset);
        assert_eq!(tracker.observations.len(), 2);
        assert_eq!(tracker.observations[1].frame_index, 13);
    }
}
