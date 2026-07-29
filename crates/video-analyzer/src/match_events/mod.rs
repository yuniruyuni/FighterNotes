//! 試合イベント層（フェーズ 1）。
//!
//! **確定層の出力のみ**（temporal::confirm_hp 済みの own_hp/opponent_hp、
//! clean_drive_temporal 済みのドライブ、入力トラッカーの TrackedInput）から、
//! アドバイス生成が使う意味的イベントを組み立てる。viewer が表示する値と
//! 同一の値だけを使う（このモジュールでは知覚の再クリーニングをしない）:
//!   - ラウンド分割（両者 HP 全快の持続区間 = ラウンド開始、HP 0 = KO）
//!   - ダメージシーケンス（連続した HP 減少のまとまり = 1 コンボ/1 被弾）
//!   - 入力セグメント（同一入力の継続区間。ジャンプ・投げ・ボタン押下の元）
//!   - ジャンプイベントと結果（通した / 落とされた / 何もなし）
//!   - 投げイベントと成否
//!   - バーンアウト期間とその間の損失
//!
//! HP は own/opp ではなく P1/P2（画面左右）で扱う。own への振り分けは
//! アドバイス層（advice.rs）が行う。

use crate::frame_features::FrameFeatures;
use crate::input_history::InputDir;
use crate::input_tracker::TrackedInput;
use crate::round_start::FightMarker;
use meter_tracker::MeterTimeline;

// ── 出力型 ───────────────────────────────────────────────────────────────────

mod actions;
mod burnouts;
mod contacts;
mod damage;
mod guard_breaks;
mod jumps;
mod minus_press;
mod model;
mod parameters;
mod punishes;
mod reversals;
mod rounds;
mod segments;
mod threats;
mod timeline;

pub use model::*;
pub use threats::{
    CompoundThreat, DefenseResponse, DefenseResponseKind, DpReachability, ProjectileThreat,
    TeleportContext, TeleportEvent, ThreatOutcome,
};
pub use timeline::round_of;

pub(crate) use contacts::*;
pub(crate) use damage::{extend_rounds_through_freezes, extract_damage_sequences};
pub(crate) use guard_breaks::*;
#[allow(unused_imports)]
pub(crate) use minus_press::*;
pub(crate) use parameters::*;
pub(crate) use punishes::*;
pub(crate) use reversals::*;
pub(crate) use rounds::*;
pub(crate) use segments::*;
pub(crate) use threats::{extract_threats, THREAT_DAMAGE_WINDOW};
pub(crate) use timeline::{
    both_freeze_spans, confidence_per_frame, continuous_epoch, epoch_per_frame, gf_per_frame,
    idx_of, movement_run_is_ground_attack_chain, movementish_per_frame, state_per_frame,
};

/// イベント層を構築する。
///
/// `features` は確定層（temporal::confirm_hp / clean_drive_temporal）通過後で
/// あること。own_hp / opponent_hp をそのまま信用し、`own_side` で P1/P2 に
/// 写像して使う。`p1_inputs` / `p2_inputs` は確定層トラッカーの出力
/// （features と同じフレーム順・同数）。入力読み取りが無いパイプラインでは
/// 空でもよい（ジャンプ・投げ・セグメントが空になるだけ）。
/// `meter` はフレームメーターの確定タイムライン（P1, P2）。None なら
/// コンタクト検出と stun ゲートは HP ベースの近似にフォールバックする。
pub fn build_match_events(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&MeterTimeline, &MeterTimeline)>,
    own_side: &str,
) -> MatchEvents {
    let context = crate::context::AnalysisContext::new(own_side);
    build_match_events_with_context(features, p1_inputs, p2_inputs, meter, &context)
}

/// Character-aware event entry point.
///
/// Character-specific move signatures are only enabled when the corresponding
/// P1/P2 metadata is present. This keeps the legacy API conservative.
pub fn build_match_events_with_context(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&MeterTimeline, &MeterTimeline)>,
    context: &crate::context::AnalysisContext,
) -> MatchEvents {
    build_match_events_with_optional_fight_markers(
        features, p1_inputs, p2_inputs, meter, context, None,
    )
}

/// `FIGHT` 画像で確定した開始位置だけを使う browser pipeline 用 entry point。
pub fn build_match_events_with_context_and_fight_markers(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&MeterTimeline, &MeterTimeline)>,
    context: &crate::context::AnalysisContext,
    markers: &[FightMarker],
) -> MatchEvents {
    build_match_events_with_optional_fight_markers(
        features,
        p1_inputs,
        p2_inputs,
        meter,
        context,
        Some(markers),
    )
}

fn build_match_events_with_optional_fight_markers(
    features: &[FrameFeatures],
    p1_inputs: &[TrackedInput],
    p2_inputs: &[TrackedInput],
    meter: Option<(&MeterTimeline, &MeterTimeline)>,
    context: &crate::context::AnalysisContext,
    fight_markers: Option<&[FightMarker]>,
) -> MatchEvents {
    let own_side = context.own_side();
    let characters = [
        context.p1.character.as_deref(),
        context.p2.character.as_deref(),
    ];
    let n = features.len();
    let (p1_stun, p2_stun) = match meter {
        Some((l, r)) => (l.stun_per_frame(n as u32), r.stun_per_frame(n as u32)),
        None => (Vec::new(), Vec::new()),
    };
    let (p1_stun, p2_stun) = (&p1_stun[..], &p2_stun[..]);

    // ── 確定済み HP を P1/P2 系列へ写像 ─────────────────────────────────
    // 値のクリーニングは確定層で完了している。ここでは先頭の不明（-1）を
    // 全快扱いにするだけ。ラウンド開始時の 1px cap ノイズは知覚層で補正し、
    // 両者満タンの持続帯を uncertain に落とさない。
    let own_is_p1 = own_side != "p2";
    let mut hp: [Vec<f32>; 2] = [Vec::with_capacity(n), Vec::with_capacity(n)];
    for f in features {
        let (l, r) = if own_is_p1 {
            (f.own_hp, f.opponent_hp)
        } else {
            (f.opponent_hp, f.own_hp)
        };
        hp[0].push(if l < 0.0 { 1.0 } else { l });
        hp[1].push(if r < 0.0 { 1.0 } else { r });
    }

    // SA 暗転・投げ・KO 演出の停止区間は、ダメージ集約とラウンド終端の
    // 両方で使うため先に確定する。
    let freeze_spans = match meter {
        Some((l, r)) => both_freeze_spans(l, r),
        None => Vec::new(),
    };

    // ── ラウンド分割 ─────────────────────────────────────────────────────
    let mut rounds = match fight_markers {
        Some(markers) => detect_rounds_from_fight_markers(features, &hp, markers),
        None => detect_rounds_from_hp(features, &hp),
    };
    extend_rounds_through_freezes(&mut rounds, features, &hp, &freeze_spans);

    // ── ラウンド内単調化（HP は増えない。灰ゲージ回復は無視） ──────────
    let mut mono = hp.clone();
    for r in &rounds {
        let (a, b) = (
            idx_of(features, r.start_frame),
            idx_of(features, r.end_frame),
        );
        for side in &mut mono {
            let mut m = f32::MAX;
            for value in side.iter_mut().take(b.min(n.saturating_sub(1)) + 1).skip(a) {
                m = m.min(*value);
                *value = m;
            }
        }
    }

    // ── ダメージシーケンス ───────────────────────────────────────────────
    // レポート用は SA 停止をゲーム時間から除いて一連の被弾へまとめる。
    // コンタクトの hit/block 判定だけは従来の動画時間区切りを保持し、結合に
    // よって SA 後半の実ヒットがガードへ変わらないようにする。
    let mut damage =
        extract_damage_sequences(features, &mono, &rounds, &freeze_spans, [p1_stun, p2_stun]);
    let mut contact_damage = extract_damage_sequences(features, &mono, &rounds, &[], [&[], &[]]);

    // ── ラウンド妥当性フィルタ ───────────────────────────────────────────
    // 実ラウンドには必ず被弾が発生する。ダメージイベントが 1 件も無い
    // 「ラウンド」はリプレイ冒頭のイントロ画面等の誤検出なので捨てる
    let valid: Vec<u32> = rounds
        .iter()
        .filter(|r| damage.iter().any(|d| d.round_no == r.round_no))
        .map(|r| r.round_no)
        .collect();
    let mut rounds: Vec<RoundInfo> = rounds
        .into_iter()
        .filter(|r| valid.contains(&r.round_no))
        .collect();
    damage.retain(|d| valid.contains(&d.round_no));
    contact_damage.retain(|d| valid.contains(&d.round_no));
    // 番号を振り直す（旧→新の対応でダメージ側も更新）
    let renum: std::collections::HashMap<u32, u32> = rounds
        .iter()
        .enumerate()
        .map(|(k, r)| (r.round_no, k as u32 + 1))
        .collect();
    for r in rounds.iter_mut() {
        r.round_no = renum[&r.round_no];
    }
    for d in damage.iter_mut() {
        d.round_no = renum[&d.round_no];
    }
    for d in contact_damage.iter_mut() {
        d.round_no = renum[&d.round_no];
    }

    // ── コンタクトイベント（メーター由来） ───────────────────────────────
    let contacts = match meter {
        Some((l, r)) => extract_contacts(l, r, &contact_damage, &rounds),
        None => Vec::new(),
    };
    let meter_state = match meter {
        Some((l, r)) => [state_per_frame(l, n), state_per_frame(r, n)],
        None => [Vec::new(), Vec::new()],
    };
    let meter_confidence = match meter {
        Some((l, r)) => [confidence_per_frame(l, n), confidence_per_frame(r, n)],
        None => [Vec::new(), Vec::new()],
    };
    let meter_epoch = match meter {
        Some((l, r)) => [epoch_per_frame(l, n), epoch_per_frame(r, n)],
        None => [Vec::new(), Vec::new()],
    };

    // ── 被弾アンカーの精密化 ─────────────────────────────────────────────
    // HP 減少の検出開始は実ヒットより数フレーム遅れる（ヒットストップ中の
    // バー演出）。対応するヒットコンタクトがあればそこへ寄せる
    for d in damage.iter_mut() {
        if let Some(c) = contacts
            .iter()
            .filter(|c| {
                c.victim == d.victim
                    && c.hit
                    && c.frame <= d.start_frame + 3
                    && c.frame + 20 >= d.start_frame
            })
            .min_by_key(|c| c.frame.abs_diff(d.start_frame))
        {
            d.start_frame = c.frame;
            d.end_frame = d.end_frame.max(d.start_frame);
        }
    }

    // ── 演出フリーズ前アンカー ───────────────────────────────────────────
    // SA 暗転等の長い演出の直後に被弾が来る場合、クリップの固定前方
    // マージン（1.5s）が演出に食われて被弾直前の行動が映らない。
    // 被弾に接するフリーズスパンの開始を pre_freeze_frame として記録する
    // （フリーズ無しは start_frame のまま。ラウンド開始より前には遡らない）
    for d in damage.iter_mut() {
        d.pre_freeze_frame = d.start_frame;
        let Some(&(fa, fb)) = freeze_spans
            .iter()
            .rfind(|&&(fa, fb)| fa <= d.start_frame && d.start_frame <= fb + FREEZE_ATTACH_GAP)
        else {
            continue;
        };
        let _ = fb;
        let round_start = rounds
            .iter()
            .find(|r| r.round_no == d.round_no)
            .map_or(0, |r| r.start_frame);
        d.pre_freeze_frame = fa.max(round_start).min(d.start_frame);
    }

    // ── 入力セグメント ───────────────────────────────────────────────────
    let segments = [
        build_segments(features, p1_inputs),
        build_segments(features, p2_inputs),
    ];

    // ── 確定反撃の機会と結果 ─────────────────────────────────────────────
    let meter_gf: [Vec<i64>; 2] = match meter {
        Some((l, r)) => [gf_per_frame(l, n), gf_per_frame(r, n)],
        None => [Vec::new(), Vec::new()],
    };
    let throw_actions = actions::extract_throw_actions(
        &meter_state,
        &meter_epoch,
        &contacts,
        &damage,
        &segments,
        &rounds,
    );
    // 旧APIの ThrowEvent は意味イベントから導出する。入力が見えただけの
    // Unconfirmed を落とし、演出で遅れてHPが減る投げも成立として残す。
    let throws: Vec<ThrowEvent> = if meter_state[0].is_empty() {
        let mut legacy = Vec::new();
        for (side_index, inputs) in segments.iter().enumerate() {
            for input in inputs.iter().filter(|input| input.throw) {
                let Some(round_no) = round_of(&rounds, input.start_frame) else {
                    continue;
                };
                legacy.push(ThrowEvent {
                    thrower: side_index as u8 + 1,
                    frame: input.start_frame,
                    connected: damage.iter().any(|event| {
                        event.victim == 2 - side_index as u8
                            && event.start_frame >= input.start_frame
                            && event.start_frame <= input.start_frame + 125
                            && event.drop >= THROW_MIN_DROP
                    }),
                    round_no,
                });
            }
        }
        legacy
    } else {
        throw_actions
            .iter()
            .filter(|event| event.confidence == EventConfidence::High)
            .map(|event| ThrowEvent {
                thrower: event.thrower,
                frame: event.input_frame,
                connected: event.outcome == ThrowOutcome::Hit,
                round_no: event.round_no,
            })
            .collect()
    };
    let drive_impacts = actions::extract_drive_impacts(
        &meter_state,
        &meter_epoch,
        &contacts,
        &damage,
        &segments,
        &rounds,
    );
    let drive_rushes = actions::extract_drive_rushes(
        features,
        &meter_state,
        &meter_epoch,
        &contacts,
        &damage,
        &segments,
        &rounds,
    );
    let punishes = extract_punishes(PunishInputs {
        features,
        meter_state: &meter_state,
        meter_epoch: &meter_epoch,
        meter_game_frame: &meter_gf,
        contacts: &contacts,
        damage: &damage,
        segments: &segments,
        rounds: &rounds,
    });

    let jumps = jumps::extract_jumps(jumps::JumpInputs {
        features,
        segments: &segments,
        p1_stun,
        p2_stun,
        meter,
        meter_game_frame: &meter_gf,
        meter_state: &meter_state,
        meter_epoch: &meter_epoch,
        damage: &damage,
        contacts: &contacts,
        rounds: &rounds,
        characters,
    });

    let burnouts = burnouts::extract_burnouts(burnouts::BurnoutInputs {
        features,
        rounds: &rounds,
        hp: &mono,
        contacts: &contacts,
        drive_impacts: &drive_impacts,
        drive_rushes: &drive_rushes,
        meter_state: &meter_state,
    });

    // ── 永続する飛び道具・テレポート・複合脅威 ───────────────────────────
    let (projectiles, teleports, compound_threats) = match meter {
        Some((left, right)) => extract_threats(threats::ThreatInputs {
            features,
            timelines: [left, right],
            meter_state: &meter_state,
            segments: &segments,
            jumps: &jumps,
            contacts: &contacts,
            damage: &damage,
            rounds: &rounds,
            characters,
        }),
        None => (Vec::new(), Vec::new(), Vec::new()),
    };

    // ── 無敵技ぶっぱ被弾 ─────────────────────────────────────────────────
    let reversals = extract_reversals(ReversalInputs {
        features,
        meter_state: &meter_state,
        meter_epoch: &meter_epoch,
        contacts: &contacts,
        damage: &damage,
        segments: &segments,
        rounds: &rounds,
        teleports: &teleports,
    });

    // ── ガード入力崩れ ───────────────────────────────────────────────────
    let guard_breaks = extract_guard_breaks(
        &damage,
        &meter_state,
        &mono,
        [p1_inputs, p2_inputs],
        &jumps,
        &throws,
        &reversals,
        &rounds,
    );

    // ── 不利フレーム中のボタン暴れ ───────────────────────────────────────
    let minus_events = minus_press::extract_minus_events(
        &meter_state,
        &meter_epoch,
        &meter_gf,
        &contacts,
        &damage,
        &segments,
        &rounds,
    );
    let presses_while_minus = minus_events.presses;
    let minus_situations = minus_events.situations;

    MatchEvents {
        rounds,
        damage,
        jumps,
        throws,
        throw_actions,
        drive_impacts,
        drive_rushes,
        burnouts,
        contacts,
        punishes,
        reversals,
        guard_breaks,
        presses_while_minus,
        minus_situations,
        projectiles,
        teleports,
        compound_threats,
        meter_state,
        meter_confidence,
        meter_game_frame: meter_gf,
        segments,
        hp: mono,
    }
}

#[cfg(test)]
mod tests;
