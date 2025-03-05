use nalgebra_glm::Vec3;

use super::{Plane, Ray};

/// Determines the intersection between the given triangle and ray. If there is an intersection it
/// returned the coefficient a that defines the intersection point along the given ray.
/// That is, ray.pos + a * ray.dir is the intersection point
///
/// # Arguments
/// * `p0` - The first vertex of the triangle.
/// * `p1` - The second vertex of the triangle.
/// * `p2` - The third vertex of the triangle.
/// * `ray` - The ray to compute the intersection with.
pub fn triangle_ray(p0: &Vec3, p1: &Vec3, p2: &Vec3, ray: &Ray) -> Option<f32> {
    // compute intersection point with the plane of the triangle and the given ray
    let plane = Plane::from_triangle(p0, p1, p2);
    let lambda = match plane_ray(&plane, ray) {
        Some(lambda) => lambda,
        None => {
            return None;
        }
    };

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
