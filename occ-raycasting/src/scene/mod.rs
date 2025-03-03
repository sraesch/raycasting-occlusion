mod io;
mod io_utils;

pub use io::*;

use nalgebra_glm::{TVec3, Vec3};

/// A simple scene.
#[derive(Default)]
pub struct Scene {
    pub meshes: Vec<Mesh>,
    pub objects: Vec<Object>,
}

impl Scene {
    /// Returns `true` if all objects are valid and false otherwise.
    pub fn is_valid(&self) -> bool {
        let num_meshes = self.meshes.len() as u32;
        self.objects.iter().all(|o| o.mesh_index < num_meshes)
    }
}

/// A simple tessellated mesh.
#[derive(Clone, Default)]
pub struct Mesh {
    pub vertices: Vec<Vec3>,
    pub indices: Vec<Triangle>,
}

impl Mesh {
    /// Returns `true` if all indices are valid and false otherwise.
    pub fn is_valid(&self) -> bool {
        let num_vertices = self.vertices.len() as u32;

        self.indices
            .iter()
            .all(|t| t.x < num_vertices && t.y < num_vertices && t.z < num_vertices)
    }
}

pub type Triangle = TVec3<u32>;

pub struct Object {
    pub mesh_index: u32,
    pub transform: Transform,
}

pub type Transform = nalgebra_glm::Mat3x4;

#[cfg(test)]
mod test {
    use nalgebra_glm::Vec4;

    use super::*;

    #[test]
    fn test_transform() {
        let pos = Vec4::new(1.0, 2.0, 3.0, 1.0);

        let transform: Transform = Transform::identity();
        assert_eq!(transform * pos, Vec3::new(1.0, 2.0, 3.0));

        let transform: Transform = Transform::new(
            1.0, 0.0, 0.0, 4.0, //
            0.0, 1.0, 0.0, 5.0, //
            0.0, 0.0, 1.0, 6.0, //
        );

        assert_eq!(transform * pos, Vec3::new(5.0, 7.0, 9.0));

        let transform: Transform = Transform::new(
            2.0, 0.0, 0.0, 4.0, //
            0.0, 2.0, 0.0, 5.0, //
            0.0, 0.0, 2.0, 6.0, //
        );

        assert_eq!(transform * pos, Vec3::new(6.0, 9.0, 12.0));
    }
}
