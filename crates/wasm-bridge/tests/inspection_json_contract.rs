use serde_json::Value;
use wasm_bridge::{
    hp_parallelogram_json, inspect_drive, inspect_frame, inspect_hp, inspect_input, inspect_super,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

fn assert_object_keys(value: &Value, expected: &[&str]) {
    let mut actual = value
        .as_object()
        .expect("contract value must be an object")
        .keys()
        .map(String::as_str)
        .collect::<Vec<_>>();
    let mut expected = expected.to_vec();
    actual.sort_unstable();
    expected.sort_unstable();
    assert_eq!(actual, expected);
}

#[test]
fn inspection_exports_keep_browser_json_shapes() {
    let rgba = vec![0; WIDTH as usize * HEIGHT as usize * 4];

    let meter: Value = serde_json::from_str(&inspect_frame(&rgba, WIDTH, HEIGHT)).unwrap();
    assert_object_keys(&meter, &["left", "right"]);
    assert_object_keys(
        &meter["left"],
        &[
            "v",
            "wf",
            "states",
            "bright",
            "fresh_edge",
            "bgr",
            "stripe",
            "cols",
            "cols_w",
            "rescued",
            "quality",
            "digit_corr",
            "slab_pos",
            "slab_state",
        ],
    );

    let hp: Value = serde_json::from_str(&inspect_hp(&rgba, WIDTH, HEIGHT)).unwrap();
    assert_object_keys(
        &hp,
        &[
            "left_score",
            "right_score",
            "left_fill",
            "right_fill",
            "left_drive",
            "right_drive",
            "left_col_active",
            "right_col_active",
            "left_col_orange",
            "right_col_orange",
            "left_col_yellow",
            "right_col_yellow",
            "left_orange_fill",
            "right_orange_fill",
            "left_yellow_fill",
            "right_yellow_fill",
        ],
    );

    let drive: Value = serde_json::from_str(&inspect_drive(&rgba, WIDTH, HEIGHT)).unwrap();
    assert_object_keys(&drive, &["left", "right"]);
    assert_object_keys(
        &drive["left"],
        &[
            "value",
            "burnout",
            "recovery",
            "uncertain",
            "roi",
            "cols",
            "runs",
        ],
    );
    assert_object_keys(&drive["left"]["roi"], &["x1", "x2", "y1", "y2", "slope"]);

    let super_gauge: Value = serde_json::from_str(&inspect_super(&rgba, WIDTH, HEIGHT)).unwrap();
    assert_object_keys(&super_gauge, &["left", "right"]);
    assert_object_keys(
        &super_gauge["left"],
        &[
            "value",
            "displayed_level",
            "critical_art",
            "uncertain",
            "label_roi",
            "bar_roi",
        ],
    );
    assert_object_keys(&super_gauge["left"]["label_roi"], &["x1", "x2", "y1", "y2"]);
    assert_eq!(super_gauge["left"]["label_roi"]["x1"], 55);
    assert_eq!(super_gauge["right"]["bar_roi"]["x1"], 1510);

    let input: Value = serde_json::from_str(&inspect_input(&rgba, WIDTH, HEIGHT)).unwrap();
    assert_object_keys(&input, &["p1", "p2"]);
    assert_object_keys(&input["p1"], &["side", "rows"]);
    assert_object_keys(
        &input["p1"]["rows"][0],
        &[
            "count",
            "dir",
            "badges",
            "auto",
            "throw",
            "empty",
            "uncertain",
        ],
    );
}

#[test]
fn hp_geometry_export_keeps_side_and_point_shapes() {
    let geometry: Value = serde_json::from_str(&hp_parallelogram_json()).unwrap();
    assert_object_keys(&geometry, &["p1", "p2"]);
    assert_object_keys(
        &geometry["p1"],
        &["top_left", "top_right", "bottom_right", "bottom_left"],
    );
    assert_object_keys(&geometry["p1"]["top_left"], &["x", "y"]);
}
