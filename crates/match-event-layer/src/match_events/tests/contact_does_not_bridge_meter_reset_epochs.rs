use super::support::*;

#[test]
fn contact_does_not_bridge_meter_reset_epochs() {
    let left = synth_segmented_timeline(3, vec![(100, "active", 100, 108)]);
    let right = synth_segmented_timeline(4, vec![(200, "stun", 100, 108)]);
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: 200,
        winner: None,
        p1_hp_end: 1.0,
        p2_hp_end: 1.0,
    }];
    assert!(
        super::contacts::extract_contacts(&left, &right, &[], &rounds).is_empty(),
        "異なるメーター区間の停止セルを接触へ結び付けない"
    );
}
