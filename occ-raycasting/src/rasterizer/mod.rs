mod frame;
mod rasterizer;
mod rasterizer_culler;

pub use frame::*;
pub use rasterizer_culler::*;

use std::fmt::Debug;

pub trait DepthBufferPrecisionType:
    Clone + Copy + PartialEq + PartialOrd + Default + Debug + Send + Sync + Sized
{
    const MAX: Self;

    /// Converts the given depth value from a floating-point value to the depth value.
    ///
    /// # Arguments
    /// * `depth` - The depth value in floating-point encoding.
    fn from_f32(depth: f32) -> Self;

    /// Converts the depth value to a floating-point value.
    fn to_f32(self) -> f32;
}

impl DepthBufferPrecisionType for u32 {
    const MAX: u32 = u32::MAX;

    #[inline]
    fn from_f32(depth: f32) -> Self {
        debug_assert!((0f32..=1f32).contains(&depth));
        const F_MAX: f32 = u32::MAX as f32;
        (depth * F_MAX) as Self
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32 / u32::MAX as f32
    }
}

impl DepthBufferPrecisionType for u16 {
    const MAX: u16 = u16::MAX;

    #[inline]
    fn from_f32(depth: f32) -> Self {
        debug_assert!((0f32..=1f32).contains(&depth));
        const F_MAX: f32 = u16::MAX as f32;
        (depth * F_MAX) as Self
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32 / u16::MAX as f32
    }
}
