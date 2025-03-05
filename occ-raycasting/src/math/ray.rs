use nalgebra_glm::{normalize, Vec3};

/// A single ray that starts at pos and goes into infinity along dir
pub struct Ray {
    /// The start position of the ray
    pub pos: Vec3,

    /// The normalized direction of the ray.
    pub dir: Vec3,
}

impl Ray {
    /// Creates a new ray spanned by the two positions x0 and x1.
    ///
    /// # Arguments
    /// * `x0` - The start position of the ray
    /// * `x1` - The next position along the line of the ray.
    pub fn from_pos(x0: &Vec3, x1: &Vec3) -> Self {
        Self {
            dir: normalize(&(x1 - x0)),
            pos: *x0,
        }
    }
}
