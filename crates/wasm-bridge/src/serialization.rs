pub(crate) fn hp_or_unknown(raw: f32, uncertain: bool) -> f32 {
    if uncertain {
        -1.0
    } else {
        raw
    }
}

pub(crate) fn tracked_to_json(tracked: &[video_analyzer::TrackedInput]) -> String {
    tracked
        .iter()
        .map(|input| {
            format!(
                r#"{{"count":{},"dir":"{}","badges":"{}","auto":{},"throw":{},"repaired":{},"uncertain":{}}}"#,
                input.count.map_or("null".to_string(), |count| count.to_string()),
                input.dir.as_str(),
                input
                    .badges
                    .iter()
                    .map(|badge| badge.label())
                    .collect::<Vec<_>>()
                    .join(" "),
                input.auto,
                input.throw,
                input.repaired,
                input.uncertain,
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests;
