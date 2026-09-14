//! ガードされた相手の技の同定。
//!
//! 相手の入力表示も画面に写っているため、接触直前の入力バッジと
//! フレームメーターの実測発生を frame_data の記譜と突き合わせれば、
//! どの技をガードしたかを外部データなしで同定できる。誤同定は
//! 「その技じゃない」という強い信頼毀損になるため、一意に絞れた
//! ときだけ技名を返し、絞れない場面は None のままにする。

use crate::frame_data::{self, MoveData};
use crate::match_events::{round_of, MatchEvents, MeterState, PunishOutcome, PunishReachability};
use crate::model::OpponentMoveStat;
use crate::MASH_METER_CONFIDENCE;

/// ヒットの HP 損失を接触へ帰属する窓。既存の被弾帰属(±25F)と同じ。
const HIT_DAMAGE_WINDOW: u32 = 25;

/// キャラクター id(例: CHUN_LI)をカード文言用の表記(CHUN-LI)へ直す。
/// client の formatCharacterId と同じ規則で、履歴画面の表記と揃える。
pub fn display_character(id: &str) -> String {
    id.replace('_', "-")
}

/// カード題名へ相手キャラ名を差し込む接尾辞。未指定なら何も足さない。
pub fn opponent_suffix(opponent_character: Option<&str>) -> String {
    opponent_character
        .map(|id| format!("(相手: {})", display_character(id)))
        .unwrap_or_default()
}

/// 同定できた相手の技ごとの、触られ方と回答の収支。
///
/// 同定は入力表示と実測発生の二重整合が取れた接触に限るため、どの値も
/// 下限になる。触られた回数の多い順に返す。
pub fn build_opponent_move_stats(
    events: &MatchEvents,
    own: u8,
    opponent_character: Option<&str>,
) -> Vec<OpponentMoveStat> {
    let opponent = 3 - own;
    let mut stats: Vec<OpponentMoveStat> = Vec::new();
    for contact in events.contacts.iter().filter(|contact| {
        contact.attacker == opponent
            && contact.victim == own
            && round_of(&events.rounds, contact.frame) == Some(contact.round_no)
    }) {
        let Some(move_data) =
            identify_contact_move(events, opponent, opponent_character, contact.frame)
        else {
            continue;
        };
        let entry = match stats.iter_mut().find(|entry| entry.name == move_data.name) {
            Some(entry) => entry,
            None => {
                stats.push(OpponentMoveStat {
                    name: move_data.name.clone(),
                    projectile: false,
                    touches: 0,
                    hits_taken: 0,
                    hp_lost: 0.0,
                    blocked: 0,
                    punished: 0,
                    punish_missed: 0,
                });
                stats.last_mut().expect("直前に push した")
            }
        };
        entry.touches += 1;
        entry.projectile |= contact.projectile;
        if contact.hit {
            entry.hits_taken += 1;
            entry.hp_lost += events
                .damage
                .iter()
                .filter(|damage| {
                    damage.victim == own
                        && damage.round_no == contact.round_no
                        && damage.start_frame.abs_diff(contact.frame) <= HIT_DAMAGE_WINDOW
                })
                .min_by_key(|damage| damage.start_frame.abs_diff(contact.frame))
                .map_or(0.0, |damage| damage.drop);
        } else {
            entry.blocked += 1;
            // このガードを起点にした確反機会の結末。
            if let Some(punish) = events.punishes.iter().find(|punish| {
                punish.side == own && punish.source_contact_frame == Some(contact.frame)
            }) {
                match punish.outcome {
                    PunishOutcome::Success => entry.punished += 1,
                    PunishOutcome::Missed
                        if punish.reachability == PunishReachability::Confirmed =>
                    {
                        entry.punish_missed += 1
                    }
                    _ => {}
                }
            }
        }
    }
    stats.sort_by(|a, b| b.touches.cmp(&a.touches).then(a.name.cmp(&b.name)));
    stats
}

/// 実測発生を数えるとき、接触から Startup まで遡って良い距離。持続
/// (Active)は長い技でもこの範囲に収まり、これを超えて遡ると別の行動の
/// 表示を読んでしまう。
const ACTIVE_SKIP_MAX: usize = 16;
/// 押下推定フレームと入力セグメント開始のずれの許容。表示遅延と
/// セグメント化の丸めを吸収する。
const PRESS_SLACK: u32 = 4;

/// 接触へ至った相手の技を同定する。
pub(crate) fn identify_contact_move(
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
    let contact = contact_frame as usize;
    // 接触から ACTIVE_SKIP_MAX まで遡り、最初に見つかる Startup が
    // この技の発生の終端。
    let run_end = (contact.saturating_sub(ACTIVE_SKIP_MAX)..=contact)
        .rev()
        .find(|&index| states.get(index) == Some(&MeterState::Startup))?;
    let mut length = 0u32;
    for index in (0..=run_end).rev() {
        if states.get(index) != Some(&MeterState::Startup) {
            break;
        }
        if !reliable(index) {
            return None;
        }
        length += 1;
    }
    Some(length)
}
