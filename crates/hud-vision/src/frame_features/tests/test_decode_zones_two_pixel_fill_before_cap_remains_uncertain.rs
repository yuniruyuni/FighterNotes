use super::support::*;

#[test]
fn test_decode_zones_two_pixel_fill_before_cap_remains_uncertain() {
    // 許容範囲を広げ過ぎると、キャラクターの赤いスプライトを cap より先に
    // 見つけたケースを満タンと誤認するため、2px は従来通り棄却する。
    use HpColColor::*;
    let zones = zones_from(&[(Fill, 2), (White, 2), (Fill, 675), (White, 2)]);
    let d = decode_hp_zones(&zones, 681);
    assert!(d.uncertain, "cap 直前でも 2px Fill は遮蔽として扱う");
}

// ── decode_drive_runs（アンカー正規化済みラン列の直接テスト） ──────────
