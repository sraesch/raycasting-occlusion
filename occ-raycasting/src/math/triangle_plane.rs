use nalgebra_glm::Vec3;

use super::Plane;

/// The result of the intersection between a triangle and a plane.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TrianglePlaneIntersection {
    Front,
    Behind,

    /// The resulting triangle-strip of the resulting intersected triangle.
    Intersecting((usize, [Vec3; 4])),
}

/// Determines the intersection between the given triangle and plane.
///
/// # Arguments
/// * `plane` - The plane to compute the intersection with.
/// * `p0` - The first vertex of the triangle.
/// * `p1` - The second vertex of the triangle.
/// * `p2` - The third vertex of the triangle.
pub fn triangle_plane_intersection(plane: &Plane, p: [&Vec3; 3]) -> TrianglePlaneIntersection {
    let d = [
        plane.signed_distance(p[0]),
        plane.signed_distance(p[1]),
        plane.signed_distance(p[2]),
    ];

    if d[0] > 0f32 && d[1] > 0f32 && d[2] > 0f32 {
        TrianglePlaneIntersection::Front
    } else if d[0] < 0f32 && d[1] < 0f32 && d[2] < 0f32 {
        TrianglePlaneIntersection::Behind
    } else {
        let ret = compute_triangle_plane_intersection(&d, p);
        TrianglePlaneIntersection::Intersecting(ret)
    }
}

/// Computes the intersection points of the triangle with the plane.
///
/// # Arguments
/// * `d` - The signed distances of the triangle vertices to the plane.
/// * `p` - The vertices of the triangle.
fn compute_triangle_plane_intersection(d: &[f32; 3], p: [&Vec3; 3]) -> (usize, [Vec3; 4]) {
    let mut ret = [Vec3::zeros(); 4];
    let mut num_points = 0;

    for i0 in 0..3 {
        let i1 = (i0 + 1) % 3;

        let d0 = d[i0];
        let d1 = d[i1];

        // if the current point is inside, add it to the result
        if d0 >= 0f32 {
            ret[num_points] = *p[i0];
            num_points += 1;
        }

        // check if the edge intersects the plane
        if d1 * d0 < 0f32 {
            let t = d0 / (d0 - d1);
            ret[num_points] = p[i0] + t * (p[i1] - p[i0]);
            num_points += 1;
        }
    }

    (num_points, ret)
}

#[cfg(test)]
mod test {
    use nalgebra_glm::Vec4;

    use super::*;

    #[test]
    fn test_triangle_plane_intersection() {
        let plane = Plane::from_equation_with_normalization(&Vec4::new(1.0, 0.0, 0.0, 0.0));

        let p0 = Vec3::new(-1.0, 0.0, 0.0);
        let p1 = Vec3::new(0.0, 1.0, 0.0);
        let p2 = Vec3::new(1.0, 0.0, 0.0);

        let res = triangle_plane_intersection(&plane, [&p0, &p1, &p2]);

        assert_eq!(
            res,
            TrianglePlaneIntersection::Intersecting((
                3,
                [
                    Vec3::new(0.0, 1.0, 0.0),
                    Vec3::new(1.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                    Vec3::new(0.0, 0.0, 0.0),
                ]
            ))
        );
    }
}
