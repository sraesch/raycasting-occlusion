use nalgebra_glm::Vec3;

use super::{Plane, Ray, AABB};

/// Determines the intersection between the given triangle and ray. If there is an intersection it
/// returned the coefficient f that defines the intersection point along the given ray.
/// That is, ray.pos + f * ray.dir is the intersection point
///
/// # Arguments
/// * `p0` - The first vertex of the triangle.
/// * `p1` - The second vertex of the triangle.
/// * `p2` - The third vertex of the triangle.
/// * `ray` - The ray to compute the intersection with.
/// * `max_f` - Optionally, the maximum value for f. If the intersection point is further away
///             than max_f, None is returned.
pub fn triangle_ray(p0: &Vec3, p1: &Vec3, p2: &Vec3, ray: &Ray, max_f: Option<f32>) -> Option<f32> {
    // compute intersection point with the plane of the triangle and the given ray
    let plane = Plane::from_triangle(p0, p1, p2);
    let lambda = match plane_ray(&plane, ray) {
        Some(lambda) => lambda,
        None => {
            return None;
        }
    };

    if let Some(max_f) = max_f {
        if lambda > max_f {
            return None;
        }
    }

    let pos0: Vec3 = ray.pos + lambda * ray.dir;

    // check if the intersection is located inside the triangle
    // see: https://www.scratchapixel.com/lessons/3d-basic-rendering/ray-tracing-rendering-a-triangle/ray-triangle-intersection-geometric-solution.html
    let edge0: Vec3 = p1 - p0;
    let edge1: Vec3 = p2 - p1;
    let edge2: Vec3 = p0 - p2;
    let c0: Vec3 = pos0 - p0;
    let c1: Vec3 = pos0 - p1;
    let c2: Vec3 = pos0 - p2;

    // check if the intersection point is inside the triangle.
    if plane.n.dot(&edge0.cross(&c0)) > 0f32
        && plane.n.dot(&edge1.cross(&c1)) > 0f32
        && plane.n.dot(&edge2.cross(&c2)) > 0f32
    {
        Some(lambda)
    } else {
        None
    }
}

/// Determines the intersection between the given plane and ray. If there is an intersection it
/// returned the coefficient a that defines the intersection point along the given ray.
/// That is, ray.pos + a * ray.dir is the intersection point
///
/// # Arguments
/// * `plane` - The plane to compute the intersection with.
/// * `ray` - The ray to compute the intersection with.
pub fn plane_ray(plane: &Plane, ray: &Ray) -> Option<f32> {
    let a = plane.n.dot(&ray.dir);
    if a == 0f32 {
        return None;
    }

    let lambda = -(plane.d + plane.n.dot(&ray.pos)) / a;
    if lambda < 0f32 {
        None
    } else {
        Some(lambda)
    }
}

/// Determines the intersection between the given AABB and ray. If there is an intersection it
/// returned the coefficient f that defines the intersection point along the given ray.
/// That is, ray.pos + f * ray.dir is the intersection point
///
/// # Arguments
/// * `aabb` - The AABB to compute the intersection with.
/// * `ray` - The ray to compute the intersection with.
/// * `max_f` - Optionally, the maximum value for f. If the intersection point is further away
///             than max_f, None is returned.
pub fn aabb_ray(aabb: &AABB, ray: &Ray, max_f: Option<f32>) -> Option<f32> {
    let mut t_min = 0f32;
    let mut t_max = max_f.unwrap_or(f32::MAX);

    // we iterate over each axis and determine the intersection point with the AABB
    for axis in 0..3 {
        // If the ray is parallel to the plane we check if the ray is inside the AABB.
        // If the ray is not inside the AABB we return None, because the ray does cannot intersect.
        if ray.dir[axis] == 0f32
            && (ray.pos[axis] < aabb.min[axis] || ray.pos[axis] > aabb.max[axis])
        {
            return None;
        }

        let t0 = (aabb.min[axis] - ray.pos[axis]) / ray.dir[axis];
        let t1 = (aabb.max[axis] - ray.pos[axis]) / ray.dir[axis];

        t_min = t_min.max(t0.min(t1));
        t_max = t_max.min(t0.max(t1));

        if t_min > t_max {
            return None;
        }
    }

    Some(t_min)
}

#[cfg(test)]
mod test {
    use std::ops::Range;

    use super::*;

    use rand::prelude::*;
    use rand_chacha::ChaCha8Rng;

    /// We tessellated the AABB into 12 triangles and test the intersection with the given ray.
    /// This function is only used for testing purposes.
    ///
    /// # Arguments
    /// * `aabb` - The AABB to tessellate.
    /// * `ray` - The ray to test the intersection with.
    fn aabb_intersection_test_using_triangles(aabb: &AABB, ray: &Ray) -> Option<f32> {
        let mut depth = f32::MAX;

        if aabb.contains_point(&ray.pos) {
            return Some(0f32);
        }

        for axis in 0..3 {
            for dir in 0..2 {
                let value = if dir == 0 {
                    aabb.min[axis]
                } else {
                    aabb.max[axis]
                };

                let axis0 = (axis + 1) % 3;
                let axis1 = (axis + 2) % 3;

                let mut p0 = Vec3::zeros();
                let mut p1 = Vec3::zeros();
                let mut p2 = Vec3::zeros();
                let mut p3 = Vec3::zeros();

                p0[axis] = value;
                p0[axis0] = aabb.min[axis0];
                p0[axis1] = aabb.min[axis1];

                p1[axis] = value;
                p1[axis0] = aabb.max[axis0];
                p1[axis1] = aabb.min[axis1];

                p2[axis] = value;
                p2[axis0] = aabb.max[axis0];
                p2[axis1] = aabb.max[axis1];

                p3[axis] = value;
                p3[axis0] = aabb.min[axis0];
                p3[axis1] = aabb.max[axis1];

                if let Some(d) = triangle_ray(&p0, &p1, &p2, ray, Some(depth)) {
                    if depth > d {
                        depth = d;
                    }
                }

                if let Some(d) = triangle_ray(&p0, &p2, &p3, ray, Some(depth)) {
                    if depth > d {
                        depth = d;
                    }
                }
            }
        }

        if depth == f32::MAX {
            None
        } else {
            Some(depth)
        }
    }

    /// Generates a random AABB with the given min and max values.
    ///
    /// # Arguments
    /// * `rng` - The random number generator.
    /// * `r` - The range of the AABB.
    /// * `n` - The number of points to the AABB.
    fn gen_random_aabb(rng: &mut ChaCha8Rng, r: Range<f32>, n: usize) -> AABB {
        let mut aabb: AABB = Default::default();

        for _ in 0..n {
            let p = Vec3::new(
                rng.random_range(r.clone()),
                rng.random_range(r.clone()),
                rng.random_range(r.clone()),
            );

            aabb.extend_pos(&p);
        }

        aabb
    }

    #[test]
    fn test_aabb_ray() {
        let mut r = ChaCha8Rng::seed_from_u64(2);

        let float_min = -10.0;
        let float_max = 10.0;

        let mut num_non_trivial_hits = 0;

        // generate random AABB
        let num_aabb = 1000;
        let num_rays = 10;
        for _ in 0..num_aabb {
            // generate random AABB
            let aabb: AABB = gen_random_aabb(&mut r, float_min..float_max, 10);

            // generate random ray
            for _ in 0..num_rays {
                let ray = Ray::from_pos(
                    &Vec3::new(
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                    ),
                    &Vec3::new(
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                        r.random_range((float_min * 2f32)..(float_max * 2f32)),
                    ),
                );

                let f1 = aabb_ray(&aabb, &ray, None);
                let f2 = aabb_intersection_test_using_triangles(&aabb, &ray);

                let p0 = f1.map(|f| ray.pos + f * ray.dir);
                let p1 = f2.map(|f| ray.pos + f * ray.dir);

                assert_eq!(f1, f2, "AABB {:?}, Ray {:?}: Position according to aabb_ray {:?} and according to aabb_intersection_test_using_triangles {:?}", aabb, ray, p0, p1);

                if let Some(f) = f1 {
                    if f > 0f32 {
                        num_non_trivial_hits += 1;
                    }
                }
            }
        }

        println!("Number of non-trivial hits: {}", num_non_trivial_hits);
    }
}
