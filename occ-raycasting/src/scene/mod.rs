mod io;
mod io_utils;

pub use io::*;

use crate::{Error, Result};
use nalgebra_glm::{TVec3, Vec3};
use serde::{Deserialize, Serialize};

/// A simple scene.
#[derive(Default, Serialize, Deserialize)]
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

    /// Writes the scene to the given writer.
    ///
    /// # Arguments
    /// * `writer` - The writer to write the scene to.
    pub fn write<W: std::io::Write>(&self, writer: W) -> Result<()> {
        bincode::serialize_into(writer, self).map_err(|e| Error::SerializationError(Box::new(e)))
    }

    /// Reads the scene from the given reader.
    ///
    /// # Arguments
    /// * `reader` - The reader to read the scene from.
    pub fn read_from<R: std::io::Read>(reader: R) -> Result<Self> {
        bincode::deserialize_from(reader).map_err(|e| Error::DeserializationError(Box::new(e)))
    }
}

/// A simple tessellated mesh.
#[derive(Clone, Default, Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize)]
pub struct Object {
    pub mesh_index: u32,
    pub transform: Transform,
}

pub type Transform = nalgebra_glm::Mat3x4;

#[cfg(test)]
mod test {
    use cad_import::loader::{Manager, MemoryResource};
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

    #[test]
    fn test_serialize_and_deserialize_scene() {
        let mut scene = Scene::default();
        {
            let scene_data = include_bytes!("../../../test_data/box.glb");
            let memory_resource = MemoryResource::new(scene_data, "model/gltf-binary".to_string());
            let m = Manager::new();
            let cad_data = m
                .get_loader_by_mime_type("model/gltf-binary")
                .unwrap()
                .read(&memory_resource)
                .unwrap();
            add_cad_data_to_scene(&mut scene, &cad_data);
        }

        // serialize the scene to a buffer
        let mut buffer = Vec::new();
        scene.write(&mut buffer).unwrap();

        // deserialize the scene from the buffer
        let scene2 = Scene::read_from(&buffer[..]).unwrap();

        // compare the two scenes
        assert_eq!(scene.meshes.len(), scene2.meshes.len());
        assert_eq!(scene.objects.len(), scene2.objects.len());

        for (m1, m2) in scene.meshes.iter().zip(scene2.meshes.iter()) {
            assert_eq!(m1.vertices.len(), m2.vertices.len());
            assert_eq!(m1.indices.len(), m2.indices.len());

            for (v1, v2) in m1.vertices.iter().zip(m2.vertices.iter()) {
                assert_eq!(v1, v2);
            }

            for (i1, i2) in m1.indices.iter().zip(m2.indices.iter()) {
                assert_eq!(i1, i2);
            }
        }

        for (o1, o2) in scene.objects.iter().zip(scene2.objects.iter()) {
            assert_eq!(o1.mesh_index, o2.mesh_index);
            assert_eq!(o1.transform, o2.transform);
        }
    }
}
