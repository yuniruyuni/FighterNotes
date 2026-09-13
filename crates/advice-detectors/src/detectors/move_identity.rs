//! ガードされた相手の技の同定。
//!
//! 相手の入力表示も画面に写っているため、接触直前の入力バッジと
//! フレームメーターの実測発生を frame_data の記譜と突き合わせれば、
//! どの技をガードしたかを外部データなしで同定できる。誤同定は
//! 「その技じゃない」という強い信頼毀損になるため、一意に絞れた
//! ときだけ技名を返し、絞れない場面は None のままにする。

use crate::frame_data::{self, MoveData};
use crate::match_events::{MatchEvents, MeterState};
use crate::MASH_METER_CONFIDENCE;

/// 実測発生を数えるとき、接触から Startup まで遡って良い距離。持続
/// (Active)は長い技でもこの範囲に収まり、これを超えて遡ると別の行動の
/// 表示を読んでしまう。
const ACTIVE_SKIP_MAX: usize = 16;
/// 押下推定フレームと入力セグメント開始のずれの許容。表示遅延と
/// セグメント化の丸めを吸収する。
const PRESS_SLACK: u32 = 4;

/// 接触へ至った相手の技を同定する。
pub(crate) fn identify_blocked_move(
    events: &MatchEvents,
    attacker: u8,
    attacker_character: Option<&str>,
    contact_frame: u32,
) -> Option<&'static MoveData> {
    let character = attacker_character?;
    let attacker_index = attacker as usize - 1;
    let states = events.meter_state.get(attacker_index)?;
    let confidence = events.meter_confidence.get(attacker_index)?;
    let startup = measured_startup(states, confidence, contact_frame)?;
    // 押下はおよそ 接触 - 発生。その近傍で直接観測できた攻撃入力を探す。
    let press = contact_frame.saturating_sub(startup);
    let segment = events
        .segments
        .get(attacker_index)?
        .iter()
        .filter(|segment| {
            segment.evidence.has_direct_observation()
                && !segment.badges.is_empty()
                && segment.start_frame <= press.saturating_add(PRESS_SLACK)
                && segment.end_frame.saturating_add(PRESS_SLACK) >= press
        })
        .max_by_key(|segment| segment.start_frame)?;
    frame_data::identify_move(
        character,
        &segment.dir,
        &segment.badges,
        segment.auto,
        startup,
    )
}

/// 接触フレームから遡って、直前の Startup 表示の連続長(実測発生)を
/// 数える。途中の持続・接触表示は跨ぐが、読み取り信頼度の足りない
/// フレームがあれば実測として使わない。
fn measured_startup(states: &[MeterState], confidence: &[f32], contact_frame: u32) -> Option<u32> {
    let reliable = |index: usize| {
        confidence.is_empty()
            || confidence
                .get(index)
                .is_some_and(|value| *value >= MASH_METER_CONFIDENCE)
    };
    let mut index = contact_frame as usize;
    let mut skipped = 0usize;
    while states.get(index) != Some(&MeterState::Startup) {
        if index == 0 || skipped >= ACTIVE_SKIP_MAX {
            return None;
        }
        index -= 1;
        skipped += 1;
    }
    let mut length = 0u32;
    while states.get(index) == Some(&MeterState::Startup) {
        if !reliable(index) {
            return None;
        }
        length += 1;
        if index == 0 {
            break;
        }
        index -= 1;
    }
    Some(length)
}
