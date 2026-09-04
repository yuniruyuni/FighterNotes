//! 精度計測用のシナリオ。
//!
//! 各シナリオは実映像で精度を制限している状況を 1 つずつ切り出す。
//! 冒頭に両者が動く助走区間を置くのは、候補区間がイベント直前から
//! 始まる production の窓と同じく、追跡の初期化に双方のモーションが
//! 要るためである。助走は `measure == false` として集計から除く。

use spatial_refine::spatial::{ActorHint, SpatialHints};

use crate::sim::{Animation, Camera, GtFrame, Scene, Spark, CHAR_HALF_WIDTH, STAGE_HALF_WIDTH};

pub type StepSink<'a> = &'a mut dyn FnMut(GtFrame, Vec<u8>, SpatialHints);

pub struct Scenario {
    pub name: &'static str,
    pub description: &'static str,
    pub run: fn(StepSink),
}

pub fn all() -> Vec<Scenario> {
    vec![
        Scenario {
            name: "neutral_walk",
            description: "両者が歩いて接近・後退。カメラのパンとズームが伴う",
            run: neutral_walk,
        },
        Scenario {
            name: "stationary_guard",
            description: "P2 がガード硬直で静止し、P1 だけが動き続ける",
            run: stationary_guard,
        },
        Scenario {
            name: "jump_crossup",
            description: "P2 が P1 を跳び越えて着地する(めくり)",
            run: jump_crossup,
        },
        Scenario {
            name: "hit_contact",
            description: "打撃 3 回(ヒット2・ガード1)。hitstop 中にスパークが出る",
            run: hit_contact,
        },
        Scenario {
            name: "crossed_window_start",
            description: "確定 window で色を学習後、入れ替わった状態から次の window が始まる",
            run: crossed_window_start,
        },
        Scenario {
            name: "corner_clamp",
            description: "画面端でカメラのパンがクランプされた状態の距離変化",
            run: corner_clamp,
        },
    ]
}

fn no_hints() -> SpatialHints {
    SpatialHints {
        p1: ActorHint {
            anchor: None,
            allow_discontinuity: false,
            allow_airborne: false,
        },
        p2: ActorHint {
            anchor: None,
            allow_discontinuity: false,
            allow_airborne: false,
        },
        contact_effect: false,
        sides_certain: false,
    }
}

fn airborne_hint_p2() -> SpatialHints {
    SpatialHints {
        p2: ActorHint {
            anchor: None,
            allow_discontinuity: false,
            allow_airborne: true,
        },
        ..no_hints()
    }
}

/// 第一段が contact frame を確定している状況を模したヒント。
fn contact_hint() -> SpatialHints {
    SpatialHints {
        contact_effect: true,
        sides_certain: false,
        ..no_hints()
    }
}

/// Round 開始直後の、側が確定しているフレームのヒント。
fn certain_hint() -> SpatialHints {
    SpatialHints {
        sides_certain: true,
        ..no_hints()
    }
}

/// 状態を進めた後のフレームを描画して 1 step 送る。
fn emit(scene: &mut Scene, measure: bool, hints: SpatialHints, sink: StepSink) {
    scene.camera = Camera::follow(scene.p1.x, scene.p2.x);
    scene.p1.advance_phase();
    scene.p2.advance_phase();
    let gt = scene.ground_truth(measure);
    scene.reset_next = false;
    let frame = scene.render();
    sink(gt, frame, hints);
    scene.frame_index += 1;
    for spark in &mut scene.sparks {
        spark.age += 1;
    }
}

/// 追跡初期化のための助走。左右に小さく足踏みし、双方に確実な
/// モーション領域を作る。
fn warmup(scene: &mut Scene, frames: u32, sink: StepSink) {
    scene.p1.anim = Animation::Full;
    scene.p2.anim = Animation::Full;
    for i in 0..frames {
        let step = if i % 8 < 4 { 0.02 } else { -0.02 };
        scene.p1.x += step;
        scene.p2.x -= step;
        emit(scene, false, no_hints(), sink);
    }
}

fn neutral_walk(sink: StepSink) {
    let mut scene = Scene::new(-2.8, 2.8);
    warmup(&mut scene, 14, sink);
    for _ in 0..70 {
        scene.p1.x += 0.03;
        scene.p2.x -= 0.03;
        emit(&mut scene, true, no_hints(), sink);
    }
    scene.p1.anim = Animation::Subtle;
    scene.p2.anim = Animation::Subtle;
    for _ in 0..30 {
        emit(&mut scene, true, no_hints(), sink);
    }
    scene.p1.anim = Animation::Full;
    scene.p2.anim = Animation::Full;
    for _ in 0..70 {
        scene.p1.x -= 0.025;
        scene.p2.x += 0.025;
        emit(&mut scene, true, no_hints(), sink);
    }
}

fn stationary_guard(sink: StepSink) {
    // 初期化はモーション領域が分離できる間合いで行い、その後に密着する。
    // 密着したまま助走すると 2 領域が 1 つへマージされ、追跡が始まらない。
    let mut scene = Scene::new(-1.3, 0.9);
    warmup(&mut scene, 14, sink);
    scene.p2.anim = Animation::Subtle;
    while scene.p2.x - scene.p1.x > 1.2 {
        scene.p1.x += 0.05;
        emit(&mut scene, true, no_hints(), sink);
    }
    for _ in 0..3 {
        // P2 はガード硬直で完全静止。P1 は攻撃動作で動き続ける。
        scene.p2.anim = Animation::Frozen;
        scene.p1.anim = Animation::Full;
        for i in 0..40u32 {
            scene.p1.x += if i % 10 < 5 { 0.02 } else { -0.02 };
            emit(&mut scene, true, no_hints(), sink);
        }
        // 硬直明けに一瞬だけ動く。
        scene.p2.anim = Animation::Full;
        for _ in 0..8 {
            scene.p2.x -= 0.01;
            emit(&mut scene, true, no_hints(), sink);
        }
        scene.p2.x += 0.08;
    }
}

fn jump_crossup(sink: StepSink) {
    let mut scene = Scene::new(-0.9, 0.9);
    warmup(&mut scene, 14, sink);
    for _ in 0..6 {
        emit(&mut scene, true, no_hints(), sink);
    }
    // 44 フレームの放物線で P1 を跳び越え、反対側へ着地する。
    let air_frames = 44u32;
    let peak = 2.3f32;
    let start_x = scene.p2.x;
    let end_x = -1.5f32;
    scene.p2.anim = Animation::Full;
    for t in 0..=air_frames {
        let progress = t as f32 / air_frames as f32;
        scene.p2.x = start_x + (end_x - start_x) * progress;
        scene.p2.air_y = 4.0 * peak * progress * (1.0 - progress);
        emit(&mut scene, true, airborne_hint_p2(), sink);
    }
    scene.p2.air_y = 0.0;
    scene.p1.anim = Animation::Subtle;
    scene.p2.anim = Animation::Subtle;
    for _ in 0..30 {
        emit(&mut scene, true, no_hints(), sink);
    }
}

fn hit_contact(sink: StepSink) {
    let mut scene = Scene::new(-1.6, 0.6);
    warmup(&mut scene, 14, sink);
    for cycle in 0..3 {
        // 密着まで歩く。
        scene.p1.anim = Animation::Full;
        scene.p2.anim = Animation::Subtle;
        while scene.p2.x - scene.p1.x > 0.95 {
            scene.p1.x += 0.05;
            emit(&mut scene, true, no_hints(), sink);
        }
        // 発生 8 フレームの踏み込み。
        for _ in 0..8 {
            scene.p1.x += 0.012;
            emit(&mut scene, true, no_hints(), sink);
        }
        // hitstop。両者とも凍結し、衝突位置にスパークが出る。
        scene.p1.anim = Animation::Frozen;
        scene.p2.anim = Animation::Frozen;
        // 2 回目はガード(寒色スパーク)、他はヒット(暖色)。
        scene.sparks = vec![Spark {
            world_x: scene.p1.x + 0.75,
            world_y: 1.0,
            age: 0,
            cold: cycle == 1,
        }];
        for _ in 0..9 {
            emit(&mut scene, true, contact_hint(), sink);
        }
        scene.sparks.clear();
        // ノックバックと後隙。
        scene.p1.anim = Animation::Subtle;
        scene.p2.anim = Animation::Full;
        for _ in 0..12 {
            scene.p2.x += 0.04;
            emit(&mut scene, true, no_hints(), sink);
        }
        scene.p2.anim = Animation::Subtle;
        for _ in 0..8 {
            emit(&mut scene, true, no_hints(), sink);
        }
    }
}

fn crossed_window_start(sink: StepSink) {
    // 確定 window: 通常の並びで歩き、色を学習させる(集計外)。
    let mut scene = Scene::new(-1.6, 1.6);
    scene.p1.anim = Animation::Full;
    scene.p2.anim = Animation::Full;
    for _ in 0..20 {
        scene.p1.x += 0.02;
        scene.p2.x -= 0.02;
        emit(&mut scene, false, certain_hint(), sink);
    }
    // window の切れ目。次の window は側が入れ替わった状態から始まる。
    scene.p1.x = 1.4;
    scene.p2.x = -1.4;
    scene.reset_next = true;
    for _ in 0..30 {
        scene.p1.x -= 0.02;
        scene.p2.x += 0.02;
        emit(&mut scene, true, no_hints(), sink);
    }
}

fn corner_clamp(sink: StepSink) {
    let wall = STAGE_HALF_WIDTH - CHAR_HALF_WIDTH;
    let mut scene = Scene::new(2.6, 4.6);
    warmup(&mut scene, 14, sink);
    // P2 が壁まで下がり、P1 が追う。カメラのパンが途中でクランプされる。
    while scene.p2.x < wall - 0.02 {
        scene.p2.x = (scene.p2.x + 0.045).min(wall);
        scene.p1.x += 0.035;
        emit(&mut scene, true, no_hints(), sink);
    }
    // 壁を背負った P2 に対して P1 が出入りし、距離だけが変わる。
    scene.p2.anim = Animation::Subtle;
    for cycle in 0..3 {
        for _ in 0..24 {
            scene.p1.x += 0.05;
            if scene.p2.x - scene.p1.x < 0.9 {
                break;
            }
            emit(&mut scene, true, no_hints(), sink);
        }
        for _ in 0..24 {
            scene.p1.x -= 0.05;
            emit(&mut scene, true, no_hints(), sink);
        }
        if cycle == 2 {
            break;
        }
    }
}
