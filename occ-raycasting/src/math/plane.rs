use nalgebra_glm::{vec4_to_vec3, Vec3, Vec4};

use super::{Ray, AABB};

pub struct Plane {
    pub d: f32,
    pub n: Vec3,
}

impl Plane {
    /// Creates the plane that is orthogonal to the given ray.
    ///
    /// # Arguments
    /// * `ray` - The ray which is orthogonal to the plane
    pub fn from_ray(ray: &Ray) -> Self {
        let n = ray.dir;
        let d = -n.dot(&ray.pos);

        Self { d, n }
    }

    /// Creates the plane from the given plane equation and normalizes it.
    /// The plane equation is as following:
    ///
    /// Let eq = (a,b,c,d) and p = (x,y,z) be some point. Then the equation is
    /// a*x + b*y + c*z + d = 0     if point is on the plane
    /// a*x + b*y + c*z + d > 0     if point is in the positive half-space of the plane
    /// a*x + b*y + c*z + d < 0     if point is in the negative half-space of the plane
    ///
    /// # Arguments
    /// * `eq` - The plane equation coefficients (a,b,c,d)
    pub fn from_equation_with_normalization(eq: &Vec4) -> Self {
        let mut n = vec4_to_vec3(&eq);
        let l = n.norm();
        assert!(l > 0f32, "Length of the normal part must be positive");

        n /= l;
        let d = eq[3] / l;

        Self { d, n }
    }

    /// Creates a plane spanned by the two given basis vectors and moved to the position.
    ///
    /// # Argument
    /// * `pos` - A position on the plane.
    /// * `b0` - The first basis vector that spans the plane.
    /// * `b1` - The second basis vector that spans the plane.
    pub fn from_basis(pos: &Vec3, b0: &Vec3, b1: &Vec3) -> Self {
        let n = b0.cross(&b1).normalize();
        let d = -n.dot(&pos);

        Self { d, n }
    }

    /// Creates a plane spanned by the given triangle.
    ///
    /// # Argument
    /// * `p0` - The first vertex of the triangle.
    /// * `p1` - The second vertex of the triangle.
    /// * `p2` - The third vertex of the triangle.
    pub fn from_triangle(p0: &Vec3, p1: &Vec3, p2: &Vec3) -> Self {
        let b0 = p1 - p0;
        let b1 = p2 - p0;
        Self::from_basis(p0, &b0, &b1)
    }

    /// Returns the signed distance, i.e., the distance between the plane and the point that can
    /// be negative or positive.
    ///
    /// # Arguments
    /// * `p` - The point to which the signed distance will be computed.
    #[inline]
    pub fn signed_distance(&self, p: &Vec3) -> f32 {
        self.n.dot(p) + self.d
    }

    /// Checks if the given aabb volume is in the negative half-space of the plane.
    ///
    /// # Arguments
    /// * `aabb` - The aabb volume to check.
    #[inline]
    pub fn is_aabb_negative_half_space(&self, aabb: &AABB) -> bool {
        // determine the corner of the aabb volume that has the largest signed distance
        let x = if self.n[0] < 0f32 {
            aabb.min[0]
        } else {
            aabb.max[0]
        };
        let y = if self.n[1] < 0f32 {
            aabb.min[1]
        } else {
            aabb.max[1]
        };
        let z = if self.n[2] < 0f32 {
            aabb.min[2]
        } else {
            aabb.max[2]
        };

        self.signed_distance(&Vec3::new(x, y, z)) <= 0f32
    }
}

#[cfg(test)]
mod test {
    use nalgebra_glm::normalize;

    use super::*;

    #[test]
    fn test_signed_distance() {
        let pos = Vec3::new(1f32, 2f32, 3f32);
        let dir = normalize(&Vec3::new(1f32, 1f32, 1f32));
        let plane = Plane::from_ray(&Ray { pos, dir });

        assert_eq!(plane.signed_distance(&pos), 0f32);
        assert!(plane.signed_distance(&Vec3::new(2f32, 4f32, 5f32)) > 0f32);
        assert!(plane.signed_distance(&Vec3::new(0.5f32, 1f32, 3f32)) < 0f32);
    }

    #[test]
    fn test_is_aabb_negative_half_space() {
        let pos = Vec3::new(1f32, 2f32, 3f32);
        let dir = normalize(&Vec3::new(1f32, 1f32, 1f32));
        let plane = Plane::from_ray(&Ray { pos, dir });

        let aabb = AABB::new_cube(&pos, 1f32);
        assert!(!plane.is_aabb_negative_half_space(&aabb));

        let aabb = AABB::new_cube(&Vec3::new(0.5f32, 1f32, 3f32), 0.11f32);
        assert!(plane.is_aabb_negative_half_space(&aabb));

        let aabb = AABB::new_cube(&Vec3::new(0f32, 1f32, 2f32), 2f32);
        assert!(plane.is_aabb_negative_half_space(&aabb));

        let aabb = AABB::new_cube(&Vec3::new(0.1f32, 1f32, 2f32), 2f32);
        assert!(!plane.is_aabb_negative_half_space(&aabb));
    }
}
