use crate::match_events::{
    DriveImpactOutcome, EventConfidence, InputSegment, MatchEvents, MeterState,
};
use crate::{
    AdviceCard, AdviceKind, EvidenceClip, MASH_METER_CONFIDENCE, MIN_REPEATED_NEGATIVE_OUTCOMES,
    OBSERVATION_REVIEW_CAVEAT,
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
    if input.throw || input.badges.len() != 1 {
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
    for previous_index in (0..index).rev() {
        let current = &segments[index];
        let previous = &segments[previous_index];
        let same_held_button = normal_button(previous) == normal_button(current)
            && previous.auto == current.auto
            && previous.dir != current.dir;
        let adjacent = previous.end_frame.saturating_add(1) >= current.start_frame;
        if !same_held_button
            || !adjacent
            || previous.start_frame < earliest_frame
            || !previous.evidence.has_direct_observation()
        {
            return &segments[index];
        }
        index = previous_index;
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
pub fn detect_committed_button_vs_di(
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
        if let Some(contact_frame) = impact.contact_frame {
            if let Some(damage) = events
                .damage
                .iter()
                .filter(|damage| {
                    damage.victim == own
                        && damage.round_no == impact.round_no
                        && damage.start_frame >= contact_frame.saturating_sub(2)
                        && damage.start_frame <= contact_frame.saturating_add(RESULT_WINDOW)
                })
                .min_by_key(|damage| damage.start_frame.abs_diff(contact_frame))
            {
                let segments = &events.segments[own_index];
                if let Some((input_index, _)) = segments
                    .iter()
                    .enumerate()
                    .filter(|input| {
                        input.1.evidence.has_direct_observation()
                            && input.1.start_frame >= impact.input_frame
                            && input.1.start_frame <= contact_frame
                    })
                    .filter(|(_, input)| normal_button(input).is_some())
                    .filter(|(_, input)| {
                        execution_is_confirmed(events, own_index, input, contact_frame)
                    })
                    .max_by_key(|(_, input)| input.start_frame)
                {
                    let input = button_press_segment(segments, input_index, impact.input_frame);
                    if let Some(label) = normal_button_label(input) {
                        caught.push(CommittedButton {
                            input_frame: input.start_frame,
                            damage_end_frame: damage.end_frame,
                            round_no: damage.round_no,
                            drop: damage.drop,
                            input: label,
                        });
                    }
                }
            }
        }
    }
    if caught.is_empty() {
        return None;
    }

    let repeated = caught.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let kind = if repeated {
        AdviceKind::Diagnosis
    } else {
        AdviceKind::Observation
    };
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
        kind,
        confidence: EventConfidence::High,
        title: match kind {
            AdviceKind::Diagnosis => "通常技の実行中にDIを繰り返し受けている",
            _ => "通常技の実行中にDIを受けた場面",
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
        practice: match kind {
            AdviceKind::Diagnosis => "各クリップをスロー再生し、技が出始めた時点とDI演出開始の順序、その技のDIキャンセル可否を確認します。技が先でキャンセル不能なら置く距離・頻度を、DIが先またはキャンセル可能ならDI返し入力を練習しましょう。",
            _ => "クリップをスロー再生し、技が出始めた時点とDI演出開始の順序、その技のDIキャンセル可否を確認します。技が先でキャンセル不能なら置く距離・頻度を、DIが先またはキャンセル可能ならDI返し入力を個別に練習しましょう。",
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

/// 壁やられからの確定反撃をこの場面の損失として帰属する窓。壁やられの
/// のけぞりが数十フレーム続くため、直後被弾の窓(80F)より長く取る。
const WALL_SPLAT_RESULT_WINDOW: u32 = 150;

/// 画面端で相手のDIをガードした場面。
///
/// 中央なら押し返されて終わる同じガードが、端では壁やられになり確定反撃を
/// 献上する。ガード入力は正しいので操作の失敗ではなく、端を背負ったときに
/// DIへの返し(DI返し・パリィ・無敵技)を用意していなかったことを指摘する。
pub fn detect_cornered_di_guard(events: &MatchEvents, own: u8) -> Option<AdviceCard> {
    let opponent = 3 - own;
    let mut guarded = Vec::new();
    for impact in events.drive_impacts.iter().filter(|impact| {
        impact.side == opponent
            && impact.outcome == DriveImpactOutcome::Blocked
            && impact.confidence == EventConfidence::High
    }) {
        let Some(contact_frame) = impact.contact_frame else {
            continue;
        };
        if !events.cornered_at(own, contact_frame, crate::CORNERED_DAMAGE_TAIL) {
            continue;
        }
        let drop: f32 = events
            .damage
            .iter()
            .filter(|damage| {
                damage.victim == own
                    && damage.round_no == impact.round_no
                    && damage.start_frame >= contact_frame
                    && damage.start_frame <= contact_frame.saturating_add(WALL_SPLAT_RESULT_WINDOW)
            })
            .map(|damage| damage.drop)
            .sum();
        guarded.push((impact, drop));
    }
    if guarded.is_empty() {
        return None;
    }
    let repeated = guarded.len() >= MIN_REPEATED_NEGATIVE_OUTCOMES;
    let kind = if repeated {
        AdviceKind::Diagnosis
    } else {
        AdviceKind::Observation
    };
    let hp_lost: f32 = guarded.iter().map(|(_, drop)| drop).sum();
    Some(AdviceCard {
        id: "cornered_di_guard".to_string(),
        kind,
        confidence: EventConfidence::High,
        title: match kind {
            AdviceKind::Diagnosis => "画面端でDIをガードして壁やられを繰り返している",
            _ => "画面端でDIをガードした場面",
        }
        .to_string(),
        severity: hp_lost + 0.03 * guarded.len() as f32,
        hp_lost: Some(hp_lost),
        description: if repeated {
            format!(
                "画面端を背負った状態で相手のDIをガードした場面を {} 回確認し、壁やられからの反撃で合計 {:.0}% 被弾しています。中央なら押し返されて終わる同じガードが、端では確定反撃の献上になります。ガードは間違いではないぶん、端でだけ回答を変える必要がある点が改善対象です。",
                guarded.len(),
                hp_lost * 100.0
            )
        } else {
            format!(
                "画面端を背負った状態で相手のDIをガードし、壁やられから {:.0}% 被弾した場面が1回あります。単発では、返しの用意が無かったのか、とっさに間に合わなかっただけなのかは{OBSERVATION_REVIEW_CAVEAT}。",
                hp_lost * 100.0
            )
        },
        practice: match kind {
            AdviceKind::Diagnosis => "トレモで相手レコードにDIを仕込み、端を背負った状態からDI返しを最優先で練習します。返しが間に合わない距離では前ジャンプや垂直ジャンプで壁やられだけ回避する選択も確認しましょう。",
            _ => "クリップで、DIを見てから返す猶予があったかを確認します。猶予があったならDI返しの反応練習を、無かったなら端でDIを撃たれる前の間合い管理を見直しましょう。",
        }
        .to_string(),
        evidence: guarded
            .iter()
            .map(|(impact, drop)| EvidenceClip {
                frame: impact.input_frame,
                end_frame: None,
                label: format!(
                    "R{} 端でDIをガード -{:.0}%",
                    impact.round_no,
                    drop * 100.0
                ),
            })
            .collect(),
    })
}
