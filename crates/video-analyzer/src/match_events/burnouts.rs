use super::*;

pub(crate) struct BurnoutInputs<'a> {
    pub(crate) features: &'a [FrameFeatures],
    pub(crate) rounds: &'a [RoundInfo],
    pub(crate) hp: &'a [Vec<f32>; 2],
    pub(crate) contacts: &'a [ContactEvent],
    pub(crate) drive_impacts: &'a [DriveImpactEvent],
    pub(crate) drive_rushes: &'a [DriveRushEvent],
    pub(crate) meter_state: &'a [Vec<MeterState>; 2],
}

pub(crate) fn extract_burnouts(inputs: BurnoutInputs<'_>) -> Vec<BurnoutPeriod> {
    let BurnoutInputs {
        features,
        rounds,
        hp,
        contacts,
        drive_impacts,
        drive_rushes,
        meter_state,
    } = inputs;
    let n = features.len();

    let mut burnouts: Vec<BurnoutPeriod> = Vec::new();
    for s in 0..2usize {
        let bo = |f: &FrameFeatures| {
            if s == 0 {
                f.left_burnout
            } else {
                f.right_burnout
            }
        };
        let mut i = 0usize;
        while i < n {
            if bo(&features[i]) && features[i].is_match_screen {
                let start = i;
                let mut last = i;
                let mut j = i + 1;
                while j < n && j - last <= BO_GAP {
                    if bo(&features[j]) && features[j].is_match_screen {
                        last = j;
                    }
                    j += 1;
                }
                if let Some(round_no) = round_of(rounds, features[start].frame_index) {
                    // 期間をラウンド内にクリップ（次ラウンドの全快と比較しない）
                    let r = rounds.iter().find(|r| r.round_no == round_no).unwrap();
                    let end_i = last.min(idx_of(features, r.end_frame));
                    let start_frame = features[start].frame_index;
                    let cause_start = start_frame.saturating_sub(120);
                    let forced_by_guard = contacts.iter().any(|contact| {
                        contact.victim == s as u8 + 1
                            && !contact.hit
                            && contact.frame >= cause_start
                            && contact.frame <= start_frame
                    });
                    let self_initiated = drive_impacts.iter().any(|impact| {
                        impact.side == s as u8 + 1
                            && impact.confidence == EventConfidence::High
                            && impact.input_frame >= cause_start
                            && impact.input_frame <= start_frame
                    }) || drive_rushes.iter().any(|rush| {
                        rush.side == s as u8 + 1
                            && rush.frame >= cause_start
                            && rush.frame <= start_frame
                    }) || (idx_of(features, cause_start)..=start)
                        .any(|frame| meter_state[s].get(frame) == Some(&MeterState::Parry));
                    let cause = match (self_initiated, forced_by_guard) {
                        (true, true) => BurnoutCause::Mixed,
                        (true, false) => BurnoutCause::SelfInitiated,
                        (false, true) => BurnoutCause::ForcedByGuard,
                        (false, false) => BurnoutCause::Unknown,
                    };
                    burnouts.push(BurnoutPeriod {
                        side: s as u8 + 1,
                        start_frame,
                        end_frame: features[end_i].frame_index,
                        hp_lost: (hp[s][start] - hp[s][end_i]).max(0.0),
                        hp_dealt: (hp[1 - s][start] - hp[1 - s][end_i]).max(0.0),
                        cause,
                        confidence: if cause == BurnoutCause::Unknown {
                            EventConfidence::Medium
                        } else {
                            EventConfidence::High
                        },
                        round_no,
                    });
                }
                i = last + 1;
            } else {
                i += 1;
            }
        }
    }

    // 短すぎるバーンアウト期間は偽物（知覚の誤読が時間フィルタをすり抜けた
    // ケース）。ラウンド終端まで続いた期間だけは「突入直後 KO」なので残す
    burnouts.retain(|b| {
        if b.end_frame - b.start_frame >= BO_MIN_FRAMES {
            return true;
        }
        rounds
            .iter()
            .find(|r| r.round_no == b.round_no)
            .is_some_and(|r| b.end_frame + BO_ROUND_END_MARGIN >= r.end_frame)
    });
    burnouts
}
