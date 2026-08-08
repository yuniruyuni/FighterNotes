use crate::advice::{
    AdviceCard, AdviceKind, EvidenceClip, MASH_METER_CONFIDENCE, MIN_REPEATED_NEGATIVE_OUTCOMES,
    OBSERVATION_REVIEW_CAVEAT,
};
use crate::match_events::{
    DriveImpactOutcome, EventConfidence, InputSegment, MatchEvents, MeterState,
};

use super::dir_arrow;

const RESULT_WINDOW: u32 = 80;
const INPUT_LOOKBACK: u32 = 2;
const INPUT_EXECUTION_LAG: u32 = 8;
const CONTACT_STATE_LOOKBACK: usize = 4;

struct CommittedButton {
    input_frame: u32,
    damage_end_frame: u32,
    round_no: u32,
    drop: f32,
    input: String,
}

fn normal_button(input: &InputSegment) -> Option<&str> {
    if input.throw || input.is_drive_impact() || input.badges.len() != 1 {
        return None;
    }
    matches!(
        input.badges[0].as_str(),
        "弱" | "中" | "強" | "弱P" | "中P" | "強P" | "弱K" | "中K" | "強K"
    )
    .then(|| input.badges[0].as_str())
}

fn normal_button_label(input: &InputSegment) -> Option<String> {
    let button = normal_button(input)?;
    Some(if input.dir == "N" {
        button.to_string()
    } else {
        format!("{}+{button}", dir_arrow(&input.dir))
    })
}

/// ボタンを保持したまま方向だけ離した場合、入力履歴は方向変更の境界で
/// 別セグメントになる。直前の隣接セグメントが同じボタンで方向だけ異なるなら、
/// ボタンを押した瞬間の方向とフレームまで戻す。
fn button_press_segment(
    segments: &[InputSegment],
    mut index: usize,
    earliest_frame: u32,
) -> &InputSegment {
    while index > 0 {
        let current = &segments[index];
        let previous = &segments[index - 1];
        let same_held_button = normal_button(previous) == normal_button(current)
            && previous.auto == current.auto
            && previous.dir != current.dir;
        let adjacent = previous.end_frame.saturating_add(1) >= current.start_frame;
        if !same_held_button
            || !adjacent
            || previous.start_frame < earliest_frame
            || !previous.evidence.has_direct_observation()
        {
            break;
        }
        index -= 1;
    }
    &segments[index]
}

fn execution_is_confirmed(
    events: &MatchEvents,
    own_index: usize,
    input: &InputSegment,
    contact_frame: u32,
) -> bool {
    let Some(states) = events.meter_state.get(own_index) else {
        return false;
    };
    if states.is_empty() {
        return false;
    }
    let confidence = &events.meter_confidence[own_index];
    let reliable = |frame: usize| {
        confidence.is_empty()
            || confidence
                .get(frame)
                .is_some_and(|value| *value >= MASH_METER_CONFIDENCE)
    };
    let startup_start = input.start_frame.saturating_sub(INPUT_LOOKBACK) as usize;
    let startup_end = input
        .end_frame
        .saturating_add(INPUT_EXECUTION_LAG)
        .min(contact_frame) as usize;
    let startup_seen = (startup_start..=startup_end)
        .any(|frame| reliable(frame) && states.get(frame) == Some(&MeterState::Startup));
    let contact = contact_frame as usize;
    let committed_at_contact =
        (contact.saturating_sub(CONTACT_STATE_LOOKBACK)..=contact).any(|frame| {
            reliable(frame)
                && matches!(
                    states.get(frame),
                    Some(MeterState::Startup | MeterState::Active | MeterState::Recovery)
                )
        });
    startup_seen && committed_at_contact
}

/// 相手DIそのものではなく、通常技を実行中でDIに取られた場面だけを提示する。
///
/// DI反応の失敗一般は統計に留める。入力表示・技発生・相手DIヒット・HP低下が
/// 揃った場合に限り、技を置いた距離やタイミングを見直す場面として扱う。
pub(crate) fn detect_committed_button_vs_di(
    events: &MatchEvents,
    own: u8,
    own_index: usize,
) -> Option<AdviceCard> {
    let opponent = 3 - own;
    let mut caught = Vec::new();
    for impact in events.drive_impacts.iter().filter(|impact| {
        impact.side == opponent
            && impact.outcome == DriveImpactOutcome::Hit
            && impact.confidence == EventConfidence::High
            && impact.damage > 0.0
    }) {
        let Some(contact_frame) = impact.contact_frame else {
            continue;
        };
        let Some(damage) = events
            .damage
            .iter()
            .filter(|damage| {
                damage.victim == own
                    && damage.round_no == impact.round_no
                    && damage.start_frame >= contact_frame.saturating_sub(2)
                    && damage.start_frame <= contact_frame.saturating_add(RESULT_WINDOW)
            })
            .min_by_key(|damage| damage.start_frame.abs_diff(contact_frame))
        else {
            continue;
        };
        let segments = &events.segments[own_index];
        let Some((input_index, _)) = segments
            .iter()
            .enumerate()
            .filter(|input| {
                input.1.evidence.has_direct_observation()
                    && input.1.start_frame >= impact.input_frame
                    && input.1.start_frame <= contact_frame
            })
            .filter(|(_, input)| normal_button(input).is_some())
            .filter(|(_, input)| execution_is_confirmed(events, own_index, input, contact_frame))
            .max_by_key(|(_, input)| input.start_frame)
        else {
            continue;
        };
        let input = button_press_segment(segments, input_index, impact.input_frame);
        let Some(label) = normal_button_label(input) else {
            continue;
        };
        caught.push(CommittedButton {
            input_frame: input.start_frame,
            damage_end_frame: damage.end_frame,
            round_no: damage.round_no,
            drop: damage.drop,
            input: label,
        });
    }
    if caught.is_empty() {
        return None;
    }

    let repeated = caught.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let hp_lost: f32 = caught.iter().map(|event| event.drop).sum();
    let common_input = caught
        .iter()
        .map(|event| event.input.as_str())
        .max_by_key(|candidate| {
            caught
                .iter()
                .filter(|event| event.input == *candidate)
                .count()
        })
        .unwrap_or("通常技");
    Some(AdviceCard {
        id: "committed_button_vs_di".to_string(),
        kind: if repeated {
            AdviceKind::Diagnosis
        } else {
            AdviceKind::Observation
        },
        confidence: EventConfidence::High,
        title: if repeated {
            "通常技の実行中にDIを繰り返し受けている"
        } else {
            "通常技の実行中にDIを受けた場面"
        }
        .to_string(),
        severity: hp_lost,
        hp_lost: Some(hp_lost),
        description: if repeated {
            format!(
                "入力表示とフレームメーターの両方で、通常技の実行中に相手DIがヒットした場面を {} 回確認し、合計 {:.0}% 被弾しています。最も多かった表示入力は {} でした。相手が技の出始めを見てDIしたのか、先に選んだDIと技がかみ合ったのかは、この時系列データだけでは断定できません。繰り返しているため、使用技のDIキャンセル可否と置く距離・頻度を見直す候補です。",
                caught.len(),
                hp_lost * 100.0,
                common_input
            )
        } else {
            format!(
                "入力表示では {}、フレームメーターでは通常技の実行中に相手DIがヒットし、{:.0}% 被弾した場面が1回あります。このデータだけでは、相手が技の出始めを見てDIしたのか、先に選んだDIと技がかみ合ったのかは{OBSERVATION_REVIEW_CAVEAT}。",
                common_input,
                hp_lost * 100.0
            )
        },
        practice: if repeated {
            "各クリップをスロー再生し、技が出始めた時点とDI演出開始の順序、その技のDIキャンセル可否を確認します。技が先でキャンセル不能なら置く距離・頻度を、DIが先またはキャンセル可能ならDI返し入力を練習しましょう。"
        } else {
            "クリップをスロー再生し、技が出始めた時点とDI演出開始の順序、その技のDIキャンセル可否を確認します。技が先でキャンセル不能なら置く距離・頻度を、DIが先またはキャンセル可能ならDI返し入力を個別に練習しましょう。"
        }
        .to_string(),
        evidence: caught
            .iter()
            .map(|event| EvidenceClip {
                frame: event.input_frame,
                end_frame: Some(event.damage_end_frame),
                label: format!(
                    "R{} {}中に相手DI→-{:.0}%",
                    event.round_no,
                    event.input,
                    event.drop * 100.0
                ),
            })
            .collect(),
    })
}
