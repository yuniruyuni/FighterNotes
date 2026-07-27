use super::support::*;

#[test]
fn test_fill_ratio_white_flash_is_uncertain() {
    // 全白 → S≈0 → vivid 判定なし → density=0 → uncertain=true
    let mut rgba = vec![0u8; 1920 * 1080 * 4];
    for y in 64u32..95 {
        for x in 172u32..870 {
            let idx = (y as usize * 1920 + x as usize) * 4;
            rgba[idx] = 255;
            rgba[idx + 1] = 255;
            rgba[idx + 2] = 255;
            rgba[idx + 3] = 255;
        }
    }
    let (_, uncertain) = hp_fill_ratio_impl(&rgba, 1920, 1080, "p1", 0);
    assert!(uncertain, "all-white → uncertain");
}
