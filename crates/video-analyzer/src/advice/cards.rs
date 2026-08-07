use super::*;
use std::cmp::Ordering;

pub(crate) fn build_advice_cards(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own: u8,
    own_index: usize,
    own_character: Option<&str>,
    round_summaries: &[RoundSummary],
    coverage: &AnalysisCoverage,
) -> (Vec<AdviceCard>, Vec<SuppressedAdviceCard>) {
    let mut cards = Vec::new();
    let mut suppressed = Vec::new();
    let opp = 3 - own;
    let candidates = [
        detect_layered_defense(events, own),
        detect_teleport_defense(events, own),
        detect_anti_air(events, own, opp),
        detect_own_jumps(events, own),
        detect_burnout(events, own),
        detect_committed_button_vs_di(events, own, own_index),
        detect_mashing(features, events, own, own_index),
        detect_press_while_minus(events, own),
        detect_throw_while_minus(events, own),
        detect_advantage_abandoned(events, own),
        detect_guard_break(events, own),
        detect_reversal_punished(events, own),
        detect_low_scaling_super(events, own),
        detect_punish_fail(events, own, own_character),
        detect_punish_missed(events, own, own_character),
        detect_low_conversion(events, own),
        detect_throw_interrupted_by_invincible(events, own),
        detect_throw_whiff_punished(events, own),
        detect_whiff_punished(events, own),
        detect_throw_loop(events, opp),
        detect_early_hits(events, round_summaries, own),
        detect_lead_loss(events, round_summaries, own_index),
    ];
    for card in candidates.into_iter().flatten() {
        retain_or_suppress(card, coverage, &mut cards, &mut suppressed);
    }
    if let Some(card) = detect_big_hits(events, own, &cards) {
        retain_or_suppress(card, coverage, &mut cards, &mut suppressed);
    }
    sort_cards(&mut cards);
    (cards, suppressed)
}

fn retain_or_suppress(
    card: AdviceCard,
    coverage: &AnalysisCoverage,
    cards: &mut Vec<AdviceCard>,
    suppressed: &mut Vec<SuppressedAdviceCard>,
) {
    let missing_requirements = card_missing_requirements(&card, coverage);
    if missing_requirements.is_empty() {
        cards.push(card);
    } else {
        suppressed.push(SuppressedAdviceCard {
            id: card.id,
            title: card.title,
            missing_requirements,
        });
    }
}

fn card_missing_requirements(
    card: &AdviceCard,
    coverage: &AnalysisCoverage,
) -> Vec<EvidenceRequirement> {
    let total = coverage.detector_match_frames;
    let legacy_detector = |observed| detector_coverage_is_sufficient(observed, total);
    let status = coverage.availability.as_ref();
    let own_input = status.map_or_else(
        || legacy_detector(coverage.own_input_observed_frames),
        |value| value.own_input.is_available(),
    );
    let opponent_input = status.map_or_else(
        || legacy_detector(coverage.opponent_input_observed_frames),
        |value| value.opponent_input.is_available(),
    );
    let own_hp = status.map_or_else(
        || legacy_detector(coverage.own_hp_reliable_frames),
        |value| value.own_hp.is_available(),
    );
    let opponent_hp = status.map_or_else(
        || legacy_detector(coverage.opponent_hp_reliable_frames),
        |value| value.opponent_hp.is_available(),
    );
    let both_meter = status.map_or_else(
        || {
            legacy_detector(
                coverage
                    .own_meter_mapped_frames
                    .min(coverage.opponent_meter_mapped_frames),
            )
        },
        |value| value.own_meter.is_available() && value.opponent_meter.is_available(),
    );
    let own_drive = status.map_or_else(
        || legacy_detector(coverage.own_drive_reliable_frames),
        |value| value.own_drive.is_available(),
    );
    let own_super = status.map_or_else(
        || super_coverage_is_sufficient(coverage.own_super_reliable_frames, total),
        |value| value.own_super.is_available(),
    );
    let spatial = status.map_or_else(
        || {
            detector_coverage_is_sufficient(
                coverage.spatial_sampled_frames,
                coverage.spatial_candidate_frames,
            ) && spatial_coverage_is_sufficient(
                coverage.spatial_usable_frames,
                coverage.spatial_candidate_frames,
            )
        },
        |value| value.spatial.is_available(),
    );
    let contacts = status.map_or(both_meter && own_hp && opponent_hp, |value| {
        value.contacts.is_available()
    });
    let punishes = status.map_or(contacts && own_input && opponent_input, |value| {
        value.punishes.is_available()
    });
    let own_attack_info = status.map_or_else(
        || {
            detector_coverage_is_sufficient(
                coverage.attack_damage_linked,
                coverage.attack_damage_events,
            )
        },
        |value| value.own_attack_info.is_available(),
    );

    use EvidenceRequirement::*;
    let requirements = match card.id.as_str() {
        // 入力履歴から機会と実行を確定するカード。
        "anti_air" => vec![
            (opponent_input, OpponentInput),
            (own_hp, OwnHp),
            (opponent_hp, OpponentHp),
        ],
        "own_jumps" => vec![
            (own_input, OwnInput),
            (own_hp, OwnHp),
            (opponent_hp, OpponentHp),
        ],
        "throw_loop" => vec![(opponent_input, OpponentInput), (own_hp, OwnHp)],
        // 入力とフレームメーターの時系列が揃って初めて因果を述べられるカード。
        "committed_button_vs_di" => vec![
            (own_input, OwnInput),
            (opponent_input, OpponentInput),
            (both_meter, FrameMeter),
            (own_hp, OwnHp),
        ],
        "mashing"
        | "press_while_minus"
        | "throw_while_minus"
        | "advantage_abandoned"
        | "throw_interrupted_by_invincible"
        | "throw_whiff_punished" => vec![
            (own_input, OwnInput),
            (both_meter, FrameMeter),
            (own_hp, OwnHp),
        ],
        "guard_break" => vec![(own_input, OwnInput), (contacts, Contacts), (own_hp, OwnHp)],
        // 攻撃判定と接触の有無だけで成立する。入力表示は使わない。
        "whiff_punished" => vec![
            (both_meter, FrameMeter),
            (contacts, Contacts),
            (own_hp, OwnHp),
        ],
        // 発生・硬直・接触をフレームメーターから作るカード。
        "reversal_punished" => vec![(punishes, Punishes), (own_hp, OwnHp)],
        "low_conversion" => vec![(punishes, Punishes), (opponent_hp, OpponentHp)],
        // 到達距離まで断定するカードは候補区間の空間観測も必要。
        "punish_fail" => vec![(punishes, Punishes), (spatial, Spatial), (own_hp, OwnHp)],
        "teleport_defense" => vec![
            (opponent_input, OpponentInput),
            (both_meter, FrameMeter),
            (spatial, Spatial),
            (own_hp, OwnHp),
        ],
        "punish_missed" => vec![(punishes, Punishes), (spatial, Spatial), (own_hp, OwnHp)],
        // 複合攻撃の成立自体はmeter/contactから確定し、距離は使わない。
        "layered_defense" => vec![
            (opponent_input, OpponentInput),
            (contacts, Contacts),
            (own_hp, OwnHp),
        ],
        "burnout" => vec![
            (own_drive, OwnDrive),
            (own_hp, OwnHp),
            (opponent_hp, OpponentHp),
        ],
        "low_scaling_super" => vec![
            (own_super, OwnSuper),
            (contacts, Contacts),
            (own_attack_info, OwnAttackInfo),
            (opponent_hp, OpponentHp),
        ],
        "early_hits" | "big_hits" => vec![(own_hp, OwnHp)],
        "lead_loss" => vec![(own_hp, OwnHp), (opponent_hp, OpponentHp)],
        _ => Vec::new(),
    };
    requirements
        .into_iter()
        .filter_map(|(available, requirement)| (!available).then_some(requirement))
        .collect()
}

fn sort_cards(cards: &mut [AdviceCard]) {
    let kind_rank = |kind| match kind {
        AdviceKind::Diagnosis => 2,
        AdviceKind::Observation => 1,
        AdviceKind::Statistic => 0,
    };
    let confidence_rank = |confidence| match confidence {
        EventConfidence::High => 2,
        EventConfidence::Medium => 1,
        EventConfidence::Low => 0,
    };
    cards.sort_by(|left, right| {
        kind_rank(right.kind)
            .cmp(&kind_rank(left.kind))
            .then_with(|| confidence_rank(right.confidence).cmp(&confidence_rank(left.confidence)))
            .then_with(|| {
                right
                    .severity
                    .partial_cmp(&left.severity)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.id.cmp(&right.id))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn available_coverage_with_opponent_input_missing() -> AnalysisCoverage {
        let available = EvidenceAvailability::Available;
        AnalysisCoverage {
            availability: Some(AnalysisAvailability {
                own_hp: available,
                opponent_hp: available,
                own_drive: available,
                opponent_drive: available,
                own_super: available,
                opponent_super: available,
                own_input: available,
                opponent_input: EvidenceAvailability::Unavailable,
                own_meter: available,
                opponent_meter: available,
                contacts: available,
                punishes: available,
                spatial: available,
                own_attack_info: available,
                opponent_attack_info: available,
            }),
            ..AnalysisCoverage::default()
        }
    }

    fn card(id: &str) -> AdviceCard {
        AdviceCard {
            id: id.to_string(),
            kind: AdviceKind::Diagnosis,
            confidence: EventConfidence::High,
            title: id.to_string(),
            severity: 0.0,
            description: String::new(),
            practice: String::new(),
            evidence: Vec::new(),
        }
    }

    #[test]
    fn teleport_cards_require_the_opponents_observed_input() {
        let coverage = available_coverage_with_opponent_input_missing();

        for id in ["teleport_defense", "layered_defense"] {
            assert_eq!(
                card_missing_requirements(&card(id), &coverage),
                vec![EvidenceRequirement::OpponentInput]
            );
        }
    }
}
