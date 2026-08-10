//! ドライブゲージ ROI の読み取り範囲に対するテスト。

use super::*;

/// 1080p では、ROI の幅からリム装飾の分だけ短くなる。
#[test]
fn the_reference_width_loses_exactly_the_rim() {
    assert_eq!(
        cells_span(ROI_REFERENCE_WIDTH),
        ROI_REFERENCE_WIDTH - RIM_WIDTH
    );
}

/// 解像度が変われば比例して伸び縮みする。固定幅で切ると、低解像度で
/// 実体まで削り、高解像度でリムを残す。
#[test]
fn the_span_scales_with_the_roi() {
    assert_eq!(
        cells_span(ROI_REFERENCE_WIDTH * 2),
        (ROI_REFERENCE_WIDTH - RIM_WIDTH) * 2
    );
    assert_eq!(
        cells_span(ROI_REFERENCE_WIDTH / 2),
        (ROI_REFERENCE_WIDTH - RIM_WIDTH) / 2
    );
}

/// リムは必ず削る。削らないと、満タン時のグローを残量に数える。
#[test]
fn the_span_is_always_shorter_than_the_roi() {
    for roi_w in [50usize, 162, 324, 648, 1000] {
        assert!(
            cells_span(roi_w) < roi_w,
            "{roi_w} 列の ROI でリムを削っていない"
        );
    }
}

/// 潰れた ROI でも 1 列は残す。空の列並びからは、決められない状態と
/// 空のゲージを区別できない。
#[test]
fn a_degenerate_roi_keeps_one_column() {
    assert_eq!(cells_span(0), 1);
    assert_eq!(cells_span(1), 1);
}
