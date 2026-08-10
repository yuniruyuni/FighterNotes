//! HUD の読みを解析用の値へ均すところに対するテスト。
//!
//! ドライブゲージは残量とバーンアウトの回復で意味が違うのに、どちらも
//! 一つの数字として渡る。CA が撃てるかどうかは、ゲージと残り体力の
//! 両方が要る。
//!
//! どちらも「そう見えたから」ではなく「そう決めたから」その値になる。
//! 決め方を変えると、以降のゲージ収支と CA の判断が全部ずれる。

use super::*;
use video_analyzer::{DriveGaugeRead, SuperGaugeRead};

/// 残量の読み取り。
fn drive(value: f32) -> DriveGaugeRead {
    DriveGaugeRead {
        value,
        burnout: false,
        recovery: 0.0,
        uncertain: false,
    }
}

/// バーンアウト中の回復の読み取り。
fn recovering(recovery: f32) -> DriveGaugeRead {
    DriveGaugeRead {
        value: 0.0,
        burnout: true,
        recovery,
        uncertain: false,
    }
}

/// SA ゲージの読み取り。
fn super_gauge(value: f32) -> SuperGaugeRead {
    SuperGaugeRead {
        value,
        displayed_level: Some(value as u8),
        critical_art: false,
        uncertain: false,
    }
}

// ── ドライブゲージ ───────────────────────────────────────────────────────

/// 残量は割合に均す。表示は本数（満タン 6 本）だが、以降は割合で扱う。
#[test]
fn the_stock_count_is_turned_into_a_share() {
    assert!((normalized_drive(&drive(6.0)) - 1.0).abs() < 1e-6, "満タン");
    assert!((normalized_drive(&drive(3.0)) - 0.5).abs() < 1e-6, "半分");
    assert_eq!(normalized_drive(&drive(0.0)), 0.0);
}

/// バーンアウト中は残量ではなく回復の進み具合。混ぜると、回復しかけの
/// ゲージが「まだ残っている」ことになる。
#[test]
fn during_a_burnout_the_number_is_the_recovery_progress() {
    assert!((normalized_drive(&recovering(0.4)) - 0.4).abs() < 1e-6);
}

/// バーンアウト中の残量欄は見ない。残っていないのだから意味が無い。
#[test]
fn the_stock_count_is_ignored_during_a_burnout() {
    let mixed = DriveGaugeRead {
        value: 6.0,
        ..recovering(0.2)
    };

    assert!(
        (normalized_drive(&mixed) - 0.2).abs() < 1e-6,
        "バーンアウト中に残量を読んでいる"
    );
}

// ── CA が撃てるか ────────────────────────────────────────────────────────

/// 画面に CA の表示が出ていれば、それが答え。ゲージも体力も見るまでも
/// ない。
#[test]
fn a_visible_critical_art_label_settles_it() {
    let labelled = SuperGaugeRead {
        critical_art: true,
        uncertain: true,
        ..super_gauge(0.0)
    };

    assert!(ca_ready(&labelled, 1.0), "表示を無視している");
}

/// 表示が無ければ、ゲージが満タンで、体力が危険域にあることの両方が要る。
#[test]
fn without_the_label_both_a_full_gauge_and_low_health_are_needed() {
    assert!(ca_ready(&super_gauge(3.0), 0.2), "両方揃っているのに false");
    assert!(!ca_ready(&super_gauge(3.0), 0.3), "体力が高いのに true");
    assert!(
        !ca_ready(&super_gauge(2.0), 0.2),
        "ゲージが足りないのに true"
    );
}

/// ゲージの読みには少しの揺れがある。満タンの手前でも満タンとして扱う。
#[test]
fn a_gauge_just_short_of_full_still_counts_as_full() {
    assert!(
        ca_ready(&super_gauge(2.95), 0.2),
        "読みの揺れを許していない"
    );
    assert!(
        !ca_ready(&super_gauge(2.94), 0.2),
        "足りないゲージを通している"
    );
}

/// 危険域の上限もそのまま。ここを緩めると、CA の撃てない場面で
/// 撃てると判断する。
#[test]
fn the_low_health_band_has_an_exact_upper_edge() {
    assert!(
        ca_ready(&super_gauge(3.0), 0.255),
        "上限ちょうどを外している"
    );
    assert!(
        !ca_ready(&super_gauge(3.0), 0.256),
        "上限を超えて通している"
    );
}

/// 体力が読めていない場面（負の値で渡る）では撃てないことにする。
/// 読めていない体力から CA の可否を決めない。
#[test]
fn an_unreadable_health_does_not_make_it_ready() {
    assert!(!ca_ready(&super_gauge(3.0), -1.0));
}

/// ゲージの読みが怪しければ、表示が無い限り撃てないことにする。
#[test]
fn an_unreadable_gauge_does_not_make_it_ready() {
    let unsure = SuperGaugeRead {
        uncertain: true,
        ..super_gauge(3.0)
    };

    assert!(!ca_ready(&unsure, 0.2), "読めていないゲージで断定している");
}
