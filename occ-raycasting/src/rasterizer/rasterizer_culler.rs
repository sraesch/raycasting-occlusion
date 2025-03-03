use log::trace;
use nalgebra_glm::Mat4;

use crate::{
    math::{mat3x4_to_mat4, project_pos},
    Error, IndexedScene, Mesh, OccOptions, OcclusionTester, Result, Scene, StatsNodeTrait,
    TestStats, Visibility,
};

use super::{rasterizer::Rasterizer, Frame};

/// A rasterizer culler that culls triangles based on the given CAD data.
pub struct RasterizerCuller<'a> {
    stats: crate::StatsNode,
    options: OccOptions,
    scene: &'a Scene,
    rasterizer: Rasterizer<u32>,
}

impl<'a> RasterizerCuller<'a> {
    /// Returns the frame size based on the number of samples.
    pub fn frame_size(&self) -> f32 {
        (self.options.num_samples as f64).sqrt().ceil() as f32
    }

    /// Rasterizes the data and returns the stats about the rendering process.
    ///
    /// # Arguments
    /// * `view_matrix` - The view matrix of the camera.
    /// * `projection_matrix` - The projection matrix of the camera.
    fn rasterize_data(
        &mut self,
        view_matrix: nalgebra_glm::Mat4,
        projection_matrix: nalgebra_glm::Mat4,
    ) -> TestStats {
        let mut stats = TestStats::default();
        let s = self.stats.get_child("rasterize");
        let _t = s.register_timing();

        let t = projection_matrix * view_matrix;

        for (object_id, object) in self.scene.objects.iter().enumerate() {
            trace!("Rasterize object: {}", object_id);

            let mesh = &self.scene.meshes[object.mesh_index as usize];

            let t = t * mat3x4_to_mat4(&object.transform);
            self.rasterize_mesh(object_id as u32, &mesh, &t, &mut stats);
        }

        stats
    }

    /// Rasterizes the given mesh with the respective transform.
    ///
    /// # Arguments
    /// * `object_id` - The id of the object.
    /// * `mesh` - The mesh to rasterize.
    /// * `transform` - The transformation matrix to apply to the vertices,
    ///                 i.e. the model-view-projection matrix.
    /// * `stats` - The stats to update.
    fn rasterize_mesh(
        &mut self,
        object_id: u32,
        mesh: &Mesh,
        transform: &Mat4,
        stats: &mut TestStats,
    ) {
        // compute the square root of the number of samples and round it up
        let frame_size = (self.options.num_samples as f64).sqrt().ceil() as f32;

        let positions = &mesh.vertices;

        for t in mesh.indices.iter() {
            stats.num_triangles += 1;

            let v0 = project_pos(frame_size, frame_size, transform, &positions[t[0] as usize]);
            let v1 = project_pos(frame_size, frame_size, transform, &positions[t[1] as usize]);
            let v2 = project_pos(frame_size, frame_size, transform, &positions[t[2] as usize]);

            self.rasterizer.rasterize(object_id, &v0, &v1, &v2);
        }
    }

    /// Computes the visibility based on the rasterized ids in the framebuffer.
    ///
    /// # Arguments
    /// * `visibility` - The visibility to update.
    fn compute_visibility_internal(&self, visibility: &mut Visibility) {
        // first create a histogram of the rendered ids
        let num_objects = self.scene.objects.len();
        let mut histogram = vec![0u32; num_objects];
        for id in self.rasterizer.id_buffer.iter() {
            match id {
                Some(id) => {
                    histogram[*id as usize] += 1;
                }
                None => {}
            }
        }

        // make sure that the visibility has the correct size
        visibility.resize(num_objects, (0, 0f32));

        // now fill the visibility based on the histogram, but not order yet
        for ((object_id, count), v) in histogram.iter().enumerate().zip(visibility.iter_mut()) {
            v.0 = object_id as u32;
            v.1 = *count as f32 / self.rasterizer.id_buffer.len() as f32;
        }

        // sort the visibility based on the visibility
        visibility.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    }
}

impl<'a> OcclusionTester<'a> for RasterizerCuller<'a> {
    type IndexedSceneType = Scene;

    fn get_name(&self) -> &str {
        "Rasterizer Culler"
    }

    fn new(stats: crate::StatsNode, scene: &'a Scene, options: OccOptions) -> Result<Self> {
        // compute the width == height which is the square root of the number of samples
        let s: usize = (options.num_samples as f64).sqrt().ceil() as usize;
        let rasterizer = Rasterizer::new(s, s);

        Ok(Self {
            stats,
            options,
            scene,
            rasterizer,
        })
    }

    fn compute_visibility(
        &mut self,
        visibility: &mut Visibility,
        frame: Option<&mut Frame>,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> TestStats {
        let stats = self.rasterize_data(view_matrix, projection_matrix);

        if let Some(frame) = frame {
            *frame = self.rasterizer.get_frame();
        }

        self.compute_visibility_internal(visibility);

        stats
    }
}
