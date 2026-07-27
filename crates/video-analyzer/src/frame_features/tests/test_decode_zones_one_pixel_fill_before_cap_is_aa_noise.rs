use super::support::*;

#[test]
fn test_decode_zones_one_pixel_fill_before_cap_is_aa_noise() {
    // 検証済みの P2 満タンサンプル:
    // 斜めスキャン先頭の 1px だけが赤 Fill へにじみ、その直後に正常な白 cap がある。
    use HpColColor::*;
    let zones = zones_from(&[(Fill, 1), (White, 2), (Fill, 676), (White, 2)]);
    let d = decode_hp_zones(&zones, 681);
    assert!(
        !d.uncertain,
        "cap 直前の 1px Fill は AA ノイズとして許容する"
    );
    assert!(
        (d.fill_ratio - 1.0).abs() < 1e-6,
        "満タンとして読む: {}",
        d.fill_ratio
    );
}
