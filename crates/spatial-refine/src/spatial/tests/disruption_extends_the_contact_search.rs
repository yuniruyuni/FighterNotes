use super::*;

/// SA 暗転などの全画面演出中は位置を読めないのでスパークを返さない。
/// ヒントが演出に潰された場合は、演出明けの猶予フレームまで探索を
/// 延長してスパークを拾う。ヒントも猶予も無いフレームでは拾わない。
#[test]
fn disruption_extends_the_contact_search() {
    let spark_frame = |base: &Vec<u8>| {
        let mut frame = base.clone();
        rect(&mut frame, 140, 60, 16, 16, [255, 210, 40]);
        frame
    };
    // 全画面演出: プレイフィールドの過半を塗り替える。
    let flash = |brightness: u8| {
        let mut frame = blank_frame();
        rect(
            &mut frame,
            0,
            0,
            WIDTH,
            HEIGHT,
            [brightness, brightness / 2, 20],
        );
        frame
    };
    let idle = |p1_x: u32, p2_x: u32| {
        let mut frame = blank_frame();
        rect(&mut frame, p1_x, 78, 30, 72, [40, 140, 220]);
        rect(&mut frame, p2_x, 76, 28, 74, [180, 100, 45]);
        frame
    };

    let run = |grace_gap: u32| {
        let mut extractor = SpatialExtractor::new(test_config());
        extractor
            .observe_rgba(100, &idle(64, 236), WIDTH, HEIGHT, hints())
            .unwrap();
        // contact ヒントのあるフレームが演出で潰れる。
        let observed = extractor
            .observe_rgba(
                101,
                &flash(120),
                WIDTH,
                HEIGHT,
                SpatialHints {
                    contact_effect: true,
                    ..SpatialHints::default()
                },
            )
            .unwrap();
        assert!(observed.contact.is_none(), "暗転にはスパーク色が無い");
        // 演出が明けて基準フレームへ戻る(この diff も全画面 = 演出扱い)。
        extractor
            .observe_rgba(102, &idle(64, 236), WIDTH, HEIGHT, SpatialHints::default())
            .unwrap();
        // 猶予内: gap フレーム置いてからスパークが出る。
        let mut frame_index = 102;
        for _ in 0..grace_gap {
            frame_index += 1;
            extractor
                .observe_rgba(
                    frame_index,
                    &idle(64, 236),
                    WIDTH,
                    HEIGHT,
                    SpatialHints::default(),
                )
                .unwrap();
        }
        frame_index += 1;
        extractor
            .observe_rgba(
                frame_index,
                &spark_frame(&idle(64, 236)),
                WIDTH,
                HEIGHT,
                SpatialHints::default(),
            )
            .unwrap()
            .contact
    };

    // 猶予内(演出明け 12 フレーム以内)はスパークを拾う。
    assert!(run(10).is_some());
    // 猶予が尽きた後は拾わない。
    assert!(run(12).is_none());
}

/// 猶予を与える演出は playfield 面積の 3/4 から。それ未満の変化は
/// 演出ではないので、ヒントが素通りした後の発光は拾わない。
#[test]
fn disruption_area_threshold_is_three_quarters_of_the_playfield() {
    // playfield を縦 0.8 に絞る。演出閾値は「playfield 面積の 3/4」なので、
    // 全高の帯の場合は幅 0.75 が境界になる(面積 = 幅 x 0.8、閾値 0.6)。
    let spark_after_flash_width = |width_px: u32| {
        let mut extractor = SpatialExtractor::new(SpatialConfig {
            playfield: SpatialRect::new(0.0, 0.0, 1.0, 0.8),
            ..test_config()
        });
        let mut first = blank_frame();
        rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
        rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
        extractor
            .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
            .unwrap();
        // ヒントフレームがスパーク色ではない帯で覆われる。
        let mut flash = first.clone();
        rect(&mut flash, 0, 0, width_px, HEIGHT, [120, 120, 120]);
        extractor
            .observe_rgba(
                101,
                &flash,
                WIDTH,
                HEIGHT,
                SpatialHints {
                    contact_effect: true,
                    ..SpatialHints::default()
                },
            )
            .unwrap();
        // 演出が続いたまま(全画面 diff を作らずに)スパークが出る。
        let mut settled = flash.clone();
        rect(&mut settled, 140, 60, 16, 16, [255, 210, 40]);
        extractor
            .observe_rgba(102, &settled, WIDTH, HEIGHT, SpatialHints::default())
            .unwrap()
            .contact
    };
    // 幅 0.80/0.90 は演出なので猶予が付き、後続のスパークを拾う。
    assert!(spark_after_flash_width(256).is_some());
    assert!(spark_after_flash_width(288).is_some());
    // 幅 0.70 は演出ではなく、猶予は付かない。
    assert!(spark_after_flash_width(224).is_none());
}

/// 演出が持続している間もヒントの探索は生きている。全画面の diff を
/// 作らずに演出が残ったままなら、その上に出たスパークは同じフレームで
/// 拾える(grace は「ヒント中の演出」だけが与える)。
#[test]
fn a_persistent_disruption_keeps_the_hinted_search_alive() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();
    // ヒントフレームで playfield の 3/4 超を覆う演出が出る。
    let mut flash = first.clone();
    rect(&mut flash, 0, 0, 260, HEIGHT, [120, 120, 120]);
    let observed = extractor
        .observe_rgba(
            101,
            &flash,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: true,
                ..SpatialHints::default()
            },
        )
        .unwrap();
    assert!(observed.contact.is_none(), "暗転にはスパーク色が無い");
    // 演出が静止して残ったまま、その外側にスパークが出る。
    let mut settled = flash.clone();
    rect(&mut settled, 220, 60, 16, 16, [255, 210, 40]);
    let observed = extractor
        .observe_rgba(102, &settled, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(observed.contact.is_some(), "{:?}", observed.contact);
}

/// 演出に潰されなかったヒントは猶予を残さない。ヒントが切れた直後の
/// 発光は別の何かである。
#[test]
fn grace_requires_a_disrupted_hint() {
    let mut extractor = SpatialExtractor::new(test_config());
    let mut first = blank_frame();
    rect(&mut first, 64, 78, 30, 72, [40, 140, 220]);
    rect(&mut first, 236, 76, 28, 74, [180, 100, 45]);
    extractor
        .observe_rgba(100, &first, WIDTH, HEIGHT, hints())
        .unwrap();
    // 演出のない普通のヒントフレーム。
    extractor
        .observe_rgba(
            101,
            &first,
            WIDTH,
            HEIGHT,
            SpatialHints {
                contact_effect: true,
                ..SpatialHints::default()
            },
        )
        .unwrap();
    // ヒントが切れた次のフレームの発光は拾わない。
    let mut spark = first.clone();
    rect(&mut spark, 140, 60, 16, 16, [255, 210, 40]);
    let observed = extractor
        .observe_rgba(102, &spark, WIDTH, HEIGHT, SpatialHints::default())
        .unwrap();
    assert!(observed.contact.is_none(), "{:?}", observed.contact);
}
