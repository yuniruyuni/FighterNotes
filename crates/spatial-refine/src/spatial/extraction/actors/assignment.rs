use super::super::super::SpatialConfig;
use super::super::motion::MotionRegion;
use super::track::{from_region, ActorTrack};

pub(super) fn initial_tracks(
    regions: &[MotionRegion],
    candidates: &[usize],
    frame_index: u32,
    allow_airborne: [bool; 2],
    config: &SpatialConfig,
) -> Option<[ActorTrack; 2]> {
    let mut ranked = candidates.to_vec();
    ranked.sort_by(|&a, &b| {
        regions[b]
            .changed_cells
            .cmp(&regions[a].changed_cells)
            .then_with(|| regions[b].energy.cmp(&regions[a].energy))
    });
    let first = ranked[0];
    let second = ranked
        .iter()
        .copied()
        .skip(1)
        .find(|&index| (regions[index].anchor().x - regions[first].anchor().x).abs() >= 0.12)?;
    // 短いイベント窓は両者が既に位置を入れ替えた状態から始まることがある。
    // 片側だけにジャンプヒントがあり、候補も空中/地上に分かれるなら、
    // 左=P1 の初期仮定より意味ヒントを優先してプレイヤーIDを割り当てる。
    if allow_airborne[0] != allow_airborne[1] {
        let first_airborne = regions[first].anchor().y < config.actor_ground_y;
        let second_airborne = regions[second].anchor().y < config.actor_ground_y;
        if first_airborne != second_airborne {
            let (airborne, grounded) = if first_airborne {
                (first, second)
            } else {
                (second, first)
            };
            let assigned = if allow_airborne[0] {
                [airborne, grounded]
            } else {
                [grounded, airborne]
            };
            return Some([
                from_region(&regions[assigned[0]], frame_index, 0.55),
                from_region(&regions[assigned[1]], frame_index, 0.55),
            ]);
        }
    }
    let (left, right) = match regions[first]
        .anchor()
        .x
        .total_cmp(&regions[second].anchor().x)
    {
        std::cmp::Ordering::Less | std::cmp::Ordering::Equal => (first, second),
        std::cmp::Ordering::Greater => (second, first),
    };
    Some([
        from_region(&regions[left], frame_index, 0.55),
        from_region(&regions[right], frame_index, 0.55),
    ])
}

pub(super) fn assign_regions(
    tracks: [Option<&ActorTrack>; 2],
    allow_discontinuity: [bool; 2],
    allow_airborne: [bool; 2],
    regions: &[MotionRegion],
    candidates: &[usize],
    config: &SpatialConfig,
) -> [Option<usize>; 2] {
    let mut scored: [Vec<(usize, f32)>; 2] = [Vec::new(), Vec::new()];
    for side in 0..2 {
        if let Some(track) = tracks[side] {
            for &index in candidates {
                let anchor = regions[index].anchor();
                let dx = (anchor.x - track.anchor.x).abs();
                let dy = (anchor.y - track.anchor.y).abs();
                let leaves_ground =
                    track.anchor.y >= config.actor_ground_y && anchor.y < config.actor_ground_y;
                let airborne_allowed =
                    !leaves_ground || allow_airborne[side] || allow_discontinuity[side];
                let within_track_range = allow_discontinuity[side]
                    || (dx <= config.max_track_dx && dy <= config.max_track_dy);
                if airborne_allowed && within_track_range {
                    scored[side].push((index, region_score(dx, dy, &regions[index], config)));
                }
            }
            scored[side].sort_by(|a, b| a.1.total_cmp(&b.1));
        }
    }
    best_pair(&scored, allow_discontinuity)
}

fn region_score(dx: f32, dy: f32, region: &MotionRegion, config: &SpatialConfig) -> f32 {
    let discontinuity_penalty = if dx > config.max_track_dx { 0.18 } else { 0.0 };
    let size_bonus = (region.changed_cells as f32 / 200.0).min(0.12);
    let ground_bonus = if region.bounds.bottom >= 0.86 {
        0.18
    } else {
        0.0
    };
    dx * 1.8 + dy * 0.35 + discontinuity_penalty - size_bonus - ground_bonus
}

fn best_pair(
    scored: &[Vec<(usize, f32)>; 2],
    allow_discontinuity: [bool; 2],
) -> [Option<usize>; 2] {
    let mut best = [None, None];
    let mut best_score = f32::INFINITY;
    let p1_options = std::iter::once(None).chain(scored[0].iter().copied().map(Some));
    for p1_option in p1_options {
        let p2_options = std::iter::once(None).chain(scored[1].iter().copied().map(Some));
        for p2_option in p2_options {
            let duplicates =
                matches!((p1_option, p2_option), (Some((a, _)), Some((b, _))) if a == b);
            if !duplicates {
                let p1_missing_penalty = if allow_discontinuity[0] { 2.0 } else { 0.55 };
                let p2_missing_penalty = if allow_discontinuity[1] { 2.0 } else { 0.55 };
                let score = p1_option.map_or(p1_missing_penalty, |(_, score)| score)
                    + p2_option.map_or(p2_missing_penalty, |(_, score)| score);
                if score.total_cmp(&best_score) == std::cmp::Ordering::Less {
                    best_score = score;
                    best = [
                        p1_option.map(|(index, _)| index),
                        p2_option.map(|(index, _)| index),
                    ];
                }
            }
        }
    }
    best
}

#[cfg(test)]
mod tests;
