use nalgebra_glm::{vec4_to_vec3, Mat3x4, Mat4, Vec3, Vec4};

/// Constraint a value to lie between two further values
///
/// # Arguments
/// * `x` - The value to constraint.
/// * `min_value` - The lower bound for the value constraint.
/// * `max_value` - The upper bound for the value constraint.
#[inline]
pub fn clamp<T>(x: T, min_value: T, max_value: T) -> T
where
    T: PartialOrd,
{
    if x < min_value {
        min_value
    } else if x > max_value {
        max_value
    } else {
        x
    }
}

/// Transforms the given vec3 with the given homogenous transformation matrix and returns the
/// transformed vec3.
///
/// # Arguments
/// * `t` - The 4x4 homogenous transformation matrix.
/// * `p` - The 3D vector to transform.
#[inline]
pub fn transform_vec3(t: &Mat4, p: &Vec3) -> Vec3 {
    let p = t * Vec4::new(p[0], p[1], p[2], 1f32);
    vec4_to_vec3(&p) / p[3]
}

/// Transforms the given position in world coordinates into screen coordinates.
///
/// # Arguments
/// * `width` - The width of the frame in pixels.
/// * `height` - The height of the frame in pixels.
/// * `t` - The combined projection, view and model matrix.
/// * `p` - The position in world coordinates to project.
#[inline]
pub fn project_pos(width: f32, height: f32, t: &Mat4, p: &Vec3) -> Vec3 {
    let p = transform_vec3(t, p);

    let x = (p[0] * 0.5 + 0.5) * width;
    let y = (p[1] * 0.5 + 0.5) * height;
    let z = (1.0 + p[2]) * 0.5;

    Vec3::new(x, y, z)
}

/// Converts a Mat4 to a Mat3x4 matrix.
///
/// # Arguments
/// * `mat` - The Mat4 matrix to convert.
pub fn mat4_to_mat3x4(mat: &Mat4) -> Mat3x4 {
    Mat3x4::new(
        mat[(0, 0)],
        mat[(0, 1)],
        mat[(0, 2)],
        mat[(0, 3)],
        mat[(1, 0)],
        mat[(1, 1)],
        mat[(1, 2)],
        mat[(1, 3)],
        mat[(2, 0)],
        mat[(2, 1)],
        mat[(2, 2)],
        mat[(2, 3)],
    )
}

/// Converts a Mat3x4 to a Mat4 matrix.
///
/// # Arguments
/// * `mat` - The Mat3x4 matrix to convert.
pub fn mat3x4_to_mat4(mat: &Mat3x4) -> Mat4 {
    Mat4::new(
        mat[(0, 0)],
        mat[(0, 1)],
        mat[(0, 2)],
        mat[(0, 3)],
        mat[(1, 0)],
        mat[(1, 1)],
        mat[(1, 2)],
        mat[(1, 3)],
        mat[(2, 0)],
        mat[(2, 1)],
        mat[(2, 2)],
        mat[(2, 3)],
        0.0,
        0.0,
        0.0,
        1.0,
    )
}

#[cfg(test)]
mod test {
    use nalgebra_glm::{rotate, translate};

    use super::*;

    #[test]
    fn test_clamp() {
        assert_eq!(clamp(3, 0, 10), 3);
        assert_eq!(clamp(-2, 0, 10), 0);
        assert_eq!(clamp(11, 0, 10), 10);
    }

    #[test]
    fn test_mat4_to_mat3x4() {
        let mat = translate(&Mat4::identity(), &Vec3::new(1.0, 2.0, 3.0));
        let mat3x4 = mat4_to_mat3x4(&mat);
        assert_eq!(mat3x4.column(3), Vec3::new(1.0, 2.0, 3.0));

        let mat = rotate(&Mat4::identity(), 90.0, &Vec3::new(1.0, 0.0, 0.0));
        let mat3x4 = mat4_to_mat3x4(&mat);

        // check the inner 3x3 part of both matrices are equal
        assert_eq!(mat3x4.column(0), vec4_to_vec3(&mat.column(0).into_owned()));
        assert_eq!(mat3x4.column(1), vec4_to_vec3(&mat.column(1).into_owned()));
        assert_eq!(mat3x4.column(2), vec4_to_vec3(&mat.column(2).into_owned()));
    }

    #[test]
    fn test_mat3x4_to_mat4() {
        let mat = translate(&Mat4::identity(), &Vec3::new(1.0, 2.0, 3.0));
        let mat3x4 = mat4_to_mat3x4(&mat);
        let mat2 = mat3x4_to_mat4(&mat3x4);
        assert_eq!(mat, mat2);

        let mat = rotate(&Mat4::identity(), 90.0, &Vec3::new(1.0, 0.0, 0.0));
        let mat3x4 = mat4_to_mat3x4(&mat);
        let mat2 = mat3x4_to_mat4(&mat3x4);
        assert_eq!(mat, mat2);
    }
}
