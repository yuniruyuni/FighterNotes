export function buildHpDebugSnapshot(featuresJson: string): unknown[] {
  try {
    const features = JSON.parse(featuresJson) as Array<Record<string, unknown>>;
    const matchCount = features.filter(
      (feature) => feature.is_match_screen,
    ).length;
    const samples: unknown[] = features
      .filter((_, index) => index % 500 === 0)
      .map((feature) => ({
        fi: feature.frame_index,
        match: feature.is_match_screen,
        lscore: fixed(feature.left_hp_score),
        rscore: fixed(feature.right_hp_score),
        lraw: fixed(feature.left_hp_raw),
        rraw: fixed(feature.right_hp_raw),
        own_hp: optionalHp(feature.own_hp),
        opp_hp: optionalHp(feature.opponent_hp),
        lqual: fixed(feature.left_hp_raw_quality ?? 0, 2),
      }));
    samples.unshift({
      summary: `total=${features.length} match_frames=${matchCount}`,
    });
    return samples;
  } catch {
    return [];
  }
}

function fixed(value: unknown, digits = 3): string {
  return typeof value === "number" ? value.toFixed(digits) : "?";
}

function optionalHp(value: unknown): string {
  return typeof value === "number" && value >= 0 ? value.toFixed(3) : "?";
}
