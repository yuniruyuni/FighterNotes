use crate::{
    attack_info::{build_attack_sequences, AttackInfoObservation, AttackSequence},
    frame_features::FrameFeatures,
};

use super::{
    round_of, AttackDamageConsistency, AttackEvidence, DamageAttackEvidence, DamageEvent,
    EventConfidence, RoundInfo, SuperArtAttackEvidence, SuperArtEvent, SuperArtOutcome, DEAD_HP,
    DMG_EPS, DMG_MIN_DROP,
};

const MAX_SEQUENCE_DISTANCE: u32 = 240;
const HIGH_CONFIDENCE_DISTANCE: u32 = 30;
const DELAYED_SEQUENCE_DISTANCE: u32 = 120;
const MIN_SCALE_SAMPLE_DROP: f32 = 0.02;
const MIN_PLAUSIBLE_HP: f32 = 7_000.0;
const MAX_PLAUSIBLE_HP: f32 = 15_000.0;
const MIN_POINT_TOLERANCE: f32 = 25.0;
const RELATIVE_POINT_TOLERANCE: f32 = 0.03;
const SPLIT_BOUNDARY_BEFORE: u32 = 12;
const SPLIT_BOUNDARY_AFTER: u32 = 24;
const SPLIT_HP_SCALE_SPREAD: f32 = 1.15;

#[derive(Debug)]
struct AssignedSequence {
    sequence_index: usize,
    distance: u32,
}

#[derive(Debug)]
struct EvidenceBuilder {
    damage_index: usize,
    assignments: Vec<AssignedSequence>,
}

pub fn build_attack_evidence(
    observations: &[AttackInfoObservation],
    damage: &[DamageEvent],
    rounds: &[RoundInfo],
) -> AttackEvidence {
    let sequences = build_attack_sequences(observations);
    if sequences.is_empty() || damage.is_empty() {
        return AttackEvidence {
            sequences,
            damage: Vec::new(),
            super_arts: Vec::new(),
        };
    }

    let mut builders: Vec<EvidenceBuilder> = (0..damage.len())
        .map(|damage_index| EvidenceBuilder {
            damage_index,
            assignments: Vec::new(),
        })
        .collect();
    for (sequence_index, sequence) in sequences.iter().enumerate() {
        let Some(round_no) = round_of(rounds, sequence.start_frame) else {
            continue;
        };
        let victim = 3 - sequence.attacker;
        let candidate = damage
            .iter()
            .enumerate()
            .filter(|(_, event)| event.victim == victim && event.round_no == round_no)
            .filter_map(|(damage_index, event)| {
                let distance =
                    interval_distance(sequence.start_frame, event.start_frame, event.end_frame);
                (distance <= MAX_SEQUENCE_DISTANCE).then_some((damage_index, distance))
            })
            .min_by_key(|(damage_index, distance)| {
                (
                    *distance,
                    damage[*damage_index]
                        .start_frame
                        .abs_diff(sequence.start_frame),
                )
            });
        if let Some((damage_index, distance)) = candidate {
            builders[damage_index].assignments.push(AssignedSequence {
                sequence_index,
                distance,
            });
        }
    }

    let mut evidence: Vec<Option<DamageAttackEvidence>> = builders
        .iter()
        .map(|builder| aggregate_evidence(builder, damage, &sequences))
        .collect();
    let hp_scale = estimate_hp_scales(&evidence, damage);
    for (damage_index, value) in evidence.iter_mut().enumerate() {
        let Some(value) = value.as_mut() else {
            continue;
        };
        value.hp_consistency = classify_consistency(
            value,
            &damage[damage_index],
            hp_scale[value.victim as usize - 1],
        );
        let max_distance = builders[damage_index]
            .assignments
            .iter()
            .map(|assignment| assignment.distance)
            .max()
            .unwrap_or(u32::MAX);
        if max_distance > DELAYED_SEQUENCE_DISTANCE
            && value.hp_consistency != AttackDamageConsistency::Consistent
        {
            *value = DamageAttackEvidence {
                confidence: EventConfidence::Low,
                ..value.clone()
            };
        }
    }

    AttackEvidence {
        sequences,
        damage: evidence
            .into_iter()
            .flatten()
            .filter(|value| value.confidence != EventConfidence::Low)
            .collect(),
        super_arts: Vec::new(),
    }
}

pub fn attach_super_art_evidence(
    attack_evidence: &mut AttackEvidence,
    super_arts: &[SuperArtEvent],
    damage: &[DamageEvent],
) {
    attack_evidence.super_arts.clear();
    for super_art in super_arts
        .iter()
        .filter(|super_art| super_art.outcome == SuperArtOutcome::Hit)
    {
        let linked = attack_evidence
            .damage
            .iter()
            .filter(|evidence| evidence.victim == 3 - super_art.side)
            .filter_map(|evidence| {
                let event = damage.iter().find(|damage| {
                    damage.victim == evidence.victim
                        && damage.start_frame == evidence.damage_start_frame
                        && damage.round_no == super_art.round_no
                })?;
                let in_result_window = event.start_frame >= super_art.frame.saturating_sub(10)
                    && event.start_frame <= super_art.frame.saturating_add(360);
                let freeze_matches = event.pre_freeze_frame.abs_diff(super_art.frame) <= 30;
                (in_result_window || freeze_matches)
                    .then_some((evidence, event.pre_freeze_frame.abs_diff(super_art.frame)))
            })
            .min_by_key(|(_, distance)| *distance)
            .map(|(evidence, _)| evidence);
        let Some(linked) = linked else {
            continue;
        };
        let sequence = linked
            .sequence_indices
            .iter()
            .filter_map(|index| attack_evidence.sequences.get(*index as usize))
            .filter(|sequence| sequence.attacker == super_art.side)
            .min_by_key(|sequence| {
                interval_distance(
                    super_art.frame,
                    sequence.start_frame,
                    sequence.closed_frame.unwrap_or(sequence.end_frame),
                )
            });
        let Some(sequence) = sequence else {
            continue;
        };
        let before = sequence
            .steps
            .iter()
            .filter(|step| step.frame_index < super_art.frame)
            .map(|step| step.combo_damage)
            .max()
            .unwrap_or(0);
        let entry_scaling_percent = sequence
            .steps
            .iter()
            .find(|step| step.frame_index >= super_art.frame.saturating_sub(5))
            .map(|step| step.scaling_percent);
        let marginal_damage =
            (sequence.complete && !sequence.recovered_from_max && sequence.combo_damage > before)
                .then_some(sequence.combo_damage - before);
        attack_evidence.super_arts.push(SuperArtAttackEvidence {
            side: super_art.side,
            super_frame: super_art.frame,
            combo_damage: sequence.combo_damage,
            marginal_damage,
            entry_scaling_percent,
            final_scaling_percent: sequence.final_scaling_percent,
            confidence: if linked.confidence == EventConfidence::High
                && sequence.complete
                && !sequence.recovered_from_max
            {
                EventConfidence::High
            } else {
                EventConfidence::Medium
            },
        });
    }
}

/// HPの連続下降が複数コンボを一つにまとめた場合だけ、中央表示のリセット位置で
/// 被弾イベントを分ける。中央表示だけを根拠にせず、各境界に実際のHP下降があり、
/// 分割後の各区間が通常の被弾下限を満たし、各コンボのpoint/HP比も揃う場合に限る。
///
/// 小さな追撃やチップまで無理に独立イベント化すると既存のアドバイス件数を
/// 不安定にするため、条件を満たさない場合は元イベントをそのまま保持する。
pub fn refine_damage_with_attack_evidence(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    damage: &mut Vec<DamageEvent>,
    evidence: &AttackEvidence,
) -> bool {
    if features.is_empty() {
        return false;
    }

    let mut changed = false;
    let mut refined = Vec::with_capacity(damage.len());
    for event in damage.drain(..) {
        let linked = evidence.damage.iter().find(|candidate| {
            candidate.victim == event.victim
                && candidate.damage_start_frame == event.start_frame
                && candidate.sequence_count >= 2
                && candidate.complete
                && !candidate.recovered_from_max
                && candidate.confidence == EventConfidence::High
                && candidate.hp_consistency != AttackDamageConsistency::Mismatch
        });
        let split = linked.and_then(|linked| {
            split_damage_event(features, hp, &event, linked, &evidence.sequences)
        });
        if let Some(events) = split {
            changed = true;
            refined.extend(events);
        } else {
            refined.push(event);
        }
    }
    refined.sort_by_key(|event| event.start_frame);
    *damage = refined;
    changed
}

fn split_damage_event(
    features: &[FrameFeatures],
    hp: &[Vec<f32>; 2],
    event: &DamageEvent,
    evidence: &DamageAttackEvidence,
    sequences: &[AttackSequence],
) -> Option<Vec<DamageEvent>> {
    let values = hp.get(event.victim as usize - 1)?;
    if values.len() != features.len() {
        return None;
    }

    let mut linked: Vec<&AttackSequence> = evidence
        .sequence_indices
        .iter()
        .filter_map(|index| sequences.get(*index as usize))
        .collect();
    linked.sort_by_key(|sequence| sequence.start_frame);
    if linked.len() != evidence.sequence_count as usize
        || linked.len() < 2
        || linked
            .iter()
            .any(|sequence| !sequence.complete || sequence.recovered_from_max)
    {
        return None;
    }

    let event_start = super::idx_of(features, event.start_frame);
    let event_end = super::idx_of(features, event.end_frame);
    let mut boundaries = Vec::with_capacity(linked.len() - 1);
    for sequence in linked.iter().skip(1) {
        let search_start = sequence
            .start_frame
            .saturating_sub(SPLIT_BOUNDARY_BEFORE)
            .max(event.start_frame);
        let search_end = sequence
            .start_frame
            .saturating_add(SPLIT_BOUNDARY_AFTER)
            .min(event.end_frame);
        let a = super::idx_of(features, search_start)
            .max(event_start.saturating_add(1))
            .max(
                boundaries
                    .last()
                    .copied()
                    .unwrap_or(event_start)
                    .saturating_add(1),
            );
        let b = super::idx_of(features, search_end).min(event_end);
        let boundary = (a..=b)
            .filter(|&index| values[index] < values[index - 1] - DMG_EPS)
            .min_by_key(|&index| features[index].frame_index.abs_diff(sequence.start_frame))?;
        boundaries.push(boundary);
    }

    let mut levels = Vec::with_capacity(boundaries.len() + 2);
    levels.push(event.hp_before);
    levels.extend(boundaries.iter().map(|&index| values[index - 1]));
    levels.push(event.hp_after);
    let drops: Vec<f32> = levels.windows(2).map(|pair| pair[0] - pair[1]).collect();
    if drops.len() != linked.len() || drops.iter().any(|drop| *drop < DMG_MIN_DROP) {
        return None;
    }

    // 同じ動画内ではpoint/HP比はほぼ一定になる。比率が揃わない境界は、
    // HPノイズまたは中央表示の別イベントへの誤帰属とみなす。
    let scales: Vec<f32> = linked
        .iter()
        .zip(&drops)
        .map(|(sequence, drop)| sequence.combo_damage as f32 / *drop)
        .collect();
    if scales
        .iter()
        .any(|scale| !(MIN_PLAUSIBLE_HP..=MAX_PLAUSIBLE_HP).contains(scale))
    {
        return None;
    }
    let min_scale = scales.iter().copied().fold(f32::INFINITY, f32::min);
    let max_scale = scales.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if max_scale > min_scale * SPLIT_HP_SCALE_SPREAD {
        return None;
    }

    let mut starts = Vec::with_capacity(linked.len());
    starts.push(event_start);
    starts.extend(boundaries.iter().copied());
    let mut ends = Vec::with_capacity(linked.len());
    for &boundary in &boundaries {
        let last_drop = (event_start.max(1)..boundary)
            .rev()
            .find(|&index| values[index] < values[index - 1] - DMG_EPS)?;
        ends.push(last_drop);
    }
    ends.push(event_end);

    Some(
        starts
            .into_iter()
            .zip(ends)
            .zip(drops)
            .enumerate()
            .map(|(index, ((start, end), drop))| DamageEvent {
                victim: event.victim,
                start_frame: features[start].frame_index,
                end_frame: features[end].frame_index,
                pre_freeze_frame: if index == 0 {
                    event.pre_freeze_frame
                } else {
                    features[start].frame_index
                },
                hp_before: levels[index],
                hp_after: levels[index + 1],
                drop,
                round_no: event.round_no,
            })
            .collect(),
    )
}

fn aggregate_evidence(
    builder: &EvidenceBuilder,
    damage: &[DamageEvent],
    sequences: &[AttackSequence],
) -> Option<DamageAttackEvidence> {
    let event = &damage[builder.damage_index];
    let mut assigned: Vec<(&AttackSequence, u32)> = builder
        .assignments
        .iter()
        .map(|assignment| (&sequences[assignment.sequence_index], assignment.distance))
        .collect();
    assigned.sort_by_key(|(sequence, _)| sequence.start_frame);
    let (first, _) = *assigned.first()?;
    let (last, _) = *assigned.last()?;
    let combo_damage = assigned
        .iter()
        .map(|(sequence, _)| sequence.combo_damage)
        .sum();
    let complete = assigned.iter().all(|(sequence, _)| sequence.complete);
    let recovered_from_max = assigned
        .iter()
        .any(|(sequence, _)| sequence.recovered_from_max);
    let max_distance = assigned
        .iter()
        .map(|(_, distance)| *distance)
        .max()
        .unwrap_or(u32::MAX);
    let confidence = if complete && !recovered_from_max && max_distance <= HIGH_CONFIDENCE_DISTANCE
    {
        EventConfidence::High
    } else {
        EventConfidence::Medium
    };
    Some(DamageAttackEvidence {
        victim: event.victim,
        attacker: 3 - event.victim,
        damage_start_frame: event.start_frame,
        sequence_start_frame: first.start_frame,
        sequence_end_frame: last.end_frame,
        combo_damage,
        sequence_count: assigned.len() as u32,
        final_scaling_percent: last.final_scaling_percent,
        starter_attribute: first.starter_attribute,
        final_attribute: last.final_attribute,
        complete,
        recovered_from_max,
        confidence,
        hp_consistency: AttackDamageConsistency::Unverified,
        sequence_indices: builder
            .assignments
            .iter()
            .map(|assignment| assignment.sequence_index as u32)
            .collect(),
    })
}

fn estimate_hp_scales(
    evidence: &[Option<DamageAttackEvidence>],
    damage: &[DamageEvent],
) -> [Option<f32>; 2] {
    let mut samples: [Vec<f32>; 2] = [Vec::new(), Vec::new()];
    for (damage_index, evidence) in evidence.iter().enumerate() {
        let Some(evidence) = evidence else {
            continue;
        };
        let event = &damage[damage_index];
        if !evidence.complete
            || evidence.recovered_from_max
            || event.drop < MIN_SCALE_SAMPLE_DROP
            || event.hp_after <= DEAD_HP
        {
            continue;
        }
        let estimate = evidence.combo_damage as f32 / event.drop;
        if (MIN_PLAUSIBLE_HP..=MAX_PLAUSIBLE_HP).contains(&estimate) {
            samples[event.victim as usize - 1].push(estimate);
        }
    }
    samples.map(|mut values| {
        if values.len() < 2 {
            return None;
        }
        values.sort_by(f32::total_cmp);
        let middle = values.len() / 2;
        Some(if values.len() % 2 == 0 {
            (values[middle - 1] + values[middle]) * 0.5
        } else {
            values[middle]
        })
    })
}

fn classify_consistency(
    evidence: &DamageAttackEvidence,
    damage: &DamageEvent,
    hp_scale: Option<f32>,
) -> AttackDamageConsistency {
    if damage.hp_after <= DEAD_HP {
        return AttackDamageConsistency::Unverified;
    }
    let Some(hp_scale) = hp_scale else {
        return AttackDamageConsistency::Unverified;
    };
    let expected = damage.drop * hp_scale;
    let actual = evidence.combo_damage as f32;
    let tolerance = MIN_POINT_TOLERANCE.max(actual * RELATIVE_POINT_TOLERANCE);
    if (expected - actual).abs() <= tolerance {
        AttackDamageConsistency::Consistent
    } else {
        AttackDamageConsistency::Mismatch
    }
}

fn interval_distance(frame: u32, start: u32, end: u32) -> u32 {
    if frame < start {
        start - frame
    } else {
        frame.saturating_sub(end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attack_info::{AttackAttribute, AttackInfoSide};
    use crate::SuperArtContext;

    fn side(last: u32, combo: u32, max: u32) -> AttackInfoSide {
        AttackInfoSide {
            last_damage: last,
            scaling_percent: 100,
            combo_damage: combo,
            max_combo_damage: max,
            attribute: AttackAttribute::Upper,
        }
    }

    fn observation(frame_index: u32, p1: AttackInfoSide) -> AttackInfoObservation {
        AttackInfoObservation {
            frame_index,
            p1,
            p2: side(0, 0, 0),
        }
    }

    fn damage(frame: u32, drop: f32, hp_after: f32) -> DamageEvent {
        DamageEvent {
            victim: 2,
            start_frame: frame,
            end_frame: frame + 30,
            pre_freeze_frame: frame,
            hp_before: hp_after + drop,
            hp_after,
            drop,
            round_no: 1,
        }
    }

    fn rounds() -> Vec<RoundInfo> {
        vec![RoundInfo {
            round_no: 1,
            start_frame: 0,
            end_frame: 1_000,
            winner: Some(1),
            p1_hp_end: 1.0,
            p2_hp_end: 0.0,
        }]
    }

    fn split_fixture(
        second_drop: f32,
        second_damage: u32,
    ) -> (
        Vec<FrameFeatures>,
        [Vec<f32>; 2],
        Vec<AttackInfoObservation>,
        Vec<DamageEvent>,
    ) {
        let mut p2_hp = vec![1.0; 61];
        p2_hp[10..35].fill(0.90);
        p2_hp[35..].fill(0.90 - second_drop);
        let features = p2_hp
            .iter()
            .enumerate()
            .map(|(index, hp)| match_event_model::test_support::feat(index as u32, 1.0, *hp))
            .collect();
        let observations = vec![
            observation(10, side(1000, 1000, 1000)),
            observation(20, side(0, 0, 1000)),
            observation(35, side(second_damage, second_damage, 1000)),
            observation(45, side(0, 0, 1000)),
        ];
        let damage = vec![DamageEvent {
            victim: 2,
            start_frame: 10,
            end_frame: 35,
            pre_freeze_frame: 10,
            hp_before: 1.0,
            hp_after: 0.90 - second_drop,
            drop: 0.10 + second_drop,
            round_no: 1,
        }];
        (features, [vec![1.0; 61], p2_hp], observations, damage)
    }

    #[test]
    fn links_exact_damage_and_detects_an_hp_outlier_from_the_video_scale() {
        let observations = vec![
            observation(100, side(600, 600, 600)),
            observation(120, side(600, 1200, 1200)),
            observation(160, side(0, 0, 1200)),
            observation(300, side(600, 600, 1200)),
            observation(320, side(600, 1200, 1200)),
            observation(360, side(0, 0, 1200)),
            observation(500, side(600, 600, 1200)),
            observation(520, side(600, 1200, 1200)),
            observation(560, side(0, 0, 1200)),
        ];
        let damage = vec![
            damage(100, 0.12, 0.88),
            damage(300, 0.12, 0.76),
            damage(500, 0.16, 0.60),
        ];
        let evidence = build_attack_evidence(&observations, &damage, &rounds());
        assert_eq!(evidence.damage.len(), 3);
        assert_eq!(
            evidence.damage[0].hp_consistency,
            AttackDamageConsistency::Consistent
        );
        assert_eq!(
            evidence.damage[2].hp_consistency,
            AttackDamageConsistency::Mismatch
        );
    }

    #[test]
    fn aggregates_two_panel_combos_inside_one_hp_sequence() {
        let observations = vec![
            observation(100, side(960, 960, 960)),
            observation(120, side(600, 1560, 1560)),
            observation(150, side(120, 120, 1560)),
            observation(180, side(0, 0, 1560)),
        ];
        let damage = vec![damage(100, 0.168, 0.832)];
        let evidence = build_attack_evidence(&observations, &damage, &rounds());
        assert_eq!(evidence.damage.len(), 1);
        assert_eq!(evidence.damage[0].sequence_count, 2);
        assert_eq!(evidence.damage[0].combo_damage, 1680);
    }

    #[test]
    fn ignores_a_distant_unverified_sequence() {
        let observations = vec![
            observation(350, side(600, 600, 600)),
            observation(370, side(0, 0, 600)),
        ];
        let damage = vec![damage(100, 0.12, 0.88)];
        let evidence = build_attack_evidence(&observations, &damage, &rounds());
        assert!(evidence.damage.is_empty());
    }

    #[test]
    fn splits_a_merged_hp_event_only_at_a_confirmed_material_hp_boundary() {
        let (features, hp, observations, mut damage) = split_fixture(0.08, 800);
        let evidence = build_attack_evidence(&observations, &damage, &rounds());
        assert_eq!(evidence.damage[0].sequence_count, 2);

        assert!(refine_damage_with_attack_evidence(
            &features,
            &hp,
            &mut damage,
            &evidence
        ));
        assert_eq!(damage.len(), 2);
        assert_eq!(damage[0].start_frame, 10);
        assert!((damage[0].drop - 0.10).abs() < 0.0001);
        assert_eq!(damage[1].start_frame, 35);
        assert!((damage[1].drop - 0.08).abs() < 0.0001);
    }

    #[test]
    fn keeps_a_small_follow_up_inside_the_original_hp_event() {
        let (features, hp, observations, mut damage) = split_fixture(0.012, 120);
        let evidence = build_attack_evidence(&observations, &damage, &rounds());
        assert_eq!(evidence.damage[0].sequence_count, 2);

        assert!(!refine_damage_with_attack_evidence(
            &features,
            &hp,
            &mut damage,
            &evidence
        ));
        assert_eq!(damage.len(), 1);
    }

    #[test]
    fn attributes_the_damage_added_after_a_super_was_inserted() {
        let mut before = side(600, 600, 600);
        before.scaling_percent = 100;
        let mut after = side(2000, 2600, 2600);
        after.scaling_percent = 40;
        let mut reset = side(0, 0, 2600);
        reset.scaling_percent = 40;
        let observations = vec![
            observation(100, before),
            observation(160, after),
            observation(220, reset),
        ];
        let mut damage = vec![damage(100, 0.26, 0.74)];
        damage[0].end_frame = 180;
        damage[0].pre_freeze_frame = 150;
        let mut evidence = build_attack_evidence(&observations, &damage, &rounds());
        let super_arts = vec![SuperArtEvent {
            side: 1,
            frame: 150,
            gauge_drop_frame: 155,
            level: 3,
            critical_art: false,
            gauge_before: 3.0,
            gauge_after: 0.0,
            context: SuperArtContext::Combo,
            outcome: SuperArtOutcome::Hit,
            contact_frame: Some(160),
            damage: 0.0,
            ko: false,
            punished: false,
            punished_damage: 0.0,
            confidence: EventConfidence::High,
            round_no: 1,
        }];

        attach_super_art_evidence(&mut evidence, &super_arts, &damage);

        assert_eq!(evidence.super_arts.len(), 1);
        assert_eq!(evidence.super_arts[0].combo_damage, 2600);
        assert_eq!(evidence.super_arts[0].marginal_damage, Some(2000));
        assert_eq!(evidence.super_arts[0].entry_scaling_percent, Some(40));
    }
}
