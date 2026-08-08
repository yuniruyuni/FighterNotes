use super::{AttackInfoObservation, AttackInfoSide, AttackSequence, AttackSequenceStep};

#[derive(Debug)]
struct SequenceBuilder {
    sequence: AttackSequence,
    max_combo_seen: u32,
}

impl SequenceBuilder {
    fn new(attacker: u8, frame_index: u32, value: &AttackInfoSide) -> Self {
        Self {
            sequence: AttackSequence {
                attacker,
                start_frame: frame_index,
                end_frame: frame_index,
                closed_frame: None,
                combo_damage: value.combo_damage,
                last_damage: value.last_damage,
                final_scaling_percent: value.scaling_percent,
                starter_attribute: (value.combo_damage == value.last_damage)
                    .then_some(value.attribute),
                final_attribute: value.attribute,
                observation_count: 1,
                steps: vec![AttackSequenceStep {
                    frame_index,
                    last_damage: value.last_damage,
                    combo_damage: value.combo_damage,
                    scaling_percent: value.scaling_percent,
                    attribute: value.attribute,
                }],
                complete: false,
                recovered_from_max: false,
            },
            max_combo_seen: value.max_combo_damage,
        }
    }

    fn update(&mut self, frame_index: u32, value: &AttackInfoSide) {
        self.sequence.end_frame = frame_index;
        self.sequence.combo_damage = value.combo_damage;
        self.sequence.last_damage = value.last_damage;
        self.sequence.final_scaling_percent = value.scaling_percent;
        self.sequence.final_attribute = value.attribute;
        self.sequence.observation_count += 1;
        self.sequence.steps.push(AttackSequenceStep {
            frame_index,
            last_damage: value.last_damage,
            combo_damage: value.combo_damage,
            scaling_percent: value.scaling_percent,
            attribute: value.attribute,
        });
        self.max_combo_seen = self.max_combo_seen.max(value.max_combo_damage);
    }

    fn close(mut self, frame_index: u32, closing_value: &AttackInfoSide) -> AttackSequence {
        if closing_value.max_combo_damage > self.max_combo_seen
            && closing_value.max_combo_damage > self.sequence.combo_damage
            && closing_value.max_combo_damage > closing_value.combo_damage
        {
            self.sequence.combo_damage = closing_value.max_combo_damage;
            self.sequence.recovered_from_max = true;
        }
        self.sequence.closed_frame = Some(frame_index);
        self.sequence.complete = true;
        self.sequence
    }

    fn finish(self) -> AttackSequence {
        self.sequence
    }
}

/// P1/P2の表示状態列を、攻撃側ごとのコンボ列へ変換する。
pub fn build_attack_sequences(observations: &[AttackInfoObservation]) -> Vec<AttackSequence> {
    let mut sorted: Vec<&AttackInfoObservation> = observations.iter().collect();
    sorted.sort_by_key(|observation| observation.frame_index);

    let mut sequences = Vec::new();
    for side_index in 0..2 {
        let attacker = side_index as u8 + 1;
        let mut previous: Option<&AttackInfoSide> = None;
        let mut active: Option<SequenceBuilder> = None;
        for observation in &sorted {
            let current = if side_index == 0 {
                &observation.p1
            } else {
                &observation.p2
            };
            if previous == Some(current) {
                continue;
            }

            if current.combo_damage == 0 {
                if let Some(builder) = active.take() {
                    sequences.push(builder.close(observation.frame_index, current));
                }
                previous = Some(current);
                continue;
            }

            let starts_new = previous.is_some_and(|previous| {
                previous.combo_damage == 0
                    || current.combo_damage < previous.combo_damage
                    || (current.combo_damage == current.last_damage
                        && previous.combo_damage > 0
                        && current.scaling_percent >= previous.scaling_percent)
            });
            if starts_new {
                if let Some(builder) = active.take() {
                    sequences.push(builder.close(observation.frame_index, current));
                }
                active = Some(SequenceBuilder::new(
                    attacker,
                    observation.frame_index,
                    current,
                ));
            } else if let Some(builder) = active.as_mut() {
                if current.combo_damage >= builder.sequence.combo_damage {
                    builder.update(observation.frame_index, current);
                }
            } else {
                active = Some(SequenceBuilder::new(
                    attacker,
                    observation.frame_index,
                    current,
                ));
            }
            previous = Some(current);
        }
        if let Some(builder) = active {
            sequences.push(builder.finish());
        }
    }
    sequences.sort_by_key(|sequence| (sequence.start_frame, sequence.attacker));
    sequences
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attack_info::AttackAttribute;

    fn value(
        last_damage: u32,
        combo_damage: u32,
        max_combo_damage: u32,
        scaling_percent: u32,
        attribute: AttackAttribute,
    ) -> AttackInfoSide {
        AttackInfoSide {
            last_damage,
            scaling_percent,
            combo_damage,
            max_combo_damage,
            attribute,
        }
    }

    fn observation(frame_index: u32, p1: AttackInfoSide) -> AttackInfoObservation {
        AttackInfoObservation {
            frame_index,
            p1,
            p2: value(0, 0, 0, 100, AttackAttribute::Upper),
        }
    }

    #[test]
    fn builds_a_complete_sequence_and_keeps_the_starter_attribute() {
        let observations = vec![
            observation(100, value(600, 600, 600, 100, AttackAttribute::Lower)),
            observation(120, value(544, 1144, 1144, 68, AttackAttribute::Upper)),
            observation(180, value(0, 0, 1144, 68, AttackAttribute::Upper)),
        ];
        let sequences = build_attack_sequences(&observations);
        assert_eq!(sequences.len(), 1);
        assert_eq!(sequences[0].combo_damage, 1144);
        assert_eq!(sequences[0].starter_attribute, Some(AttackAttribute::Lower));
        assert_eq!(sequences[0].closed_frame, Some(180));
        assert!(sequences[0].complete);
    }

    #[test]
    fn separates_a_nonzero_combo_reset() {
        let observations = vec![
            observation(100, value(960, 960, 3030, 100, AttackAttribute::Upper)),
            observation(120, value(600, 1560, 3030, 100, AttackAttribute::Upper)),
            observation(160, value(120, 120, 3030, 80, AttackAttribute::Upper)),
            observation(200, value(0, 0, 3030, 80, AttackAttribute::Upper)),
        ];
        let sequences = build_attack_sequences(&observations);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].combo_damage, 1560);
        assert_eq!(sequences[0].closed_frame, Some(160));
        assert_eq!(sequences[1].combo_damage, 120);
    }

    #[test]
    fn recovers_a_missed_record_from_the_next_sequence() {
        let observations = vec![
            observation(100, value(600, 600, 600, 100, AttackAttribute::Lower)),
            observation(140, value(204, 1855, 1855, 51, AttackAttribute::Upper)),
            observation(300, value(720, 720, 2401, 100, AttackAttribute::Upper)),
        ];
        let sequences = build_attack_sequences(&observations);
        assert_eq!(sequences.len(), 2);
        assert_eq!(sequences[0].combo_damage, 2401);
        assert!(sequences[0].recovered_from_max);
        assert_eq!(sequences[1].combo_damage, 720);
    }

    #[test]
    fn leaves_the_last_unclosed_sequence_incomplete() {
        let observations = vec![observation(
            100,
            value(600, 600, 600, 100, AttackAttribute::Upper),
        )];
        let sequences = build_attack_sequences(&observations);
        assert_eq!(sequences.len(), 1);
        assert!(!sequences[0].complete);
        assert_eq!(sequences[0].closed_frame, None);
    }
}
