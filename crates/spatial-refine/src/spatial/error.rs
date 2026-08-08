use std::error::Error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpatialError {
    InvalidDimensions,
    BufferTooSmall {
        expected: usize,
        actual: usize,
    },
    DimensionsChanged {
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },
}

impl fmt::Display for SpatialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => write!(f, "frame dimensions must be non-zero"),
            Self::BufferTooSmall { expected, actual } => {
                write!(f, "RGBA buffer is too small: expected {expected}, got {actual}")
            }
            Self::DimensionsChanged {
                expected_width,
                expected_height,
                actual_width,
                actual_height,
            } => write!(
                f,
                "frame dimensions changed from {expected_width}x{expected_height} to {actual_width}x{actual_height}"
            ),
        }
    }
}

impl Error for SpatialError {}
