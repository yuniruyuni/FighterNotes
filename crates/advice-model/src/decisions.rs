//! 読み合いが起きる状況を、種類・選んだ回答・結末という同じ形へ揃える。
//!
//! 「不利フレーム後」「有利フレームを取った後」「ダウンからの起き上がり」は
//! どれも同じ構造をしている。状況が発生し、回答を選び、結果が出る。
//! 偏りの判定条件（機会数・同一回答数・損失数・選択率）も共通で、
//! 状況ごとに書き分ける理由がない。
//!
//! ここは**射影であって再導出ではない**。各状況の検出は既存のイベント層が
//! 担当し、この層はそれを同じ形へ並べ替えるだけにする。判定の根拠を1か所へ
//! 集めながら、既存の検出結果を動かさないための境界である。

use crate::match_events::{
    AdvantageOutcome, DefensiveActionKind, EventConfidence, MatchEvents, MinusPressOutcome,
    OkizemeOutcome,
};

/// 読み合いが発生する状況の種類。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionSituation {
    /// ガード後に不利フレームを背負った。
    Disadvantage,
    /// ガードさせて有利フレームを取った。
    Advantage,
    /// ダウンを取り、相手の起き上がりに向き合っている。
    Okizeme,
}

/// その状況で選んだ回答。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionOption {
    Strike,
    Throw,
    /// 攻撃を始めなかった。ガード継続・移動・様子見をまとめたもので、
    /// どれであったかまでは入力表示から断定しない。
    NoAttack,
}

/// 選んだ回答の結末。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionResult {
    /// その回答が負けた（被弾、またはターンを渡した）。
    Lost,
    /// 損失なく済んだ。
    Survived,
}

pub struct DecisionEvent {
    pub situation: DecisionSituation,
    pub frame: u32,
    /// 状況の大きさ（不利／有利フレーム数）。起き攻めは 0。
    pub frames: u32,
    pub option: DecisionOption,
    pub pressed: String,
    pub result: DecisionResult,
    pub drop: f32,
    pub round_no: u32,
}

/// 確度の高い判断機会だけを同じ形へ並べる。
pub fn collect_decisions(events: &MatchEvents, own: u8) -> Vec<DecisionEvent> {
    let mut out = Vec::new();

    // 不利フレーム後。最速打撃／最速投げを選んだ機会は presses_while_minus が
    // 結果まで持ち、それ以外の回答は minus_situations にだけ残る。
    for press in events
        .presses_while_minus
        .iter()
        .filter(|event| event.side == own && event.confidence == EventConfidence::High)
    {
        out.push(DecisionEvent {
            situation: DecisionSituation::Disadvantage,
            frame: press.frame,
            frames: press.minus_frames,
            option: match press.action_kind {
                DefensiveActionKind::Strike => DecisionOption::Strike,
                DefensiveActionKind::Throw => DecisionOption::Throw,
            },
            pressed: press.pressed.clone(),
            result: match press.outcome {
                MinusPressOutcome::CounterHit => DecisionResult::Lost,
                MinusPressOutcome::Won | MinusPressOutcome::GotAway => DecisionResult::Survived,
            },
            drop: press.drop,
            round_no: press.round_no,
        });
    }
    for situation in events
        .minus_situations
        .iter()
        .filter(|event| event.side == own && event.confidence == EventConfidence::High)
        .filter(|event| event.fastest_action.is_none())
    {
        out.push(DecisionEvent {
            situation: DecisionSituation::Disadvantage,
            frame: situation.frame,
            frames: situation.minus_frames,
            option: DecisionOption::NoAttack,
            pressed: String::new(),
            result: DecisionResult::Survived,
            drop: 0.0,
            round_no: situation.round_no,
        });
    }

    // 有利フレームを取った後。
    for advantage in events
        .advantage_situations
        .iter()
        .filter(|event| event.side == own && event.confidence == EventConfidence::High)
    {
        out.push(DecisionEvent {
            situation: DecisionSituation::Advantage,
            frame: advantage.frame,
            frames: advantage.plus_frames,
            option: if advantage.action_frame.is_some() {
                DecisionOption::Strike
            } else {
                DecisionOption::NoAttack
            },
            pressed: advantage.pressed.clone(),
            result: match advantage.outcome {
                AdvantageOutcome::TurnLost => DecisionResult::Lost,
                AdvantageOutcome::Continued | AdvantageOutcome::Reset => DecisionResult::Survived,
            },
            drop: advantage.drop,
            round_no: advantage.round_no,
        });
    }

    // ダウンを取った側の起き攻め。攻めなかったことによる直接の損失は
    // 観測できないため、結末は常に Survived にする。
    for down in events
        .knockdowns
        .iter()
        .filter(|event| event.attacker == own && event.confidence == EventConfidence::High)
    {
        out.push(DecisionEvent {
            situation: DecisionSituation::Okizeme,
            frame: down.wakeup_frame,
            frames: 0,
            option: match down.okizeme {
                OkizemeOutcome::Meaty | OkizemeOutcome::Pressured => DecisionOption::Strike,
                OkizemeOutcome::Neutral => DecisionOption::NoAttack,
            },
            pressed: String::new(),
            result: DecisionResult::Survived,
            drop: 0.0,
            round_no: down.round_no,
        });
    }

    out.sort_by_key(|event| event.frame);
    out
}

/// ある状況で、ある回答を選んだ機会。
pub fn selections(
    decisions: &[DecisionEvent],
    situation: DecisionSituation,
    option: DecisionOption,
) -> Vec<&DecisionEvent> {
    decisions
        .iter()
        .filter(|event| event.situation == situation && event.option == option)
        .collect()
}

/// ある状況の判断機会の総数。回答の偏りを測る分母になる。
pub fn opportunities(decisions: &[DecisionEvent], situation: DecisionSituation) -> usize {
    decisions
        .iter()
        .filter(|event| event.situation == situation)
        .count()
}

pub fn losses<'a>(selected: &'a [&DecisionEvent]) -> Vec<&'a DecisionEvent> {
    selected
        .iter()
        .copied()
        .filter(|event| event.result == DecisionResult::Lost)
        .collect()
}

/// その状況で最も多かった回答が占める割合（百分率）と、その回答。
///
/// 同率のときは選択肢の定義順で先に来るものを返す。回答が読まれているか
/// を測る指標なので、機会が少ないうちは意味を持たない。判断は呼び出し側で
/// 行えるよう、ここでは機会数も一緒に返す。
pub fn option_bias(
    decisions: &[DecisionEvent],
    situation: DecisionSituation,
) -> Option<(DecisionOption, usize, usize)> {
    let total = opportunities(decisions, situation);
    if total == 0 {
        return None;
    }
    [
        DecisionOption::Strike,
        DecisionOption::Throw,
        DecisionOption::NoAttack,
    ]
    .into_iter()
    .map(|option| (option, selections(decisions, situation, option).len()))
    .max_by_key(|(_, count)| *count)
    .map(|(option, count)| (option, count, total))
}
