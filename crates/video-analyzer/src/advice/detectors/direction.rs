/// 方向ラベルを表示用の矢印へ変換する。
pub(crate) fn dir_arrow(direction: &str) -> &'static str {
    match direction {
        "N" => "N",
        "U" => "↑",
        "UR" => "↗",
        "R" => "→",
        "DR" => "↘",
        "D" => "↓",
        "DL" => "↙",
        "L" => "←",
        "UL" => "↖",
        _ => "?",
    }
}
