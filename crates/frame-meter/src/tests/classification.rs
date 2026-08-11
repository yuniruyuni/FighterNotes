use crate::classification::fresh_v_min_for;
use crate::palette::PaletteName;
use crate::{
    classify_cell_pair, classify_cell_raw, BrightClass, CellState, EMPTY_V_MAX, STRIPE_WF_MIN,
};

use super::{assert_close, bgr_from_hsv};

#[test]
fn fresh_thresholds_exist_only_for_colored_states() {
    let expected = [
        (CellState::Counter, 172.0),
        (CellState::PunishCounter, 145.0),
        (CellState::MotionRecovery, 216.0),
        (CellState::Active, 157.0),
        (CellState::ProjectileActive, 160.0),
        (CellState::Stun, 219.0),
        (CellState::Parry, 95.0),
    ];
    for (state, threshold) in expected {
        assert_close(fresh_v_min_for(&state).unwrap(), threshold);
    }
    for state in [
        CellState::InvFull,
        CellState::InvStrike,
        CellState::InvProj,
        CellState::Empty,
        CellState::Other,
        CellState::Unknown,
    ] {
        assert_eq!(fresh_v_min_for(&state), None);
    }
}

#[test]
fn pair_distance_limit_is_inclusive_for_each_sample() {
    let boundary = [187.0, 17.0, 65.0];
    let black = PaletteName::Black.color();

    assert_eq!(
        classify_cell_pair(boundary, black),
        (CellState::Parry, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(black, boundary),
        (CellState::Parry, BrightClass::Fresh)
    );
}

#[test]
fn pair_stripe_rules_require_both_expected_palette_roles() {
    let white = PaletteName::White.color();
    let gray = PaletteName::Gray.color();
    let active = PaletteName::Active.color();
    let counter = PaletteName::Counter.color();

    assert_eq!(
        classify_cell_pair(white, active),
        (CellState::Active, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(counter, gray),
        (CellState::Counter, BrightClass::Fresh)
    );
}

#[test]
fn pair_stripe_rules_reject_distant_palette_lookalikes() {
    let rejected_whiteish = [252.0, 84.0, 156.0];
    let rejected_pink = [170.0, 0.0, 255.0];
    let rejected_orange = [0.0, 30.0, 255.0];
    let white = PaletteName::White.color();

    for (stripe, expected) in [
        (PaletteName::StripePink.color(), CellState::InvStrike),
        (PaletteName::StripeOrange.color(), CellState::InvProj),
    ] {
        assert_eq!(
            classify_cell_pair(rejected_whiteish, stripe),
            (CellState::Other, BrightClass::None_),
            "a rejected first sample must not create {expected:?}"
        );
    }
    for rejected_stripe in [rejected_pink, rejected_orange] {
        assert_eq!(
            classify_cell_pair(white, rejected_stripe),
            (CellState::Other, BrightClass::None_)
        );
    }
}

#[test]
fn pair_same_family_is_fresh_only_when_both_samples_are_fresh() {
    assert_eq!(
        classify_cell_pair(
            PaletteName::Counter.color(),
            PaletteName::CounterDim.color()
        ),
        (CellState::Counter, BrightClass::Low)
    );
}

#[test]
fn pair_colored_samples_use_both_family_identity_and_brightness() {
    assert_eq!(
        classify_cell_pair(
            PaletteName::CounterDim.color(),
            PaletteName::Counter.color()
        ),
        (CellState::Counter, BrightClass::Low)
    );
    assert_eq!(
        classify_cell_pair(PaletteName::Counter.color(), PaletteName::Active.color()),
        (CellState::Active, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(PaletteName::Counter.color(), PaletteName::ActiveDim.color()),
        (CellState::Active, BrightClass::Low)
    );
}

#[test]
fn pair_stripe_brightness_requires_both_samples_to_be_full_palette_colors() {
    for (second, expected) in [
        (PaletteName::GrayDim, CellState::InvFull),
        (PaletteName::StripePinkDim, CellState::InvStrike),
        (PaletteName::StripeOrangeDim, CellState::InvProj),
    ] {
        assert_eq!(
            classify_cell_pair(PaletteName::White.color(), second.color()),
            (expected, BrightClass::Low)
        );
    }
}

#[test]
fn pair_empty_requires_both_samples_to_be_emptyish() {
    assert_eq!(
        classify_cell_pair(PaletteName::Black.color(), PaletteName::Gray.color()),
        (CellState::Other, BrightClass::None_)
    );
    assert_eq!(
        classify_cell_pair(PaletteName::Gray.color(), PaletteName::Black.color()),
        (CellState::Other, BrightClass::None_)
    );
}

#[test]
fn raw_classification_preserves_strict_boundaries() {
    assert_eq!(
        classify_cell_raw([0.0; 3], STRIPE_WF_MIN, None),
        CellState::Empty
    );
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(50.0, 60.0, 100.0), 0.0, Some(true)),
        CellState::InvProj
    );
    assert_eq!(
        classify_cell_raw([EMPTY_V_MAX; 3], 0.0, Some(false)),
        CellState::Other
    );
}

#[test]
fn raw_counter_accepts_either_low_hue_or_high_saturation_evidence() {
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(80.0, 100.0, 180.0), 0.0, Some(false)),
        CellState::Counter
    );
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(90.0, 210.0, 180.0), 0.0, Some(false)),
        CellState::Counter
    );
}

#[test]
fn raw_attack_includes_its_exact_hue_and_saturation_boundaries() {
    assert_eq!(raw(145, 40), CellState::Active);
    assert_eq!(raw(160, 40), CellState::Active);
}

// ── 生の色から状態を読む ─────────────────────────────────────────────────

/// 色相と彩度の帯。SF6 が実際に使う色の並びで、隣り合う帯の境目が
/// そのまま状態の境目になる。境目がずれると、隣の状態として読む。
const BANDS: &[(&str, i32, i32, i32, CellState)] = &[
    ("弾の攻撃判定", 9, 21, 100, CellState::ProjectileActive),
    ("硬直", 22, 38, 55, CellState::Stun),
    ("発生", 75, 85, 40, CellState::Counter),
    ("移動硬直", 86, 100, 40, CellState::MotionRecovery),
    ("後隙", 101, 137, 40, CellState::PunishCounter),
    ("パリィ", 138, 152, 150, CellState::Parry),
];

/// 指定の色相・彩度で読み取れるセル。彩度は切り捨てで整数に落ちるので、
/// 帯の中央にあたる値を渡して丸めの揺れを避ける。
fn raw(hue: i32, saturation: i32) -> CellState {
    let bgr = bgr_from_hsv(hue as f32, saturation as f32 + 0.5, 200.0);
    classify_cell_raw(bgr, 0.0, Some(false))
}

/// 帯の内側は、その帯の状態。
#[test]
fn each_band_of_hue_names_one_state() {
    for (label, low, high, saturation, state) in BANDS {
        for hue in *low..=*high {
            assert_eq!(
                raw(hue, *saturation),
                state.clone(),
                "{label}: 色相 {hue} を読み違えている"
            );
        }
    }
}

/// 帯の外側は、その帯の状態にならない。
#[test]
fn the_edges_of_each_band_are_where_the_meaning_changes() {
    for (label, low, high, saturation, state) in BANDS {
        assert_ne!(
            raw(low - 1, *saturation),
            state.clone(),
            "{label}: 下の境目の外まで読んでいる"
        );
        assert_ne!(
            raw(high + 1, *saturation),
            state.clone(),
            "{label}: 上の境目の外まで読んでいる"
        );
    }
}

/// 攻撃判定の赤は色相の環をまたぐ。上端と下端の両方に現れる。
#[test]
fn the_attack_red_wraps_around_both_ends_of_the_hue_circle() {
    for hue in [0, 4, 8] {
        assert_eq!(raw(hue, 60), CellState::Active, "色相 {hue}");
    }
    for hue in [153, 165, 179] {
        assert_eq!(raw(hue, 60), CellState::Active, "色相 {hue}");
    }
    assert_ne!(raw(9, 60), CellState::Active, "弾の帯まで赤にしている");
}

/// パリィの紫は攻撃判定の赤と色相が重なる。彩度の濃さで分ける。
#[test]
fn the_parry_purple_wins_over_the_attack_red_where_they_overlap() {
    assert_eq!(raw(148, 150), CellState::Parry);
    assert_eq!(
        raw(148, 149),
        CellState::Active,
        "薄い紫までパリィにしている"
    );
}

/// どの帯にも、それだけの濃さが要る。薄い色は何とも言えない。
#[test]
fn a_colour_too_pale_for_its_band_is_not_read_as_that_state() {
    for (label, low, high, saturation, state) in BANDS {
        let middle = (low + high) / 2;

        assert_eq!(
            raw(middle, *saturation),
            state.clone(),
            "{label}: ちょうどの濃さを落としている"
        );
        assert_ne!(
            raw(middle, saturation - 1),
            state.clone(),
            "{label}: 薄すぎる色まで読んでいる"
        );
    }
}

/// 発生の緑と移動硬直の黄緑は隣り合う。色相で分けきれない濃い色は、
/// 濃さで発生と判断する。
#[test]
fn a_deeply_saturated_yellow_green_is_still_the_startup_colour() {
    assert_eq!(raw(95, 199), CellState::MotionRecovery);
    assert_eq!(
        raw(95, 200),
        CellState::Counter,
        "濃い緑を移動硬直にしている"
    );
}

/// 暗すぎるセルは空。何色にも読まない。
#[test]
fn a_cell_too_dark_to_read_is_empty() {
    let dark = bgr_from_hsv(30.0, 200.0, EMPTY_V_MAX - 1.0);
    let lit = bgr_from_hsv(30.0, 200.0, EMPTY_V_MAX);

    assert_eq!(classify_cell_raw(dark, 0.0, Some(false)), CellState::Empty);
    assert_eq!(classify_cell_raw(lit, 0.0, Some(false)), CellState::Stun);
}

/// 縞模様なら無敵。彩度が無ければ全無敵、あれば色で打撃無敵と飛び道具
/// 無敵を分ける。
#[test]
fn a_striped_cell_is_read_as_invincibility() {
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(30.0, 59.0, 200.0), 0.0, Some(true)),
        CellState::InvFull
    );
    assert_eq!(
        classify_cell_raw(bgr_from_hsv(30.0, 60.0, 200.0), 0.0, Some(true)),
        CellState::InvProj,
        "彩度のある縞を全無敵にしている"
    );
    for hue in [0.0, 8.0, 145.0, 179.0] {
        assert_eq!(
            classify_cell_raw(bgr_from_hsv(hue, 200.0, 200.0), 0.0, Some(true)),
            CellState::InvStrike,
            "色相 {hue}"
        );
    }
}

/// 縞かどうかを指定しなければ、白の割合から決める。
#[test]
fn without_a_verdict_the_white_share_decides_whether_it_is_striped() {
    let purple = bgr_from_hsv(30.0, 200.0, 200.0);

    assert_eq!(
        classify_cell_raw(purple, STRIPE_WF_MIN + 0.01, None),
        CellState::InvProj
    );
    assert_eq!(
        classify_cell_raw(purple, STRIPE_WF_MIN, None),
        CellState::Stun,
        "白の割合がちょうどの境目で縞にしている"
    );
}
