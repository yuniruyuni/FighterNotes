use super::*;
use crate::match_events::AttackDamageConsistency;

pub(crate) const MIN_DETECTOR_COVERAGE_PERCENT: u64 = 60;
/// SAの単フレーム値は演出やラベル遮蔽でuncertainになりやすい一方、
/// 時系列確定層は長い区間に散在する確実値から消費を検証できる。
pub(crate) const MIN_SUPER_COVERAGE_PERCENT: u64 = 20;
/// 空間カードは全候補フレームの連続認識ではなく、各イベント近傍の
/// 安定した距離サンプルで確定する。実動画で追跡可能な割合を踏まえた下限。
pub(crate) const MIN_SPATIAL_COVERAGE_PERCENT: u64 = 20;

pub(crate) fn detector_coverage_is_sufficient(observed: u32, total: u32) -> bool {
    total == 0 || u64::from(observed) * 100 >= u64::from(total) * MIN_DETECTOR_COVERAGE_PERCENT
}

pub(crate) fn super_coverage_is_sufficient(observed: u32, total: u32) -> bool {
    total == 0 || u64::from(observed) * 100 >= u64::from(total) * MIN_SUPER_COVERAGE_PERCENT
}

pub(crate) fn spatial_coverage_is_sufficient(observed: u32, total: u32) -> bool {
    total == 0 || u64::from(observed) * 100 >= u64::from(total) * MIN_SPATIAL_COVERAGE_PERCENT
}

pub(crate) fn build_coverage(
    features: &[FrameFeatures],
    events: &MatchEvents,
    own_index: usize,
    round_summaries: &[RoundSummary],
) -> (AnalysisCoverage, Vec<String>) {
    let match_frames = features
        .iter()
        .filter(|feature| feature.is_match_screen)
        .count() as u32;
    let analyzed_match_frames = features
        .iter()
        .filter(|feature| {
            feature.is_match_screen
                && crate::match_events::round_of(&events.rounds, feature.frame_index).is_some()
        })
        .count() as u32;
    let in_validated_round = |feature: &FrameFeatures| {
        feature.is_match_screen
            && crate::match_events::round_of(&events.rounds, feature.frame_index).is_some()
    };
    let input_segments = events.segments[own_index].len() as u32;
    let analyzed_input_segments = events.segments[own_index]
        .iter()
        .filter(|segment| {
            events.rounds.iter().any(|round| {
                segment.start_frame <= round.end_frame && segment.end_frame >= round.start_frame
            })
        })
        .count() as u32;
    let opponent_index = 1 - own_index;
    let detector_match_frames = analyzed_match_frames;
    let reliable_hp = |side: usize| {
        features
            .iter()
            .filter(|feature| in_validated_round(feature))
            .filter(|feature| {
                let (raw, quality) = if side == 0 {
                    (feature.left_hp_raw, feature.left_hp_raw_quality)
                } else {
                    (feature.right_hp_raw, feature.right_hp_raw_quality)
                };
                raw.is_finite() && (0.0..=1.0).contains(&raw) && quality < 0.5
            })
            .count() as u32
    };
    let reliable_drive = |side: usize| {
        features
            .iter()
            .filter(|feature| in_validated_round(feature))
            .filter(|feature| {
                if side == 0 {
                    !feature.left_drive_uncertain
                } else {
                    !feature.right_drive_uncertain
                }
            })
            .count() as u32
    };
    let reliable_super = |side: usize| {
        features
            .iter()
            .filter(|feature| in_validated_round(feature))
            .filter(|feature| {
                if side == 0 {
                    !feature.left_super_uncertain
                } else {
                    !feature.right_super_uncertain
                }
            })
            .count() as u32
    };
    let input_frames = |side: usize, repaired: bool| {
        if events.input_coverage.measured {
            return match (side, repaired) {
                (0, false) => events.input_coverage.p1_observed_frames,
                (1, false) => events.input_coverage.p2_observed_frames,
                (0, true) => events.input_coverage.p1_repaired_frames,
                (1, true) => events.input_coverage.p2_repaired_frames,
                _ => 0,
            }
            .min(detector_match_frames);
        }
        events.segments[side]
            .iter()
            .map(|segment| {
                let overlap: u32 = events
                    .rounds
                    .iter()
                    .map(|round| {
                        let start = segment.start_frame.max(round.start_frame);
                        let end = segment.end_frame.min(round.end_frame);
                        if start <= end {
                            end - start + 1
                        } else {
                            0
                        }
                    })
                    .sum();
                let observed = if repaired {
                    segment.evidence.repaired_frames
                } else {
                    segment.evidence.observed_frames
                };
                observed.min(overlap)
            })
            .sum::<u32>()
            .min(detector_match_frames)
    };
    let meter_frames = |side: usize| {
        features
            .iter()
            .filter(|feature| in_validated_round(feature))
            .filter(|feature| {
                events.meter_game_frame[side]
                    .get(feature.frame_index as usize)
                    .is_some_and(|game_frame| *game_frame >= 0)
            })
            .count() as u32
    };
    let last_super_is_reliable = |side: usize| {
        features
            .iter()
            .filter(|feature| in_validated_round(feature))
            .max_by_key(|feature| feature.frame_index)
            .is_some_and(|feature| {
                if side == 0 {
                    !feature.left_super_uncertain
                } else {
                    !feature.right_super_uncertain
                }
            })
    };
    let attack_damage_events = events.damage.len() as u32;
    let attack_evidence: Vec<_> = events.attack_evidence.damage.iter().collect();
    let coverage = AnalysisCoverage {
        match_frames,
        analyzed_match_frames,
        input_segments,
        analyzed_input_segments,
        detector_match_frames,
        own_hp_reliable_frames: reliable_hp(own_index),
        opponent_hp_reliable_frames: reliable_hp(opponent_index),
        own_drive_reliable_frames: reliable_drive(own_index),
        opponent_drive_reliable_frames: reliable_drive(opponent_index),
        own_super_reliable_frames: reliable_super(own_index),
        opponent_super_reliable_frames: reliable_super(opponent_index),
        own_super_end_reliable: last_super_is_reliable(own_index),
        opponent_super_end_reliable: last_super_is_reliable(opponent_index),
        own_input_observed_frames: input_frames(own_index, false),
        opponent_input_observed_frames: input_frames(opponent_index, false),
        own_input_repaired_frames: input_frames(own_index, true),
        opponent_input_repaired_frames: input_frames(opponent_index, true),
        own_meter_mapped_frames: meter_frames(own_index),
        opponent_meter_mapped_frames: meter_frames(opponent_index),
        spatial_candidate_frames: events.spatial_coverage.candidate_frames,
        spatial_sampled_frames: events.spatial_coverage.sampled_frames,
        spatial_usable_frames: events.spatial_coverage.usable_frames,
        own_spatial_observed_frames: if own_index == 0 {
            events.spatial_coverage.p1_observed_frames
        } else {
            events.spatial_coverage.p2_observed_frames
        },
        opponent_spatial_observed_frames: if opponent_index == 0 {
            events.spatial_coverage.p1_observed_frames
        } else {
            events.spatial_coverage.p2_observed_frames
        },
        attack_damage_events,
        attack_damage_linked: attack_evidence.len() as u32,
        attack_damage_consistent: attack_evidence
            .iter()
            .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Consistent)
            .count() as u32,
        attack_damage_mismatched: attack_evidence
            .iter()
            .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Mismatch)
            .count() as u32,
        attack_damage_unverified: attack_evidence
            .iter()
            .filter(|evidence| evidence.hp_consistency == AttackDamageConsistency::Unverified)
            .count() as u32,
    };
    let warnings = analysis_warnings(&coverage, events.rounds.len(), round_summaries);
    (coverage, warnings)
}

fn analysis_warnings(
    coverage: &AnalysisCoverage,
    rounds_detected: usize,
    round_summaries: &[RoundSummary],
) -> Vec<String> {
    let mut warnings = Vec::new();
    if rounds_detected == 0 {
        warnings.push("ラウンド境界を確定できなかったため、戦術統計を表示できません。".to_string());
    } else if coverage.match_frames > 0
        && u64::from(coverage.analyzed_match_frames) * 100 < u64::from(coverage.match_frames) * 70
    {
        let ratio =
            u64::from(coverage.analyzed_match_frames) * 100 / u64::from(coverage.match_frames);
        warnings.push(format!(
            "試合画面のうち解析ラウンドへ割り当てられた範囲は {ratio}% です。未検出ラウンドがないか確認してください。"
        ));
    }
    if round_summaries.iter().any(|round| round.won.is_none()) {
        warnings.push(
            "勝敗を確定できないラウンドがあります。ラウンド表の対象シーンを確認してください。"
                .to_string(),
        );
    }
    if coverage.attack_damage_events > 0
        && !detector_coverage_is_sufficient(
            coverage.attack_damage_linked,
            coverage.attack_damage_events,
        )
    {
        warnings.push(format!(
            "中央攻撃表示をHP被弾列へ帰属できた割合が60%未満です（{} / {} 件）。表示値に依存する統計とカードを確認不能として扱います。",
            coverage.attack_damage_linked, coverage.attack_damage_events
        ));
    }
    if coverage.attack_damage_mismatched > 0 {
        warnings.push(format!(
            "ゲーム内ダメージ表示とHPバー由来の減少量が一致しない被弾が {} 件あります。HP値は自動補正せず、ゲーム内表示を補助証拠として記録します。",
            coverage.attack_damage_mismatched
        ));
    }
    if coverage.attack_damage_unverified > 0 {
        warnings.push(format!(
            "ゲーム内ダメージ表示をHPバーと照合できなかった被弾が {} 件あります。これらの表示値は断定的な指摘には使いません。",
            coverage.attack_damage_unverified
        ));
    }
    let total = coverage.detector_match_frames;
    if total > 0 {
        if !detector_coverage_is_sufficient(
            coverage
                .own_hp_reliable_frames
                .min(coverage.opponent_hp_reliable_frames),
            total,
        ) {
            warnings.push(
                "HPバーの直接観測が確定ラウンドの60%未満です。HP由来の件数・割合を確認不能とし、依存するカードを抑制しています。"
                    .to_string(),
            );
        }
        if !detector_coverage_is_sufficient(
            coverage
                .own_drive_reliable_frames
                .min(coverage.opponent_drive_reliable_frames),
            total,
        ) {
            warnings.push(
                "Driveゲージの直接観測が確定ラウンドの60%未満です。読取不足側に依存するバーンアウト集計と指摘を確認不能として扱います。"
                    .to_string(),
            );
        }
        if !detector_coverage_is_sufficient(coverage.own_input_observed_frames, total) {
            warnings.push(
                "自分の入力履歴の直接観測が確定ラウンドの60%未満です。入力習慣統計と入力依存カードを抑制しています。"
                    .to_string(),
            );
        }
        if !detector_coverage_is_sufficient(coverage.opponent_input_observed_frames, total) {
            warnings.push(
                "相手の入力履歴の直接観測が確定ラウンドの60%未満です。相手入力に依存するカードを抑制しています。"
                    .to_string(),
            );
        }
        if !detector_coverage_is_sufficient(
            coverage
                .own_meter_mapped_frames
                .min(coverage.opponent_meter_mapped_frames),
            total,
        ) {
            warnings.push(
                "フレームメーターの対応付けが確定ラウンドの60%未満です。技の発生・硬直・接触に依存するカードを抑制しています。"
                    .to_string(),
            );
        }
        if !super_coverage_is_sufficient(coverage.own_super_reliable_frames, total)
            || !super_coverage_is_sufficient(coverage.opponent_super_reliable_frames, total)
        {
            warnings.push(
                "SAゲージの直接観測が不足しています。使用0回とは断定せず、読取不足側のSA/CA集計を確認不能として扱います。"
                    .to_string(),
            );
        }
    }
    if coverage.spatial_candidate_frames > 0
        && !detector_coverage_is_sufficient(
            coverage.spatial_sampled_frames,
            coverage.spatial_candidate_frames,
        )
    {
        warnings.push(
            "空間解析の候補区間を60%以上復号できませんでした。距離・到達可否に依存するカードを抑制しています。"
                .to_string(),
        );
    }
    if coverage.spatial_candidate_frames > 0
        && !spatial_coverage_is_sufficient(
            coverage.spatial_usable_frames,
            coverage.spatial_candidate_frames,
        )
    {
        warnings.push(
            "空間解析で両者の位置と距離を利用できた候補フレームが20%未満です。距離・到達可否に依存するカードを抑制しています。"
                .to_string(),
        );
    }
    warnings
}
