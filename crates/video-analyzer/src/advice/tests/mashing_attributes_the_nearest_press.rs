use super::support::*;

#[test]
fn mashing_attributes_the_nearest_press() {
    use crate::match_events::{InputSegment, MeterState};

    let mut ev = basic_mashing_events();
    ev.segments[0] = vec![
        InputSegment {
            start_frame: 980,
            end_frame: 982,
            dir: "N".to_string(),
            badges: vec!["SP".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        },
        InputSegment {
            start_frame: 990,
            end_frame: 992,
            dir: "N".to_string(),
            badges: vec!["弱".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        },
        InputSegment {
            start_frame: 1190,
            end_frame: 1192,
            dir: "N".to_string(),
            badges: vec!["弱".to_string()],
            auto: false,
            throw: false,
            evidence: Default::default(),
        },
    ];
    let n = 6000;
    let mut own_state = vec![MeterState::Free; n];
    for state in own_state.iter_mut().take(986).skip(982) {
        *state = MeterState::ProjectileActive;
    }
    for state in own_state.iter_mut().take(998).skip(996) {
        *state = MeterState::Startup;
    }
    for state in own_state.iter_mut().take(1001).skip(998) {
        *state = MeterState::Active;
    }
    for state in own_state.iter_mut().take(1198).skip(1196) {
        *state = MeterState::Startup;
    }
    for state in own_state.iter_mut().take(1201).skip(1198) {
        *state = MeterState::Active;
    }
    ev.meter_state = [own_state, vec![MeterState::Free; n]];
    ev.meter_confidence = [vec![1.0; n], vec![1.0; n]];

    let card = detect_mashing(&[], &ev, 1, 0).expect("直近の通常技入力へ帰属する");
    assert_eq!(card.evidence[0].frame, 990);
}
