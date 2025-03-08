use std::fmt;
use std::fmt::Display;

use nalgebra_glm as glm;
use serde::{Deserialize, Serialize};

/// An AABB bounding volume
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AABB {
    /// the corner with the lower coordinates
    pub min: glm::Vec3,
    /// the corner with the upper coordinates
    pub max: glm::Vec3,
}

impl AABB {
    /// Creates a new empty bounding volume
    pub fn new() -> Self {
        let min = glm::vec3(f32::MAX, f32::MAX, f32::MAX);
        let max = glm::vec3(f32::MIN, f32::MIN, f32::MIN);

        AABB { min, max }
    }

    /// Creates a new bounding volume from the given iterator of vec3 positions.
    ///
    /// # Arguments
    /// * `positions` - The iterator of vec3 positions to create the bounding volume from.
    pub fn from_iter<I>(positions: I) -> Self
    where
        I: Iterator<Item = glm::Vec3>,
    {
        let mut result = AABB::new();

        result.extend_iter(positions);

        result
    }

    /// Creates a new cubic bounding volume with the specified center and size.
    ///
    /// # Arguments
    /// * `center` - The center of the AABB bounding volume.
    /// * `size` - The edge length of the cubic bounding volume.
    pub fn new_cube(center: &glm::Vec3, size: f32) -> Self {
        let half_size = size / 2f32;

        let mut result = AABB::new();
        result.min = *center - glm::vec3(half_size, half_size, half_size);
        result.max = *center + glm::vec3(half_size, half_size, half_size);

        result
    }

    /// Returns true if the bbox is empty and false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min.x > self.max.x || self.min.y > self.max.y || self.min.z > self.max.z
    }

    /// Extends the bounding volume with the given position
    ///
    ///* `p` - The position about which the volume is extended
    pub fn extend_pos(&mut self, p: &glm::Vec3) {
        self.min.x = self.min.x.min(p.x);
        self.min.y = self.min.y.min(p.y);
        self.min.z = self.min.z.min(p.z);

        self.max.x = self.max.x.max(p.x);
        self.max.y = self.max.y.max(p.y);
        self.max.z = self.max.z.max(p.z);
    }

    /// Extends the bounding volume with the given position
    ///
    ///* `rhs` - The right-hand-side bounding volume about which the volume is extended
    pub fn extend_bbox(&mut self, rhs: &Self) {
        self.min.x = self.min.x.min(rhs.min.x);
        self.min.y = self.min.y.min(rhs.min.y);
        self.min.z = self.min.z.min(rhs.min.z);

        self.max.x = self.max.x.max(rhs.max.x);
        self.max.y = self.max.y.max(rhs.max.y);
        self.max.z = self.max.z.max(rhs.max.z);
    }

    /// Extends the bounding volume from the given iterator of vec3 positions.
    pub fn extend_iter<I>(&mut self, positions: I)
    where
        I: Iterator<Item = glm::Vec3>,
    {
        positions.for_each(|p| self.extend_pos(&p))
    }

    /// Computes and returns the bounding box center
    #[inline]
    pub fn get_center(&self) -> glm::Vec3 {
        let center = (self.min + self.max) / 2.0;
        center
    }

    /// Computes and returns the bounding box size
    #[inline]
    pub fn get_size(&self) -> glm::Vec3 {
        let size = self.max - self.min;
        size
    }

    /// Returns a reference onto the minimum
    #[inline]
    pub fn get_min(&self) -> &glm::Vec3 {
        &self.min
    }

    /// Returns a reference onto the maximum
    #[inline]
    pub fn get_max(&self) -> &glm::Vec3 {
        &self.max
    }

    /// Returns the i-th corner, i.e., i==0 => min and i==1 => max.
    ///
    /// # Arguments
    /// * `i` - The index of the corner to return.
    #[inline]
    pub fn corner(&self, i: usize) -> &glm::Vec3 {
        assert!(i < 2);

        if i == 0 {
            &self.min
        } else {
            &self.max
        }
    }

    #[inline]
    pub fn contains_point(&self, p: &glm::Vec3) -> bool {
        self.min[0] <= p[0]
            && p[0] <= self.max[0]
            && self.min[1] <= p[1]
            && p[1] <= self.max[1]
            && self.min[2] <= p[2]
            && p[2] <= self.max[2]
    }

    #[inline]
    pub fn contains_aabb(&self, aabb: &AABB) -> bool {
        self.min[0] <= aabb.min[0]
            && aabb.max[0] <= self.max[0]
            && self.min[1] <= aabb.min[1]
            && aabb.max[1] <= self.max[1]
            && self.min[2] <= aabb.min[2]
            && aabb.max[2] <= self.max[2]
    }

    /// Returns the euclidean distance between the AABB and the given point.
    ///
    /// # Arguments
    /// * `point` - The point to compute the distance to.
    pub fn point_distance(&self, point: &glm::Vec3) -> f32 {
        let mut distance_squared = 0.0;

        for i in 0..3 {
            let v = point[i];

            if v < self.min[i] {
                distance_squared += (self.min[i] - v) * (self.min[i] - v);
            } else if v > self.max[i] {
                distance_squared += (v - self.max[i]) * (v - self.max[i]);
            }
        }

        distance_squared.sqrt()
    }
}

impl Default for AABB {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

fn vec3_to_string(f: &mut fmt::Formatter<'_>, v: &glm::Vec3) -> fmt::Result {
    write!(f, "({}, {}, {})", v[0], v[1], v[2])
}

impl Display for AABB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        vec3_to_string(f, &self.min)?;
        write!(f, "-")?;
        vec3_to_string(f, &self.max)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_point_distance() {
        let mut aabb = AABB::new();
        aabb.extend_pos(&glm::vec3(0.0, 0.0, 0.0));
        aabb.extend_pos(&glm::vec3(1.0, 1.0, 1.0));

        assert_eq!(aabb.point_distance(&glm::vec3(0.0, 0.0, 0.0)), 0.0);
        assert_eq!(aabb.point_distance(&glm::vec3(-1.0, 0.0, 0.0)), 1.0);
        assert_eq!(aabb.point_distance(&glm::vec3(2.0, 0.0, 0.0)), 1.0);
        assert_eq!(
            aabb.point_distance(&glm::vec3(-1.0, 2.0, 0.0)),
            2.0f32.sqrt()
        );
    }
}
