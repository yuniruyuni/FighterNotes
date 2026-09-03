use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpatialPoint {
    pub x: f32,
    pub y: f32,
}

impl SpatialPoint {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct SpatialRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

impl SpatialRect {
    pub const fn new(left: f32, top: f32, right: f32, bottom: f32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn width(self) -> f32 {
        (self.right - self.left).max(0.0)
    }

    pub fn height(self) -> f32 {
        (self.bottom - self.top).max(0.0)
    }

    pub fn center(self) -> SpatialPoint {
        SpatialPoint::new(
            (self.left + self.right) * 0.5,
            (self.top + self.bottom) * 0.5,
        )
    }

    pub(in crate::spatial) fn contains(self, point: SpatialPoint) -> bool {
        point.x >= self.left && point.x < self.right && point.y >= self.top && point.y < self.bottom
    }

    pub(in crate::spatial) fn union(self, other: Self) -> Self {
        Self::new(
            self.left.min(other.left),
            self.top.min(other.top),
            self.right.max(other.right),
            self.bottom.max(other.bottom),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DistanceBand {
    Overlap,
    Close,
    Mid,
    Far,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalOrder {
    P1Left,
    P1Right,
    Overlapping,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorizontalMotion {
    Left,
    Right,
    Stationary,
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 退化した矩形の幅・高さは負にせず 0 に切り上げる。
    #[test]
    fn inverted_rect_extents_clamp_to_zero() {
        let inverted = SpatialRect::new(0.6, 0.7, 0.4, 0.5);
        assert_eq!(inverted.width(), 0.0);
        assert_eq!(inverted.height(), 0.0);
        let normal = SpatialRect::new(0.2, 0.1, 0.6, 0.4);
        assert!((normal.width() - 0.4).abs() < 1e-6);
        assert!((normal.height() - 0.3).abs() < 1e-6);
    }
}
