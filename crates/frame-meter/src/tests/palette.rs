//! フレームメーターの色見本に対するテスト。
//!
//! SF6 のフレームメーターは、キャラクターの状態を色で描く。緑が発生、
//! 赤が攻撃判定、水色が硬直。数フレーム前のセルは同じ色を暗く落として
//! 描かれるので、色は「明るい版」と「暗い版」の対で覚える。
//!
//! ここで固定するのは、どの色が何を意味するかという対応そのもの。
//! 実装をなぞるのではなく、画面で見えるものを書き下す。

use crate::classification::classify_cell_pair;
use crate::palette::{nearest_palette, state_quality, PaletteName};
use crate::{BrightClass, CellState};

use super::assert_close;

/// 明るい色と、その暗い版と、意味。
const FAMILIES: &[(PaletteName, PaletteName, CellState)] = &[
    (
        PaletteName::Counter,
        PaletteName::CounterDim,
        CellState::Counter,
    ),
    (
        PaletteName::MotionRecovery,
        PaletteName::MotionRecoveryDim,
        CellState::MotionRecovery,
    ),
    (
        PaletteName::PunishCounter,
        PaletteName::PunishCounterDim,
        CellState::PunishCounter,
    ),
    (
        PaletteName::Active,
        PaletteName::ActiveDim,
        CellState::Active,
    ),
    (
        PaletteName::ProjectileActive,
        PaletteName::ProjectileActiveDim,
        CellState::ProjectileActive,
    ),
    (PaletteName::Stun, PaletteName::StunDim, CellState::Stun),
    (PaletteName::Parry, PaletteName::ParryDim, CellState::Parry),
];

/// 状態を持たない色。無敵の縞・空セル・ラベル枠に使う。
const COLORLESS: &[PaletteName] = &[
    PaletteName::White,
    PaletteName::WhiteDim,
    PaletteName::Gray,
    PaletteName::GrayDim,
    PaletteName::StripePink,
    PaletteName::StripePinkDim,
    PaletteName::StripeOrange,
    PaletteName::StripeOrangeDim,
    PaletteName::Black,
    PaletteName::Gap,
    PaletteName::LabelBox,
];

/// どの色がどの状態を指すか。
#[test]
fn every_colour_names_the_state_it_draws() {
    for (fresh, dim, state) in FAMILIES {
        assert_eq!(
            fresh.as_colored_entry(),
            Some((state.clone(), false)),
            "{fresh:?} の意味がずれている"
        );
        assert_eq!(
            dim.as_colored_entry(),
            Some((state.clone(), true)),
            "{dim:?} の意味がずれている"
        );
    }
    // 発生の色にはもう一段淡い版がある。暗い版ではない。
    assert_eq!(
        PaletteName::CounterTint.as_colored_entry(),
        Some((CellState::Counter, false))
    );
    for &name in COLORLESS {
        assert_eq!(name.as_colored_entry(), None, "{name:?} に状態を与えている");
    }
}

/// 色見本は全部で 26 個。数え漏らすと、その色のセルが読めなくなる。
#[test]
fn every_colour_is_listed_once() {
    let all = PaletteName::all();

    assert_eq!(all.len(), FAMILIES.len() * 2 + COLORLESS.len() + 1);
    for (index, &name) in all.iter().enumerate() {
        assert!(!all[..index].contains(&name), "{name:?} が二度並んでいる");
    }
}

/// 色見本の色そのものを渡せば、その色見本が返る。二つの色が近すぎると
/// 片方が決して選ばれなくなる。
#[test]
fn each_colour_identifies_itself() {
    for &name in PaletteName::all() {
        let (found, distance) = nearest_palette(name.color());

        assert_eq!(found, name, "{name:?} が別の色見本に吸われている");
        assert_close(distance, 0.0);
    }
}

/// 暗い版は明るい版と同じ状態で、暗いと分かる。暗さが失われると、
/// 数フレーム前のセルを「今まさに出ている」と読んでしまう。
#[test]
fn a_dimmed_cell_keeps_its_state_and_is_read_as_dim() {
    for (fresh, dim, state) in FAMILIES {
        assert_eq!(
            classify_cell_pair(fresh.color(), dim.color()),
            (state.clone(), BrightClass::Low),
            "{dim:?} を暗いと読めていない"
        );
        assert_eq!(
            classify_cell_pair(fresh.color(), fresh.color()),
            (state.clone(), BrightClass::Fresh),
            "{fresh:?} を暗いと読んでいる"
        );
    }
}

/// 暗い版は明るい版を 3/4 に落とした色。黒ではない。
#[test]
fn the_dim_colour_is_the_fresh_one_darkened() {
    for (fresh, dim, _) in FAMILIES {
        let fresh_colour = fresh.color();
        let dim_colour = dim.color();

        for channel in 0..3 {
            assert_close(
                dim_colour[channel],
                (fresh_colour[channel] * 7.5).round() / 10.0,
            );
        }
        assert_ne!(dim_colour, [0.0, 0.0, 0.0], "{dim:?} が黒になっている");
    }
}

#[test]
fn stripe_and_neutral_dim_colours_have_exact_nonzero_anchors() {
    for (name, expected) in [
        (PaletteName::WhiteDim, [177.0, 174.8, 174.8]),
        (PaletteName::GrayDim, [150.0, 147.0, 147.8]),
        (PaletteName::StripePinkDim, [105.0, 60.0, 150.0]),
        (PaletteName::StripeOrangeDim, [30.0, 97.5, 172.5]),
    ] {
        assert_eq!(name.color(), expected, "{name:?}");
    }
}

/// 見分けの役割は色ごとに一つだけ。
#[test]
fn each_colour_has_at_most_one_role() {
    for &name in PaletteName::all() {
        let roles = [
            name.is_whiteish(),
            name.is_grayish(),
            name.is_emptyish(),
            name.is_stripe_pink(),
            name.is_stripe_orange(),
            name.as_colored_entry().is_some(),
        ];

        assert!(
            roles.iter().filter(|held| **held).count() <= 1,
            "{name:?} が複数の役割を持っている"
        );
        assert_eq!(name.state_family(), name.as_colored_entry().map(|e| e.0));
    }
}

/// 無敵の縞に使う色は、白と組んだときだけ意味を持つ。
#[test]
fn the_stripe_colours_only_mean_something_paired_with_white() {
    let white = PaletteName::White.color();

    assert_eq!(
        classify_cell_pair(white, PaletteName::Gray.color()),
        (CellState::InvFull, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(white, PaletteName::StripePink.color()),
        (CellState::InvStrike, BrightClass::Fresh)
    );
    assert_eq!(
        classify_cell_pair(white, PaletteName::StripeOrange.color()),
        (CellState::InvProj, BrightClass::Fresh)
    );
}

/// 空のセルは、黒と隙間の色でできている。
#[test]
fn an_empty_cell_is_black_or_gap_on_both_samples() {
    for &first in &[PaletteName::Black, PaletteName::Gap] {
        for &second in &[PaletteName::Black, PaletteName::Gap] {
            assert_eq!(
                classify_cell_pair(first.color(), second.color()),
                (CellState::Empty, BrightClass::None_),
                "{first:?} と {second:?} を空と読めていない"
            );
        }
    }
}

#[test]
fn nearest_palette_keeps_first_entry_when_distances_tie() {
    let midpoint = [127.748_64, 175.900_99, 16.650_03];
    let (name, distance) = nearest_palette(midpoint);

    assert_eq!(name, PaletteName::Counter);
    assert_close(distance, 31.122_257);
}

#[test]
fn state_quality_is_nearest_distance_within_requested_family() {
    let counter = PaletteName::Counter.color();
    let a = [counter[0] + 3.0, counter[1], counter[2]];
    let b = [counter[0], counter[1] + 4.0, counter[2]];

    assert_close(state_quality(&CellState::Counter, a, b), 3.0);
    assert_close(state_quality(&CellState::Empty, a, b), 0.0);

    let far = [255.0, 0.0, 255.0];
    let near_b = [counter[0], counter[1], counter[2] + 2.0];
    assert_close(state_quality(&CellState::Counter, far, near_b), 2.0);
}
