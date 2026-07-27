use crate::frame_data;

fn options(character: Option<&str>, advantage: u32) -> Vec<String> {
    character
        .map(|name| frame_data::punish_options(name, advantage, 3))
        .unwrap_or_default()
        .iter()
        .map(|move_data| {
            format!(
                "{}（発生{}F/威力{}）",
                move_data.name, move_data.startup, move_data.damage
            )
        })
        .collect()
}

pub(super) fn missed_option_text(character: Option<&str>, advantage: u32) -> String {
    let options = options(character, advantage);
    if options.is_empty() {
        "この近距離で、発生が有利フレーム以下の技を反撃候補にします。まずは発生の速い技で確実に取る癖をつけましょう。".to_string()
    } else {
        format!(
            "位置解析で重なりを確認したこの場面では、有利 {advantage}F なら {} などが反撃候補です。",
            options.join("、")
        )
    }
}

pub(super) fn failed_option_text(character: Option<&str>, advantage: u32) -> String {
    let options = options(character, advantage);
    if options.is_empty() {
        "この有利フレーム内に発生し、実戦と同じ距離に届く技があるかをトレモで確認しましょう。"
            .to_string()
    } else {
        format!(
            "有利 {advantage}F なら、発生上は {} などが候補です。実際に確定するかは、この距離でのリーチをトレモで確認しましょう。",
            options.join("、")
        )
    }
}
