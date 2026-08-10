use super::*;

const STRIP_WIDTH: usize = 1920;
const STRIP_HEIGHT: usize = 70;

/// テンプレートそのものを置いたら一致度は 1。上へも外れないこと。
/// 正規化を取り違えると 1 を超えた値が出て、閾値が意味を失う。
#[test]
fn template_patch_scores_exactly_one() {
    let strip = strip_with_template(0, 0);
    let score = fight_score_from_hud_strip(&strip, STRIP_WIDTH);

    assert!(
        (score - 1.0).abs() < 0.01,
        "一致度が 1 から外れている: {score}"
    );
}

/// browser の縮小で 1px 程度ずれても拾う。探索は上下左右の四方向とも。
#[test]
fn the_alignment_search_covers_every_direction() {
    for (shift_x, shift_y) in [(2, -1), (-2, 1), (0, 1), (0, -1), (2, 1), (-2, -1)] {
        let strip = strip_with_template(shift_x, shift_y);
        let score = fight_score_from_hud_strip(&strip, STRIP_WIDTH);

        assert!(
            (score - 1.0).abs() < 0.01,
            "({shift_x}, {shift_y}) のずれを拾えていない: {score}"
        );
    }
}

/// パッチがちょうど収まる幅なら読む。1 列足りない幅と取り違えると、
/// 端に寄せた HUD 帯を丸ごと捨てる。
#[test]
fn a_strip_exactly_wide_enough_is_read() {
    const EXACT: usize = FIGHT_PATCH_X + FIGHT_PATCH_WIDTH;
    let mut strip = vec![0u8; EXACT * (FIGHT_PATCH_Y + FIGHT_PATCH_HEIGHT) * 4];
    for y in 0..FIGHT_PATCH_HEIGHT {
        for x in 0..FIGHT_PATCH_WIDTH {
            let value = FIGHT_TEMPLATE[y * FIGHT_PATCH_WIDTH + x];
            let index = ((FIGHT_PATCH_Y + y) * EXACT + FIGHT_PATCH_X + x) * 4;
            strip[index..index + 3].fill(value);
            strip[index + 3] = 255;
        }
    }

    let score = fight_score_from_hud_strip(&strip, EXACT);

    assert!(score > 0.9, "ちょうど収まる帯を捨てている: {score}");
}

/// 明るさは三つの channel から作る。どれか一つだけを見ていると、
/// 色の付いた場面で輪郭が別物になる。
///
/// 赤にテンプレート、緑にその反転を置くと、明るさの勾配は反転する。
/// 反転した輪郭は `FIGHT` ではない。
#[test]
fn the_match_reads_brightness_not_a_single_channel() {
    let mut strip = vec![0u8; STRIP_WIDTH * STRIP_HEIGHT * 4];
    for y in 0..FIGHT_PATCH_HEIGHT {
        for x in 0..FIGHT_PATCH_WIDTH {
            let value = FIGHT_TEMPLATE[y * FIGHT_PATCH_WIDTH + x];
            let index = ((FIGHT_PATCH_Y + y) * STRIP_WIDTH + FIGHT_PATCH_X + x) * 4;
            strip[index] = value;
            strip[index + 1] = 255 - value;
            strip[index + 2] = 0;
            strip[index + 3] = 255;
        }
    }

    let score = fight_score_from_hud_strip(&strip, STRIP_WIDTH);

    assert_eq!(score, 0.0, "反転した輪郭を FIGHT と読んでいる");
}

#[test]
fn flat_patch_does_not_match() {
    let strip = vec![128; STRIP_WIDTH * STRIP_HEIGHT * 4];
    assert_eq!(fight_score_from_hud_strip(&strip, STRIP_WIDTH), 0.0);
}

#[test]
fn temporal_hits_become_one_marker_per_fight_animation() {
    let observations = [
        FightObservation {
            frame: 100,
            score: 0.7,
        },
        FightObservation {
            frame: 104,
            score: 0.8,
        },
        FightObservation {
            frame: 108,
            score: 0.75,
        },
        FightObservation {
            frame: 800,
            score: 0.72,
        },
        FightObservation {
            frame: 804,
            score: 0.88,
        },
        FightObservation {
            frame: 808,
            score: 0.73,
        },
    ];
    assert_eq!(
        detect_fight_markers(&observations),
        vec![
            FightMarker {
                first_frame: 100,
                last_frame: 108,
                peak_frame: 104,
                peak_score: 0.8,
            },
            FightMarker {
                first_frame: 800,
                last_frame: 808,
                peak_frame: 804,
                peak_score: 0.88,
            },
        ]
    );
}

#[test]
fn isolated_or_weak_hits_are_rejected() {
    let observations = [
        FightObservation {
            frame: 100,
            score: 0.9,
        },
        FightObservation {
            frame: 200,
            score: 0.5,
        },
        FightObservation {
            frame: 204,
            score: 0.51,
        },
        FightObservation {
            frame: 208,
            score: 0.52,
        },
    ];
    assert!(detect_fight_markers(&observations).is_empty());
}

fn strip_with_template(shift_x: i16, shift_y: i16) -> Vec<u8> {
    let mut strip = vec![0; STRIP_WIDTH * STRIP_HEIGHT * 4];
    for y in 0..FIGHT_PATCH_HEIGHT {
        for x in 0..FIGHT_PATCH_WIDTH {
            let target_x = x as i16 + shift_x;
            let target_y = y as i16 + shift_y;
            if target_x < 0
                || target_x >= FIGHT_PATCH_WIDTH as i16
                || target_y < 0
                || target_y >= FIGHT_PATCH_HEIGHT as i16
            {
                continue;
            }
            let value = FIGHT_TEMPLATE[y * FIGHT_PATCH_WIDTH + x];
            let index = ((FIGHT_PATCH_Y + target_y as usize) * STRIP_WIDTH
                + FIGHT_PATCH_X
                + target_x as usize)
                * 4;
            strip[index..index + 3].fill(value);
            strip[index + 3] = 255;
        }
    }
    strip
}

// ── 明るさへの落とし方 ───────────────────────────────────────────────────

/// 重みは目の感度に合わせる。緑が最も効き、青が最も効かない。等しく
/// 混ぜると、色の違う場面で輪郭が出たり消えたりする。
#[test]
fn the_luma_weights_follow_the_eye() {
    assert_eq!(luma(255, 0, 0), 77, "赤の重みが違う");
    assert_eq!(luma(0, 255, 0), 149, "緑の重みが違う");
    assert_eq!(luma(0, 0, 255), 29, "青の重みが違う");
}

/// 三つの重みは 256 を成すので、無彩色はそのままの値で通る。
/// 縮んだり伸びたりすると、勾配の強さがテンプレートと食い違う。
#[test]
fn a_grey_pixel_passes_through_unchanged() {
    for value in [0u8, 1, 64, 128, 200, 255] {
        assert_eq!(
            luma(value, value, value),
            i16::from(value),
            "{value} が動いた"
        );
    }
}

/// 混ざった色でも、各成分の寄与を足したものになる。
#[test]
fn a_mixed_colour_is_the_sum_of_its_contributions() {
    assert_eq!(luma(255, 255, 0), 226);
    assert_eq!(luma(255, 0, 255), 106);
}

// ── 入力の門 ─────────────────────────────────────────────────────────────

/// パッチの終わりまで届かないバッファは読まない。届かないまま読むと
/// 範囲外を触る。
#[test]
fn a_strip_that_stops_before_the_patch_scores_zero() {
    let mut strip = strip_with_template(0, 0);
    strip.truncate(STRIP_WIDTH * (FIGHT_PATCH_Y + FIGHT_PATCH_HEIGHT) * 4 - 1);

    assert_eq!(fight_score_from_hud_strip(&strip, STRIP_WIDTH), 0.0);
}

/// パッチが収まらない幅の帯も読まない。読むと行をまたいだ画素を
/// 並べて照合することになる。
#[test]
fn a_strip_too_narrow_for_the_patch_scores_zero() {
    const NARROW: usize = FIGHT_PATCH_X + FIGHT_PATCH_WIDTH - 1;
    let mut strip = vec![0u8; NARROW * (FIGHT_PATCH_Y + FIGHT_PATCH_HEIGHT) * 4];
    for y in 0..FIGHT_PATCH_HEIGHT {
        for x in 0..FIGHT_PATCH_WIDTH {
            let value = FIGHT_TEMPLATE[y * FIGHT_PATCH_WIDTH + x];
            let index = ((FIGHT_PATCH_Y + y) * NARROW + FIGHT_PATCH_X + x) * 4;
            if index + 3 < strip.len() {
                strip[index..index + 3].fill(value);
                strip[index + 3] = 255;
            }
        }
    }

    assert_eq!(fight_score_from_hud_strip(&strip, NARROW), 0.0);
}

// ── 観測をラウンドへまとめる ─────────────────────────────────────────────

fn markers_from(hits: &[(u32, f32)]) -> Vec<FightMarker> {
    let observations: Vec<_> = hits
        .iter()
        .map(|&(frame, score)| FightObservation { frame, score })
        .collect();
    detect_fight_markers(&observations)
}

/// 一定より離れた観測は別のラウンド。近ければ同じ演出の連続。
#[test]
fn distance_decides_whether_hits_belong_to_the_same_round() {
    let together = markers_from(&[
        (100, 0.7),
        (104, 0.8),
        (108, 0.7),
        (132, 0.7),
        (136, 0.7),
        (140, 0.7),
    ]);
    let apart = markers_from(&[
        (100, 0.7),
        (104, 0.8),
        (108, 0.7),
        (133, 0.7),
        (137, 0.7),
        (141, 0.7),
    ]);

    assert_eq!(together.len(), 1, "同じ演出を二つに割っている");
    assert_eq!(apart.len(), 2, "別のラウンドを一つにまとめている");
}

/// 弱すぎる観測は連続の一部にも数えない。数えると、離れた本物の
/// 表示どうしが繋がってしまう。
#[test]
fn a_weak_observation_does_not_join_a_run() {
    let counted = markers_from(&[(100, 0.45), (104, 0.7), (108, 0.45)]);
    let ignored = markers_from(&[(100, 0.44), (104, 0.7), (108, 0.44)]);

    assert_eq!(counted.len(), 1, "閾値ちょうどの観測を捨てている");
    assert_eq!(counted[0].first_frame, 100);
    assert!(ignored.is_empty(), "弱い観測を連続に数えている");
}

/// 連続していても、山が低ければ `FIGHT` の表示ではない。似た輪郭の
/// 演出を拾わないため。
#[test]
fn a_run_without_a_strong_peak_is_not_a_marker() {
    let strong = markers_from(&[(100, 0.5), (104, 0.60), (108, 0.5)]);
    let weak = markers_from(&[(100, 0.5), (104, 0.59), (108, 0.5)]);

    assert_eq!(strong.len(), 1, "閾値ちょうどの山を捨てている");
    assert!(weak.is_empty(), "低い山を表示と読んでいる");
}

/// 数が足りない連続も表示ではない。単発の誤検出を拾わないため。
#[test]
fn a_run_that_is_too_short_is_not_a_marker() {
    let enough = markers_from(&[(100, 0.7), (104, 0.7), (108, 0.7)]);
    let too_few = markers_from(&[(100, 0.7), (104, 0.7)]);

    assert_eq!(enough.len(), 1);
    assert!(too_few.is_empty(), "二回の観測を表示と読んでいる");
}

/// 画素は 4 byte ずつ並ぶ。読む位置がずれると、隣の画素の成分から
/// 明るさを作ることになる。
#[test]
fn a_patch_pixel_is_read_from_its_own_four_bytes() {
    let mut strip = vec![0u8; STRIP_WIDTH * STRIP_HEIGHT * 4];
    let index = (FIGHT_PATCH_Y * STRIP_WIDTH + FIGHT_PATCH_X) * 4;
    strip[index..index + 4].copy_from_slice(&[10, 20, 30, 255]);

    assert_eq!(patch_luma(&strip, STRIP_WIDTH, 0, 0), luma(10, 20, 30));
}
