use crate::test_support::*;

#[test]
fn later_damage_in_the_same_guard_release_is_not_a_second_break() {
    use MeterState::*;

    let n = 200usize;
    let mut own_meter = vec![Free; n];
    own_meter[70..150].fill(Stun);
    let opponent_meter = vec![Free; n];

    let mut own_hp = vec![1.0; n];
    own_hp[100..].fill(0.9706);
    own_hp[129..].fill(0.8502);
    let opponent_hp = vec![1.0; n];

    // 実動画 yuniruyuni-001 と同じく、初段から29 video frames 後に
    // 後続ダメージがある。一度ガードを外した後は同じ方向を保持する。
    let own_inputs: Vec<_> = (0..n)
        .map(|frame| {
            let (count, direction) = if frame < 100 {
                (frame as u32 + 1, InputDir::DownRight)
            } else {
                (frame as u32 - 99, InputDir::UpRight)
            };
            tracked(count, direction, vec![], false, false)
        })
        .collect();
    let damage = vec![
        DamageEvent {
            victim: 1,
            start_frame: 100,
            pre_freeze_frame: 100,
            end_frame: 104,
            hp_before: 1.0,
            hp_after: 0.9706,
            drop: 0.0294,
            round_no: 1,
        },
        DamageEvent {
            victim: 1,
            start_frame: 129,
            pre_freeze_frame: 129,
            end_frame: 140,
            hp_before: 0.9706,
            hp_after: 0.8502,
            drop: 0.1204,
            round_no: 1,
        },
    ];
    let rounds = vec![RoundInfo {
        round_no: 1,
        start_frame: 0,
        end_frame: n as u32 - 1,
        winner: None,
        p1_hp_end: 0.8502,
        p2_hp_end: 1.0,
    }];

    let breaks = crate::guard_breaks::extract_guard_breaks(
        &damage,
        &[own_meter, opponent_meter],
        &[own_hp, opponent_hp],
        [&own_inputs, &[]],
        &[],
        &[],
        &[],
        &rounds,
    );

    assert_eq!(
        breaks.len(),
        1,
        "1回のガード解除に続くダメージ片を別判断として数えない: {breaks:?}"
    );
    assert_eq!(breaks[0].frame, 100);
    assert_eq!(
        (breaks[0].guard_dir.as_str(), breaks[0].broke_to.as_str()),
        ("DR", "UR")
    );
}
