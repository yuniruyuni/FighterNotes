use super::*;

/// デバッグ JSON（examples / viewer 用）。
pub fn input_history_debug_json(rgba: &[u8], width: u32, height: u32, side: &str) -> String {
    let rows = read_input_rows(rgba, width, height, side);
    let rows_json: Vec<String> = rows.iter().map(|r| {
        format!(
            r#"{{"count":{},"dir":"{}","badges":"{}","auto":{},"throw":{},"empty":{},"uncertain":{}}}"#,
            r.count.map_or("null".to_string(), |c| c.to_string()),
            r.dir.as_str(),
            r.badges.iter().map(|b| b.label()).collect::<Vec<_>>().join(" "),
            r.auto,
            r.throw,
            r.empty,
            r.uncertain,
        )
    }).collect();
    format!(r#"{{"side":"{}","rows":[{}]}}"#, side, rows_json.join(","))
}
