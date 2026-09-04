use super::super::super::{ActorObservation, SpatialConfig, SpatialPoint, SpatialRect};
use super::super::grid::{CellColor, CellGrid};
use super::super::motion::MotionRegion;

#[derive(Clone, Debug)]
pub(super) struct ActorTrack {
    pub(super) anchor: SpatialPoint,
    pub(super) bounds: SpatialRect,
    pub(super) confidence: f32,
    pub(super) last_observed_frame: u32,
    /// Cell snapshot of the last observed bounds, used to confirm that an
    /// actor producing no motion is standing still rather than lost.
    pub(super) appearance: Option<Appearance>,
}

#[derive(Clone, Debug)]
pub(super) struct Appearance {
    cell_left: usize,
    cell_top: usize,
    cell_width: usize,
    cell_height: usize,
    cells: Vec<CellColor>,
}

impl Appearance {
    fn capture(grid: &CellGrid, bounds: SpatialRect) -> Self {
        let cell_left = (bounds.left * grid.width as f32).floor() as usize;
        let cell_top = (bounds.top * grid.height as f32).floor() as usize;
        let cell_right = ((bounds.right * grid.width as f32).ceil() as usize).min(grid.width);
        let cell_bottom = ((bounds.bottom * grid.height as f32).ceil() as usize).min(grid.height);
        let cell_width = cell_right.saturating_sub(cell_left);
        let cell_height = cell_bottom.saturating_sub(cell_top);
        // 退化した範囲は cells が空になり、still_matches が常に false を
        // 返すことで自然に「確認できない」へ落ちる。
        let cells = (cell_top..cell_bottom)
            .flat_map(|y| (cell_left..cell_right).map(move |x| grid.cells[y * grid.width + x]))
            .collect();
        Self {
            cell_left,
            cell_top,
            cell_width,
            cell_height,
            cells,
        }
    }

    /// True when the same cells still show the same colors. VFX overlapping
    /// the patch or a vacated position both fail the match, which correctly
    /// falls back to the decaying carry-forward.
    fn still_matches(&self, grid: &CellGrid, config: &SpatialConfig) -> bool {
        // 空の記憶は下の分数が 0/0 になり自然に不成立へ落ちる。グリッドが
        // 記憶時より縮んだ場合だけ、参照前に諦める。
        if self.cell_left + self.cell_width > grid.width
            || self.cell_top + self.cell_height > grid.height
        {
            return false;
        }
        let mut unchanged = 0usize;
        for y in 0..self.cell_height {
            for x in 0..self.cell_width {
                let stored = self.cells[y * self.cell_width + x];
                let current = grid.cells[(self.cell_top + y) * grid.width + self.cell_left + x];
                let delta = stored
                    .r
                    .abs_diff(current.r)
                    .max(stored.g.abs_diff(current.g))
                    .max(stored.b.abs_diff(current.b));
                if delta < config.motion_threshold {
                    unchanged += 1;
                }
            }
        }
        unchanged as f32 / self.cells.len() as f32 >= config.still_match_min_fraction
    }
}

pub(super) fn apply_anchor_hint(
    track: &mut Option<ActorTrack>,
    hint: Option<SpatialPoint>,
    frame_index: u32,
) {
    let Some(anchor) = hint else {
        return;
    };
    let anchor = SpatialPoint::new(anchor.x.clamp(0.0, 1.0), anchor.y.clamp(0.0, 1.0));
    match track {
        Some(track) => {
            track.anchor = anchor;
            track.bounds = SpatialRect::new(anchor.x, anchor.y, anchor.x, anchor.y);
            track.confidence = track.confidence.max(0.80);
            track.last_observed_frame = frame_index;
            // 位置が外部から書き換えられた以上、旧位置の画は静止確認に使えない。
            track.appearance = None;
        }
        None => {
            *track = Some(ActorTrack {
                anchor,
                bounds: SpatialRect::new(anchor.x, anchor.y, anchor.x, anchor.y),
                confidence: 0.80,
                last_observed_frame: frame_index,
                appearance: None,
            });
        }
    }
}

pub(super) fn update(
    track: &mut Option<ActorTrack>,
    region: Option<&MotionRegion>,
    merged: bool,
    grid: &CellGrid,
    allow_discontinuity: bool,
    frame_index: u32,
    config: &SpatialConfig,
) -> Option<ActorObservation> {
    if let Some(region) = region {
        // 両者の体が 1 つの blob へ合体した領域の中心は 2 人の中間で
        // あって、どちらの足元でもない。x は自分の追跡値を保ち、領域は
        // 存在と縦方向の証拠としてだけ使う。合体領域の画は個人の静止
        // 確認に使えないため appearance も更新しない。
        if merged {
            if let Some(existing) = track.as_mut() {
                let anchor = SpatialPoint::new(existing.anchor.x, region.anchor().y);
                existing.anchor = anchor;
                existing.bounds = region.bounds;
                existing.confidence = config.merged_region_confidence;
                existing.last_observed_frame = frame_index;
                return Some(ActorObservation {
                    anchor,
                    bounds: region.bounds,
                    confidence: config.merged_region_confidence,
                    observed: true,
                    ground_anchor: anchor.y >= config.actor_ground_y,
                    discontinuity: false,
                });
            }
        }
        let anchor = region.anchor();
        let discontinuity = track.as_ref().is_some_and(|old| {
            allow_discontinuity && (old.anchor.x - anchor.x).abs() > config.max_track_dx
        });
        let confidence = if discontinuity { 0.58 } else { 0.72 };
        let mut fresh = from_region(region, frame_index, confidence);
        fresh.appearance = Some(Appearance::capture(grid, region.bounds));
        *track = Some(fresh);
        return Some(ActorObservation {
            anchor,
            bounds: region.bounds,
            confidence,
            observed: true,
            ground_anchor: anchor.y >= config.actor_ground_y,
            discontinuity,
        });
    }

    let track = track.as_mut()?;
    // モーションが無い場合でも、前回観測時と同じ画が同じ場所に残って
    // いれば、それは喪失ではなく静止の確認である。ガード硬直・ダウン・
    // hitstop の間の減衰喪失を防ぐ。
    if track
        .appearance
        .as_ref()
        .is_some_and(|appearance| appearance.still_matches(grid, config))
    {
        track.last_observed_frame = frame_index;
        return Some(ActorObservation {
            anchor: track.anchor,
            bounds: track.bounds,
            confidence: config.still_confidence,
            observed: false,
            ground_anchor: track.anchor.y >= config.actor_ground_y,
            discontinuity: false,
        });
    }
    let stale = frame_index.saturating_sub(track.last_observed_frame);
    if stale > config.max_stale_frames {
        return None;
    }
    let confidence = track.confidence * 0.92f32.powi(stale as i32);
    Some(ActorObservation {
        anchor: track.anchor,
        bounds: track.bounds,
        confidence,
        observed: false,
        ground_anchor: track.anchor.y >= config.actor_ground_y,
        discontinuity: false,
    })
}

pub(super) fn from_region(region: &MotionRegion, frame_index: u32, confidence: f32) -> ActorTrack {
    ActorTrack {
        anchor: region.anchor(),
        bounds: region.bounds,
        confidence,
        last_observed_frame: frame_index,
        appearance: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_grid() -> CellGrid {
        CellGrid {
            cells: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// セル値 = y*width + x の判別可能なグリッド。
    fn numbered_grid(width: usize, height: usize) -> CellGrid {
        CellGrid {
            cells: (0..width * height)
                .map(|index| CellColor {
                    r: index as u8,
                    g: 0,
                    b: 0,
                })
                .collect(),
            width,
            height,
        }
    }

    #[test]
    fn capture_reads_the_exact_cell_window() {
        let grid = numbered_grid(10, 10);
        let bounds = SpatialRect::new(0.25, 0.35, 0.65, 0.75);
        let appearance = Appearance::capture(&grid, bounds);
        assert_eq!(appearance.cell_left, 2);
        assert_eq!(appearance.cell_top, 3);
        assert_eq!(appearance.cell_width, 5);
        assert_eq!(appearance.cell_height, 5);
        let expected: Vec<u8> = (3..8)
            .flat_map(|y| (2..7).map(move |x| (y * 10 + x) as u8))
            .collect();
        let actual: Vec<u8> = appearance.cells.iter().map(|cell| cell.r).collect();
        assert_eq!(actual, expected);

        // 右下の境界はグリッド幅・高さで切り詰める。
        let clamped = Appearance::capture(&grid, SpatialRect::new(0.85, 0.85, 1.3, 1.3));
        assert_eq!(clamped.cell_left, 8);
        assert_eq!(clamped.cell_width, 2);
        assert_eq!(clamped.cell_height, 2);
    }

    #[test]
    fn still_match_counts_unchanged_cells_against_the_fraction() {
        let config = SpatialConfig {
            motion_threshold: 18,
            still_match_min_fraction: 0.90,
            ..SpatialConfig::default()
        };
        let grid = numbered_grid(10, 10);
        // 行 0 の 10 セルを覚える。
        let appearance = Appearance::capture(&grid, SpatialRect::new(0.0, 0.0, 1.0, 0.1));
        assert!(appearance.still_matches(&grid, &config));

        // 1 セルだけ閾値未満 (17) 変わっても、変化とは数えない。
        let mut subtle = numbered_grid(10, 10);
        subtle.cells[3].r += 17;
        assert!(appearance.still_matches(&subtle, &config));

        // 1 セルが閾値ちょうど (18) 変わると 9/10 = 0.90 で、まだ一致。
        let mut one_changed = numbered_grid(10, 10);
        one_changed.cells[3].r += 18;
        assert!(appearance.still_matches(&one_changed, &config));

        // 2 セル変わると 8/10 = 0.80 < 0.90 で不一致。
        let mut two_changed = numbered_grid(10, 10);
        two_changed.cells[3].r += 18;
        two_changed.cells[7].r += 18;
        assert!(!appearance.still_matches(&two_changed, &config));
    }

    #[test]
    fn degenerate_or_shrunk_grids_never_confirm_stillness() {
        let config = SpatialConfig::default();
        let grid = numbered_grid(10, 10);
        // 幅 0 の範囲はセルを持たず、静止を確認できない。
        let empty = Appearance::capture(&grid, SpatialRect::new(0.5, 0.2, 0.5, 0.8));
        assert!(!empty.still_matches(&grid, &config));
        // 幅か高さの片方だけが縮んでも確認しない。
        let appearance = Appearance::capture(&grid, SpatialRect::new(0.5, 0.5, 1.0, 1.0));
        assert!(!appearance.still_matches(&numbered_grid(5, 20), &config));
        assert!(!appearance.still_matches(&numbered_grid(20, 5), &config));
        // グリッドの右下端に接しているだけなら縮小ではない。
        assert!(appearance.still_matches(&grid, &config));
        // 和が幅に収まっていれば、積が幅を超えても縮小ではない
        // (left 3 + width 4 = 7 <= 10)。
        let offset = Appearance::capture(&grid, SpatialRect::new(0.3, 0.3, 0.7, 0.7));
        assert!(offset.still_matches(&grid, &config));
    }

    /// 記憶した窓と現在のグリッドの対応は、セル 1 つ単位で正確でなければ
    /// ならない。窓の外がいくら変わっても静止は成立し、窓の中が 1 セルでも
    /// 変われば(全一致を要求する設定では)成立しない。
    #[test]
    fn still_match_correspondence_is_cell_exact() {
        let config = SpatialConfig {
            still_match_min_fraction: 1.0,
            ..SpatialConfig::default()
        };
        let grid = numbered_grid(10, 10);
        let window = SpatialRect::new(0.2, 0.3, 0.7, 0.7);
        let appearance = Appearance::capture(&grid, window);

        // 窓 (x 2..7, y 3..7) の外を全部壊しても一致する。
        let mut outside_wrecked = numbered_grid(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                if !(2..7).contains(&x) || !(3..7).contains(&y) {
                    outside_wrecked.cells[y * 10 + x].r ^= 0x80;
                }
            }
        }
        assert!(appearance.still_matches(&outside_wrecked, &config));

        // 窓の中の 1 セルが変わると全一致では通らない。
        let mut inside_changed = numbered_grid(10, 10);
        inside_changed.cells[5 * 10 + 4].r ^= 0x80;
        assert!(!appearance.still_matches(&inside_changed, &config));

        // 変化は G・B チャネル単独でも数える。
        let mut green_changed = numbered_grid(10, 10);
        green_changed.cells[5 * 10 + 4].g = 200;
        assert!(!appearance.still_matches(&green_changed, &config));
        let mut blue_changed = numbered_grid(10, 10);
        blue_changed.cells[5 * 10 + 4].b = 200;
        assert!(!appearance.still_matches(&blue_changed, &config));
    }

    /// 静止確認の接地判定は基準ちょうどを含む。
    #[test]
    fn grounded_still_confirmation_is_inclusive_at_the_boundary() {
        let config = SpatialConfig::default();
        let grid = numbered_grid(10, 10);
        let at_ground = MotionRegion {
            bounds: SpatialRect::new(0.2, 0.3, 0.5, config.actor_ground_y),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [0, 0, 0],
        };
        let mut track = None;
        update(
            &mut track,
            Some(&at_ground),
            false,
            &grid,
            false,
            10,
            &config,
        )
        .unwrap();
        let observed = update(&mut track, None, false, &grid, false, 11, &config).unwrap();
        assert_eq!(observed.confidence, config.still_confidence);
        assert!(observed.ground_anchor);
    }

    #[test]
    fn merged_region_update_keeps_x_and_reports_merged_evidence() {
        let config = SpatialConfig::default();
        let mut track = Some(from_region(&region(0.3, 0.9), 5, 0.72));
        let merged = MotionRegion {
            bounds: SpatialRect::new(0.2, 0.4, 0.6, 0.68),
            changed_cells: 200,
            energy: 2_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [0, 0, 0],
        };

        let observed = update(
            &mut track,
            Some(&merged),
            true,
            &empty_grid(),
            false,
            9,
            &config,
        )
        .unwrap();
        assert_eq!(observed.anchor, SpatialPoint::new(0.3, 0.68));
        assert_eq!(observed.bounds, merged.bounds);
        assert_eq!(observed.confidence, config.merged_region_confidence);
        assert!(observed.observed);
        assert!(!observed.ground_anchor, "0.68 は接地帯 0.70 より上");
        assert!(!observed.discontinuity);
        let stored = track.as_ref().unwrap();
        assert_eq!(stored.anchor.x, 0.3);
        assert_eq!(stored.last_observed_frame, 9);
        assert_eq!(stored.confidence, config.merged_region_confidence);

        // 接地帯ちょうどに届く合体領域は接地とみなす。
        let grounded = MotionRegion {
            bounds: SpatialRect::new(0.2, 0.4, 0.6, config.actor_ground_y),
            ..merged.clone()
        };
        let observed = update(
            &mut track,
            Some(&grounded),
            true,
            &empty_grid(),
            false,
            10,
            &config,
        )
        .unwrap();
        assert!(observed.ground_anchor);

        // トラックが無ければ合体扱いにできず、通常の観測として始まる。
        let mut vacant: Option<ActorTrack> = None;
        let observed = update(
            &mut vacant,
            Some(&merged),
            true,
            &empty_grid(),
            false,
            11,
            &config,
        )
        .unwrap();
        assert_eq!(observed.anchor, merged.anchor());
    }

    #[test]
    fn still_confirmation_reports_calibrated_evidence_and_resets_staleness() {
        let config = SpatialConfig::default();
        let grid = numbered_grid(10, 10);
        let body = MotionRegion {
            bounds: SpatialRect::new(0.2, 0.2, 0.5, 0.9),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [0, 0, 0],
        };
        let mut track = None;
        update(&mut track, Some(&body), false, &grid, false, 10, &config).unwrap();

        // max_stale_frames を超えた後でも、同じ画が残っていれば静止確認。
        let frame = 10 + config.max_stale_frames + 10;
        let observed = update(&mut track, None, false, &grid, false, frame, &config).unwrap();
        assert!(!observed.observed);
        assert_eq!(observed.confidence, config.still_confidence);
        assert_eq!(observed.anchor, body.anchor());
        assert!(observed.ground_anchor);
        assert!(!observed.discontinuity);

        // 静止確認は staleness を巻き戻す。次のフレームで画が壊れても、
        // 減衰は直前の確認フレームから 1 フレームぶんだけ進む。
        let mut wrecked = numbered_grid(10, 10);
        for cell in &mut wrecked.cells {
            cell.r = cell.r.wrapping_add(120);
        }
        let decayed = update(&mut track, None, false, &wrecked, false, frame + 1, &config).unwrap();
        assert!(!decayed.observed);
        assert!(
            (decayed.confidence - 0.72 * 0.92).abs() < 1e-6,
            "{decayed:?}"
        );
    }

    #[test]
    fn airborne_still_confirmation_is_not_grounded() {
        let config = SpatialConfig::default();
        let grid = numbered_grid(10, 10);
        let airborne = MotionRegion {
            bounds: SpatialRect::new(0.2, 0.1, 0.5, 0.5),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [0, 0, 0],
        };
        let mut track = None;
        update(
            &mut track,
            Some(&airborne),
            false,
            &grid,
            false,
            10,
            &config,
        )
        .unwrap();
        let observed = update(&mut track, None, false, &grid, false, 11, &config).unwrap();
        assert_eq!(observed.confidence, config.still_confidence);
        assert!(!observed.ground_anchor);
    }

    fn region(x: f32, y: f32) -> MotionRegion {
        MotionRegion {
            bounds: SpatialRect::new(x - 0.05, y - 0.2, x + 0.05, y),
            changed_cells: 100,
            energy: 1_000,
            effect_cells: 0,
            effect_x_sum: 0.0,
            effect_y_sum: 0.0,
            effect_xx_sum: 0.0,
            effect_yy_sum: 0.0,
            cold_effect_cells: 0,
            cold_x_sum: 0.0,
            cold_y_sum: 0.0,
            cold_xx_sum: 0.0,
            cold_yy_sum: 0.0,
            color_sum: [0, 0, 0],
        }
    }

    #[test]
    fn anchor_hints_clamp_coordinates_and_update_every_track_field() {
        let mut track = None;
        apply_anchor_hint(&mut track, Some(SpatialPoint::new(-0.2, 1.4)), 37);
        let created = track.as_ref().unwrap();
        assert_eq!(created.anchor, SpatialPoint::new(0.0, 1.0));
        assert_eq!(created.bounds, SpatialRect::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(created.confidence, 0.80);
        assert_eq!(created.last_observed_frame, 37);

        track.as_mut().unwrap().confidence = 0.91;
        apply_anchor_hint(&mut track, Some(SpatialPoint::new(0.4, 0.6)), 42);
        let updated = track.as_ref().unwrap();
        assert_eq!(updated.anchor, SpatialPoint::new(0.4, 0.6));
        assert_eq!(updated.bounds, SpatialRect::new(0.4, 0.6, 0.4, 0.6));
        assert_eq!(updated.confidence, 0.91);
        assert_eq!(updated.last_observed_frame, 42);

        let snapshot = track.clone();
        apply_anchor_hint(&mut track, None, 99);
        assert_eq!(
            track.as_ref().unwrap().last_observed_frame,
            snapshot.unwrap().last_observed_frame
        );
    }

    #[test]
    fn region_updates_report_discontinuity_observation_and_ground_boundary() {
        let config = SpatialConfig::default();
        let mut track = Some(from_region(&region(0.1, config.actor_ground_y), 5, 0.72));
        let moved = region(0.1 + config.max_track_dx + 0.01, config.actor_ground_y);

        let observed = update(
            &mut track,
            Some(&moved),
            false,
            &empty_grid(),
            true,
            17,
            &config,
        )
        .unwrap();

        assert_eq!(observed.anchor, moved.anchor());
        assert_eq!(observed.bounds, moved.bounds);
        assert_eq!(observed.confidence, 0.58);
        assert!(observed.observed);
        assert!(observed.ground_anchor);
        assert!(observed.discontinuity);
        let stored = track.unwrap();
        assert_eq!(stored.last_observed_frame, 17);
        assert_eq!(stored.confidence, 0.58);
    }

    #[test]
    fn stale_tracks_decay_through_the_last_allowed_frame() {
        let config = SpatialConfig {
            max_stale_frames: 2,
            ..SpatialConfig::default()
        };
        let mut track = Some(from_region(&region(0.3, 0.6), 10, 0.8));

        let stale = update(&mut track, None, false, &empty_grid(), false, 12, &config).unwrap();
        assert!((stale.confidence - 0.8 * 0.92f32.powi(2)).abs() < 1e-6);
        assert!(!stale.observed);
        assert!(!stale.ground_anchor);
        assert!(!stale.discontinuity);
        assert!(update(&mut track, None, false, &empty_grid(), false, 13, &config).is_none());

        let stored = from_region(&region(0.2, 0.9), 23, 0.67);
        assert_eq!(stored.last_observed_frame, 23);
        assert_eq!(stored.confidence, 0.67);

        let mut grounded = Some(from_region(&region(0.2, config.actor_ground_y), 20, 0.7));
        assert!(
            update(
                &mut grounded,
                None,
                false,
                &empty_grid(),
                false,
                20,
                &config
            )
            .unwrap()
            .ground_anchor
        );
    }
}
