//! 観測列を一本の試合として組み立てるところに対するテスト。
//!
//! ここまでの層は、それぞれ一つの現象しか見ていない。試合として意味を
//! 持たせるのはこの層で、HP・フレームメーター・入力履歴という別々の
//! 読み取りを同じ時間軸に並べ、ラウンドで区切り、それぞれの抽出器へ
//! 渡す。渡し忘れれば、その種類の指摘だけが静かに消える。
//!
//! 合成試合を一本通し、出てくるべきものが出てくることを確かめる。

use super::support::*;
use crate::attack_info::{AttackAttribute, AttackInfoObservation, AttackInfoSide};
use crate::input_history::{BadgeColor, BadgeMark};
use crate::round_start::FightMarker;

/// 片側のフレームメーターを、時間を進めながら組み立てる。
///
/// ゲーム内時間と動画時間は同じ速さでは進まない。ヒットストップや
/// 演出の間は動画だけが進む。この違いが有利フレームの計算を左右する
/// ので、二つの時計を別々に持つ。
struct MeterBuilder {
    entries: Vec<(i64, String, i64, i64)>,
    game_frame: i64,
    video_frame: i64,
}

impl MeterBuilder {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            game_frame: 0,
            video_frame: 0,
        }
    }

    /// 普通に流れている区間。1 ゲームフレームが 1 動画フレーム。
    fn run(&mut self, state: &str, frames: i64) -> &mut Self {
        for _ in 0..frames {
            self.entries.push((
                self.game_frame,
                state.to_string(),
                self.video_frame,
                self.video_frame,
            ));
            self.game_frame += 1;
            self.video_frame += 1;
        }
        self
    }

    /// 止まっている区間。1 ゲームフレームが複数の動画フレームに伸びる。
    fn hold(&mut self, state: &str, frames: i64) -> &mut Self {
        self.entries.push((
            self.game_frame,
            state.to_string(),
            self.video_frame,
            self.video_frame + frames - 1,
        ));
        self.game_frame += 1;
        self.video_frame += frames;
        self
    }

    /// 動画時間を指定の位置まで無表示で進める。
    fn until(&mut self, video_frame: i64) -> &mut Self {
        self.run("empty", video_frame - self.video_frame)
    }

    fn build(&self) -> MeterTimeline {
        synth_timeline(
            self.entries
                .iter()
                .map(|(gf, st, a, b)| (*gf, st.as_str(), *a, *b))
                .collect(),
        )
    }
}

fn circle(color: BadgeColor) -> BadgeMark {
    BadgeMark {
        color,
        boxed: false,
        glyph: None,
    }
}

fn boxed(color: BadgeColor) -> BadgeMark {
    BadgeMark {
        color,
        boxed: true,
        glyph: None,
    }
}

/// 合成試合。HP・メーター・入力を同じ長さで組み立てる。
struct Match {
    features: Vec<FrameFeatures>,
    left: MeterBuilder,
    right: MeterBuilder,
    inputs: [Vec<TrackedInput>; 2],
}

const FRAMES: usize = 1_800;

impl Match {
    fn new() -> Self {
        let idle = || {
            (0..FRAMES)
                .map(|k| tracked(k as u32 % 60 + 1, InputDir::Neutral, vec![], false, false))
                .collect()
        };
        let features = (0..FRAMES as u32)
            .map(|i| {
                let mut feature = feat(i, 1.0, 1.0);
                // SA ゲージは読めているものとする。読めていない扱いだと
                // SA の消費が見えない。
                feature.left_super_uncertain = false;
                feature.right_super_uncertain = false;
                feature.left_super_value = 3.0;
                feature.right_super_value = 1.0;
                feature
            })
            .collect();
        Self {
            features,
            left: MeterBuilder::new(),
            right: MeterBuilder::new(),
            inputs: [idle(), idle()],
        }
    }

    /// 指定のフレーム以降、片側の SA ゲージを指定の本数へ落とす。
    fn spend_super(&mut self, side: usize, from: usize, stocks: f32) -> &mut Self {
        for feature in &mut self.features[from..] {
            if side == 0 {
                feature.left_super_value = stocks;
            } else {
                feature.right_super_value = stocks;
            }
        }
        self
    }

    /// 指定のフレーム以降、片側のドライブゲージを指定の割合まで落とす。
    fn spend_drive(&mut self, side: usize, from: usize, ratio: f32) -> &mut Self {
        for feature in &mut self.features[from..] {
            if side == 0 {
                feature.left_drive_ratio = ratio;
            } else {
                feature.right_drive_ratio = ratio;
            }
        }
        self
    }

    /// 指定の区間で片側をバーンアウトさせる。
    fn burnout(&mut self, side: usize, from: usize, to: usize) -> &mut Self {
        for feature in &mut self.features[from..=to] {
            if side == 0 {
                feature.left_burnout = true;
                feature.left_drive_ratio = 0.0;
            } else {
                feature.right_burnout = true;
                feature.right_drive_ratio = 0.0;
            }
        }
        self
    }

    /// 指定の区間で片側の HP を落とす。以降のフレームはその値のまま。
    fn drain(&mut self, victim: usize, from: usize, to: usize, drop: f32) -> &mut Self {
        let before = if victim == 0 {
            self.features[from].own_hp
        } else {
            self.features[from].opponent_hp
        };
        for frame in from..FRAMES {
            let progress = ((frame - from) as f32 / (to - from) as f32).min(1.0);
            let value = (before - drop * progress).max(0.0);
            let feature = &mut self.features[frame];
            if victim == 0 {
                feature.own_hp = value;
                feature.left_hp_raw = value;
            } else {
                feature.opponent_hp = value;
                feature.right_hp_raw = value;
            }
        }
        self
    }

    /// 指定の区間にボタン入力を置く。
    fn press(
        &mut self,
        side: usize,
        from: usize,
        to: usize,
        dir: InputDir,
        badges: Vec<BadgeMark>,
        throw: bool,
    ) -> &mut Self {
        for (offset, frame) in (from..=to).enumerate() {
            self.inputs[side][frame] =
                tracked(offset as u32 + 1, dir, badges.clone(), false, throw);
        }
        self
    }

    fn build(&self) -> MatchEvents {
        let left = self.left.build();
        let right = self.right.build();
        build_match_events(
            &self.features,
            &self.inputs[0],
            &self.inputs[1],
            Some((&left, &right)),
            "p1",
        )
    }
}

/// 一本の合成試合。P1 が押し切って KO する。
fn synth_match() -> Match {
    let mut m = Match::new();

    // ── 開幕 ────────────────────────────────────────────────────────────
    m.left.until(200);
    m.right.until(200);

    // f200: P1 の打撃がヒット。P2 が倒れる（ダウン）。
    m.left.hold("active", 10).run("punish_counter", 20);
    m.right.hold("stun", 10).run("stun", 90);
    m.drain(1, 205, 225, 0.15);

    // f320: P2 の起き上がりに P1 が重ねてガードされる。P1 の後隙が短く、
    // P2 のガード硬直が長いので P2 が不利。
    m.left
        .until(320)
        .hold("active", 8)
        .run("punish_counter", 20);
    m.right
        .until(320)
        .hold("stun", 8)
        .run("stun", 30)
        .run("counter", 4)
        .run("active", 4);
    // P2 は不利のままボタンを押す（暴れ）。
    m.press(
        1,
        358,
        362,
        InputDir::Neutral,
        vec![circle(BadgeColor::Red)],
        false,
    );

    // f420: P2 の技が空振りし、P1 が差し返す。
    m.left.until(420).run("empty", 40).run("counter", 6);
    m.right
        .until(420)
        .run("active", 12)
        .run("punish_counter", 38);
    m.press(
        0,
        458,
        462,
        InputDir::Neutral,
        vec![circle(BadgeColor::Red)],
        false,
    );
    m.left.until(470).hold("active", 9);
    m.right.until(470).hold("stun", 9);
    m.drain(1, 474, 494, 0.2);

    // f560: P1 が投げる。
    m.press(
        0,
        556,
        560,
        InputDir::Neutral,
        vec![circle(BadgeColor::Green)],
        true,
    );
    m.left.until(560).run("counter", 6).hold("active", 8);
    m.right.until(566).hold("stun", 8);
    m.drain(1, 570, 590, 0.1);

    // f640: P1 のドライブインパクト。
    m.press(
        0,
        636,
        640,
        InputDir::Neutral,
        vec![boxed(BadgeColor::Green)],
        false,
    );
    m.left.until(646).run("inv_full", 8).run("active", 8);
    m.right.until(646).run("empty", 16);
    m.left.until(664).hold("active", 8);
    m.right.until(664).hold("stun", 8);

    // f720: P2 が無敵技をぶっぱするがガードされ、後隙を狩られる。
    m.press(
        1,
        716,
        720,
        InputDir::Neutral,
        vec![boxed(BadgeColor::Blue)],
        false,
    );
    m.right.until(726).run("inv_full", 10).run("active", 8);
    m.left.until(726).run("empty", 18);
    m.right.until(744).hold("active", 9);
    m.left.until(744).hold("stun", 9);
    m.right.until(753).run("punish_counter", 8);
    m.left.until(753).run("empty", 8);
    m.left.until(761).hold("active", 9);
    m.right.until(761).hold("stun", 9);
    m.drain(1, 765, 785, 0.1);

    // f820: P1 がジャンプして飛び込む。
    m.press(0, 820, 860, InputDir::Up, vec![], false);
    m.left.until(820).run("counter", 40);
    m.right.until(820).run("empty", 40);
    m.left.until(866).hold("active", 9);
    m.right.until(866).hold("stun", 9);
    m.drain(1, 870, 890, 0.1);

    // f900: P1 が飛び道具を撃つ。画面に残る間ずっと脅威が続く。
    m.left.until(900).run("projectile_active", 24);
    m.right.until(900).run("empty", 24);

    // f960: P1 のドライブラッシュ（パリィ表示 + 前入力）。
    m.press(0, 956, 968, InputDir::Right, vec![], false);
    m.left.until(960).run("parry", 20);
    m.right.until(960).run("empty", 20);
    m.spend_drive(0, 962, 0.5);
    m.spend_drive(0, 962, 0.5);

    // f1020: SA の暗転。両者が長く止まり、P1 のゲージが 3 本から 0 本へ。
    m.left.until(1_020).hold("empty", 40);
    m.right.until(1_020).hold("empty", 40);
    m.spend_super(0, 1_030, 0.0);
    m.left.until(1_066).hold("active", 10);
    m.right.until(1_066).hold("stun", 10);
    m.drain(1, 1_070, 1_100, 0.2);

    // f1120: P2 がバーンアウトする。
    m.burnout(1, 1_120, 1_300);

    // f1160: P2 がガードを固めていたのに前へ歩いて崩れる。
    m.left.until(1_160).run("empty", 30);
    m.right.until(1_160).run("stun", 30);
    for frame in 1_160..1_190 {
        m.inputs[1][frame] = tracked(
            (frame - 1_160) as u32 + 1,
            InputDir::DownRight,
            vec![],
            false,
            false,
        );
    }
    m.press(1, 1_190, 1_194, InputDir::Left, vec![], false);
    m.left.until(1_192).hold("active", 9);
    m.right.until(1_192).hold("stun", 9);
    m.drain(1, 1_196, 1_220, 0.1);

    // 残りは無表示のまま KO まで。
    m.drain(1, 1_400, 1_450, 1.0);
    m.left.until(FRAMES as i64 - 1);
    m.right.until(FRAMES as i64 - 1);

    m
}

// ── 試合の骨格 ───────────────────────────────────────────────────────────

/// 全快の持続からラウンドが始まり、KO で終わる。
#[test]
fn the_match_is_split_into_a_round_that_ends_in_a_knockout() {
    let events = synth_match().build();

    assert_eq!(events.rounds.len(), 1, "{:?}", events.rounds);
    assert_eq!(events.rounds[0].winner, Some(1));
}

/// 各抽出器へ観測が届いている。届かなければ、その種類の指摘だけが
/// 静かに空になる。
#[test]
fn every_kind_of_event_is_extracted_from_one_match() {
    let events = synth_match().build();

    assert!(!events.damage.is_empty(), "被弾");
    assert!(!events.contacts.is_empty(), "接触");
    assert!(!events.punishes.is_empty(), "確定反撃の機会");
    assert!(!events.whiffs.is_empty(), "空振り");
    assert!(!events.knockdowns.is_empty(), "ダウン");
    assert!(!events.presses_while_minus.is_empty(), "不利中の暴れ");
    assert!(!events.minus_situations.is_empty(), "不利状況");
    assert!(!events.advantage_situations.is_empty(), "有利状況");
    assert!(!events.throw_actions.is_empty(), "投げ");
    assert!(!events.throws.is_empty(), "確定した投げ");
    assert!(!events.drive_impacts.is_empty(), "ドライブインパクト");
    assert!(!events.drive_rushes.is_empty(), "ドライブラッシュ");
    assert!(!events.reversals.is_empty(), "無敵技");
    assert!(!events.guard_breaks.is_empty(), "ガード崩れ");
    assert!(!events.super_arts.is_empty(), "SA");
    assert!(!events.burnouts.is_empty(), "バーンアウト");
    assert!(!events.jumps.is_empty(), "ジャンプ");
    assert!(!events.projectiles.is_empty(), "飛び道具");
    assert!(
        events.attack_evidence.damage.is_empty(),
        "中央表示が無ければ裏付けも無い"
    );
    assert!(!events.segments.iter().all(Vec::is_empty), "入力セグメント");
}

/// 入力欄が読めていたフレーム数を数える。読めていなかった時間を
/// 「何もしなかった」と混ぜないための分母になる。
#[test]
fn the_input_coverage_counts_observed_and_repaired_frames_apart() {
    let mut m = synth_match();
    // 序盤の 100 フレームだけ、トラッカーが推測で埋めたことにする。
    for input in &mut m.inputs[0][0..100] {
        input.repaired = true;
    }
    // 別の 50 フレームは読めなかったことにする。
    for input in &mut m.inputs[1][0..50] {
        input.uncertain = true;
    }

    let events = m.build();
    let coverage = &events.input_coverage;

    assert!(coverage.measured);
    assert_eq!(
        coverage.p1_repaired_frames, 100,
        "補修フレームを数えていない"
    );
    assert!(
        coverage.p1_observed_frames > 0
            && coverage.p1_repaired_frames < coverage.p1_observed_frames
    );
    assert_eq!(
        coverage.p2_observed_frames + 50,
        coverage.p1_observed_frames + coverage.p1_repaired_frames,
        "読めなかったフレームを直接観測に数えている"
    );
    assert_eq!(coverage.p2_repaired_frames, 0);
}

/// ラウンドの外は分母に入れない。試合が始まる前の入力欄も読めている。
#[test]
fn frames_outside_the_rounds_are_not_counted_as_coverage() {
    let inside = synth_match().build().input_coverage.p1_observed_frames;

    let mut m = synth_match();
    // 最後の 200 フレームを試合画面の外にする。
    for feature in &mut m.features[1_600..] {
        feature.is_match_screen = false;
    }
    let outside = m.build().input_coverage.p1_observed_frames;

    assert_eq!(inside, outside, "ラウンド外のフレームを数えている");
}

/// フレームメーターの読みは、動画フレームごとの列として持ち出される。
/// 後段はこの列だけを見るので、どれか一つでも空なら判断材料が消える。
#[test]
fn the_meter_is_projected_onto_every_video_frame() {
    let events = synth_match().build();

    for side in 0..2 {
        assert_eq!(events.meter_state[side].len(), FRAMES, "状態列 {side}");
        assert_eq!(
            events.meter_game_frame[side].len(),
            FRAMES,
            "ゲーム時刻 {side}"
        );
        assert_eq!(events.meter_confidence[side].len(), FRAMES, "確度 {side}");
        assert!(
            events.meter_state[side]
                .iter()
                .any(|state| *state != MeterState::Free),
            "状態が全て無表示 {side}"
        );
        assert!(
            events.meter_game_frame[side].iter().any(|gf| *gf >= 0),
            "ゲーム時刻が全て未観測 {side}"
        );
        assert!(
            events.meter_confidence[side].iter().any(|c| *c > 0.0),
            "確度が全て 0 {side}"
        );
    }
}

/// メーターが無ければ、メーター由来のイベントは出ない。HP だけの
/// パイプラインでも落ちずに動く。
#[test]
fn without_a_meter_the_meter_derived_events_are_empty() {
    let m = synth_match();
    let events = build_match_events(&m.features, &m.inputs[0], &m.inputs[1], None, "p1");

    assert!(!events.rounds.is_empty(), "ラウンドは HP だけでも割れる");
    assert!(events.contacts.is_empty());
    assert!(events.meter_state.iter().all(Vec::is_empty));
    assert!(events.meter_game_frame.iter().all(Vec::is_empty));
    assert!(events.meter_confidence.iter().all(Vec::is_empty));
}

// ── 別の入口から入っても同じ試合になる ───────────────────────────────────

/// `FIGHT` の表示で区切る入口でも、観測は同じように渡る。ここを
/// 取りこぼすと、ブラウザ側のパイプラインだけが空の指摘を返す。
#[test]
fn the_fight_banner_entry_point_carries_the_same_observations() {
    let m = synth_match();
    let left = m.left.build();
    let right = m.right.build();
    let context = crate::context::AnalysisContext::new("p1");
    let markers = [FightMarker {
        first_frame: 0,
        last_frame: 5,
        peak_frame: 2,
        peak_score: 1.0,
    }];

    let events = build_match_events_with_context_and_fight_markers(
        &m.features,
        &m.inputs[0],
        &m.inputs[1],
        Some((&left, &right)),
        &context,
        &markers,
    );

    assert_eq!(events.rounds.len(), 1, "{:?}", events.rounds);
    assert_eq!(
        events.rounds[0].start_frame, 5,
        "FIGHT の表示から始めていない"
    );
    assert!(!events.contacts.is_empty(), "メーターが渡っていない");
    assert!(!events.jumps.is_empty(), "入力が渡っていない");
    assert!(!events.presses_while_minus.is_empty(), "入力が渡っていない");
}

/// 中央表示があれば、被弾に技の情報が結び付く。
#[test]
fn the_attack_info_entry_point_attaches_evidence_to_the_damage() {
    let m = synth_match();
    let left = m.left.build();
    let right = m.right.build();
    let context = crate::context::AnalysisContext::new("p1");

    // f200 のヒットに合わせて、中央表示が 2 段のコンボを出す。
    let idle = || AttackInfoSide {
        last_damage: 0,
        scaling_percent: 100,
        combo_damage: 0,
        max_combo_damage: 0,
        attribute: AttackAttribute::Middle,
    };
    let step = |last: u32, combo: u32, scaling: u32| AttackInfoSide {
        last_damage: last,
        scaling_percent: scaling,
        combo_damage: combo,
        max_combo_damage: combo,
        attribute: AttackAttribute::Middle,
    };
    let attack_info: Vec<AttackInfoObservation> = [
        (198, idle()),
        (202, step(800, 800, 100)),
        (212, step(600, 1_400, 80)),
        (260, idle()),
    ]
    .into_iter()
    .map(|(frame_index, p1)| AttackInfoObservation {
        frame_index,
        p1,
        p2: idle(),
    })
    .collect();

    let with_info = build_match_events_with_context_and_attack_info(
        &m.features,
        &m.inputs[0],
        &m.inputs[1],
        Some((&left, &right)),
        &context,
        &attack_info,
    );

    assert!(
        !with_info.attack_evidence.sequences.is_empty(),
        "中央表示からコンボを組み立てていない"
    );
    assert!(
        !with_info.attack_evidence.damage.is_empty(),
        "被弾に技の情報を結び付けていない"
    );
    assert!(
        m.build().attack_evidence.damage.is_empty(),
        "中央表示が無いのに裏付けが付いている"
    );
}

// ── 被弾の位置合わせ ─────────────────────────────────────────────────────

/// 被弾は、HP が減り始めたフレームではなく当たったフレームに置く。
/// ヒットストップ中はバーがまだ動かない。
#[test]
fn the_damage_is_anchored_to_the_contact_that_caused_it() {
    let events = synth_match().build();

    let first = events
        .damage
        .iter()
        .find(|damage| damage.victim == 2)
        .expect("P2 の被弾");

    assert_eq!(first.start_frame, 200, "当たった瞬間へ寄せていない");
    assert!(
        events
            .damage
            .iter()
            .all(|damage| damage.end_frame >= damage.start_frame),
        "寄せた結果、終端が始端より前になっている"
    );
}

/// 演出で止まっている間に始まった被弾は、演出の頭を手前の目印にする。
/// クリップの前置き時間が演出に食われて、直前の行動が映らなくなる。
#[test]
fn damage_after_a_freeze_remembers_where_the_freeze_started() {
    let events = synth_match().build();

    let after_freeze = events
        .damage
        .iter()
        .find(|damage| damage.pre_freeze_frame < damage.start_frame)
        .expect("演出直後の被弾");

    assert_eq!(after_freeze.pre_freeze_frame, 1_020, "演出の頭を見ていない");
    assert!(
        events
            .damage
            .iter()
            .filter(|damage| damage.start_frame < 1_000)
            .all(|damage| damage.pre_freeze_frame == damage.start_frame),
        "演出が無い被弾にも目印を付けている"
    );
}

/// 手前の目印はラウンドの頭より前へは戻さない。前のラウンドの演出まで
/// クリップに入れない。
#[test]
fn the_freeze_anchor_never_reaches_before_the_round() {
    let mut m = synth_match();
    // ラウンドの開始前から続く長い演出を置く。
    m.left = MeterBuilder::new();
    m.right = MeterBuilder::new();
    m.left.hold("empty", 220);
    m.right.hold("empty", 220);
    m.left.until(FRAMES as i64 - 1);
    m.right.until(FRAMES as i64 - 1);

    // 開始位置を FIGHT の表示で後ろへずらし、演出の頭がラウンドの外に
    // なるようにする。
    let left = m.left.build();
    let right = m.right.build();
    let context = crate::context::AnalysisContext::new("p1");
    let markers = [FightMarker {
        first_frame: 100,
        last_frame: 120,
        peak_frame: 110,
        peak_score: 1.0,
    }];
    let events = build_match_events_with_context_and_fight_markers(
        &m.features,
        &m.inputs[0],
        &m.inputs[1],
        Some((&left, &right)),
        &context,
        &markers,
    );

    let round_start = events.rounds[0].start_frame;
    assert_eq!(round_start, 120, "{:?}", events.rounds);

    let after_freeze = events
        .damage
        .iter()
        .find(|damage| damage.start_frame < 300)
        .expect("演出中に始まった被弾");

    assert_eq!(
        after_freeze.pre_freeze_frame, round_start,
        "ラウンドの頭より前へ戻している"
    );
}

// ── メーターが無いときの投げ ─────────────────────────────────────────────

/// メーターが読めていなければ、投げは入力表示だけから拾う。
/// ラウンドの外の投げは数えないが、そこで走査を打ち切らない。
#[test]
fn without_a_meter_the_throws_come_from_the_input_panel_alone() {
    // 全快が二度続き、間に実ダメージが無い。ラウンドの開始は後ろの
    // 全快になるので、前半は試合画面のままラウンドの外になる。
    let mut features: Vec<FrameFeatures> = Vec::new();
    let push = |features: &mut Vec<FrameFeatures>, count: usize, left: f32, right: f32| {
        for _ in 0..count {
            let frame = features.len() as u32;
            features.push(feat(frame, left, right));
        }
    };
    push(&mut features, 100, 1.0, 1.0);
    push(&mut features, 30, 0.9, 0.9);
    push(&mut features, 100, 1.0, 1.0);
    push(&mut features, 200, 1.0, 0.5);

    let mut inputs: Vec<TrackedInput> = (0..features.len())
        .map(|k| tracked(k as u32 % 60 + 1, InputDir::Neutral, vec![], false, false))
        .collect();
    let throw_at = |inputs: &mut Vec<TrackedInput>, frame: usize| {
        for offset in 0..5 {
            inputs[frame + offset] = tracked(
                offset as u32 + 1,
                InputDir::Neutral,
                vec![circle(BadgeColor::Green)],
                false,
                true,
            );
        }
    };
    throw_at(&mut inputs, 20);
    throw_at(&mut inputs, 300);

    let events = build_match_events(&features, &inputs, &[], None, "p1");

    assert_eq!(events.rounds[0].start_frame, 130, "{:?}", events.rounds);
    assert!(
        events.throws.iter().any(|throw| throw.frame == 300),
        "ラウンド外の投げで走査を打ち切っている: {:?}",
        events.throws
    );
    assert!(
        events.throws.iter().all(|throw| throw.frame != 20),
        "ラウンド外の投げを拾っている"
    );
}

/// 硬直が途切れていない間の被弾は一連のコンボ。フレームメーターの
/// 硬直表示が抽出器へ渡らないと、長い運びコンボが二度の被弾に割れる。
#[test]
fn an_unbroken_stun_on_the_meter_keeps_a_long_combo_together() {
    let mut m = Match::new();
    m.left.until(200);
    m.right.until(200);
    // P1 が当ててから、P2 は 200 フレーム硬直したまま運ばれる。
    m.left.hold("active", 10).run("empty", 240);
    m.right.hold("stun", 10).run("stun", 240);
    m.drain(1, 205, 225, 0.2);
    m.drain(1, 380, 400, 0.2);
    m.left.until(FRAMES as i64 - 1);
    m.right.until(FRAMES as i64 - 1);

    let left = m.left.build();
    let right = m.right.build();
    let joined = build_match_events(
        &m.features,
        &m.inputs[0],
        &m.inputs[1],
        Some((&left, &right)),
        "p1",
    );
    let without_meter = build_match_events(&m.features, &m.inputs[0], &m.inputs[1], None, "p1");

    assert_eq!(
        joined.damage.iter().filter(|d| d.victim == 2).count(),
        1,
        "硬直の継続が抽出器へ渡っていない: {:?}",
        joined.damage
    );
    assert_eq!(
        without_meter
            .damage
            .iter()
            .filter(|d| d.victim == 2)
            .count(),
        2,
        "硬直が分からなければ別々の被弾"
    );
}
