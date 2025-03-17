mod frame;
mod rasterizer;

use arrayvec::ArrayVec;
pub use frame::*;
use log::trace;
use nalgebra_glm::{transpose, Mat4, Vec3};
use rasterizer::Rasterizer;

use std::fmt::Debug;

use crate::{
    math::{
        mat3x4_to_mat4, project_pos, transform_vec3, triangle_plane_intersection, Plane,
        TrianglePlaneIntersection,
    },
    utils::compute_visibility_from_id_buffer,
    OccOptions, OcclusionTester, Result, Scene, StatsNodeTrait, TestStats, Visibility,
};

pub trait DepthBufferPrecisionType:
    Clone + Copy + PartialEq + PartialOrd + Default + Debug + Send + Sync + Sized
{
    const MAX: Self;

    /// Converts the given depth value from a floating-point value to the depth value.
    ///
    /// # Arguments
    /// * `depth` - The depth value in floating-point encoding.
    fn from_f32(depth: f32) -> Self;

    /// Converts the depth value to a floating-point value.
    fn to_f32(self) -> f32;
}

impl DepthBufferPrecisionType for u32 {
    const MAX: u32 = u32::MAX;

    #[inline]
    fn from_f32(depth: f32) -> Self {
        debug_assert!((0f32..=1f32).contains(&depth));
        const F_MAX: f32 = u32::MAX as f32;
        (depth * F_MAX) as Self
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32 / u32::MAX as f32
    }
}

impl DepthBufferPrecisionType for u16 {
    const MAX: u16 = u16::MAX;

    #[inline]
    fn from_f32(depth: f32) -> Self {
        debug_assert!((0f32..=1f32).contains(&depth));
        const F_MAX: f32 = u16::MAX as f32;
        (depth * F_MAX) as Self
    }

    #[inline]
    fn to_f32(self) -> f32 {
        self as f32 / u16::MAX as f32
    }
}

/// A rasterizer culler that culls triangles based on the given CAD data.
pub struct RasterizerCuller {
    stats: crate::StatsNode,
    options: OccOptions,
    scene: Scene,
    rasterizer: Rasterizer<u32>,
}

impl RasterizerCuller {
    /// Clears the rasterizer.
    #[inline]
    pub fn clear(&mut self) {
        self.rasterizer.clear();
    }
    /// Rasterizes the data and returns the stats about the rendering process.
    ///
    /// # Arguments
    /// * `near` - The near plane of the camera.
    /// * `view_matrix` - The view matrix of the camera.
    /// * `projection_matrix` - The projection matrix of the camera.
    pub fn rasterize_data(
        &mut self,
        near: f32,
        view_matrix: nalgebra_glm::Mat4,
        projection_matrix: nalgebra_glm::Mat4,
    ) -> TestStats {
        let frame_size = self.options.frame_size as f32;
        let mut stats = TestStats::default();
        let s = self.stats.get_child("rasterize");
        let _t = s.register_timing();

        let near = -near;

        // iterate over all objects and rasterize them
        for (object_id, object) in self.scene.objects.iter().enumerate() {
            let object_id = object_id as u32;
            trace!("Rasterize object: {}", object_id);

            let model_view = view_matrix * mat3x4_to_mat4(&object.transform);

            let mesh = &self.scene.meshes[object.mesh_index as usize];
            let positions = &mesh.vertices;

            for t in mesh.indices.iter() {
                let p: ArrayVec<Vec3, 3> = ArrayVec::from_iter(
                    (0..3).map(|i| transform_vec3(&model_view, &positions[t[i] as usize])),
                );

                let z = [p[0].z, p[1].z, p[2].z];
                let max_z = z.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
                // println!("max z: {:?}", max_z);

                if max_z >= near {
                    continue;
                }

                let v: ArrayVec<Vec3, 3> = ArrayVec::from_iter(
                    (0..3).map(|i| project_pos(frame_size, frame_size, &projection_matrix, &p[i])),
                );

                self.rasterizer.rasterize(object_id, &v);

                // match triangle_plane_intersection(&near_plane, p) {
                //     TrianglePlaneIntersection::Front => {
                //         stats.num_triangles += 1;
                //         let v0 = project_pos(
                //             frame_size,
                //             frame_size,
                //             &transform,
                //             &positions[t[0] as usize],
                //         );
                //         let v1 = project_pos(
                //             frame_size,
                //             frame_size,
                //             &transform,
                //             &positions[t[1] as usize],
                //         );
                //         let v2 = project_pos(
                //             frame_size,
                //             frame_size,
                //             &transform,
                //             &positions[t[2] as usize],
                //         );

                //         self.rasterizer.rasterize(object_id, &v0, &v1, &v2);
                //     }
                //     TrianglePlaneIntersection::Behind => {
                //         trace!("Triangle is behind the near plane");
                //     }
                //     TrianglePlaneIntersection::Intersecting((n, p)) => {
                //         trace!("Triangle is intersecting the near plane");
                //         stats.num_triangles += 1;
                //         let v0 = project_pos(frame_size, frame_size, &transform, &p[0]);
                //         let v1 = project_pos(frame_size, frame_size, &transform, &p[1]);
                //         let v2 = project_pos(frame_size, frame_size, &transform, &p[2]);

                //         self.rasterizer.rasterize(object_id, &v0, &v1, &v2);

                //         if n == 4 {
                //             stats.num_triangles += 1;
                //             let v3 = project_pos(frame_size, frame_size, &transform, &p[3]);
                //             self.rasterizer.rasterize(object_id, &v0, &v2, &v3);
                //         }
                //     }
            }

            // let v0 = project_pos(
            //     frame_size,
            //     frame_size,
            //     &transform,
            //     &positions[t[0] as usize],
            // );
            // let v1 = project_pos(
            //     frame_size,
            //     frame_size,
            //     &transform,
            //     &positions[t[1] as usize],
            // );
            // let v2 = project_pos(
            //     frame_size,
            //     frame_size,
            //     &transform,
            //     &positions[t[2] as usize],
            // );

            // self.rasterizer.rasterize(object_id, &v0, &v1, &v2);
            // }
        }

        stats
    }

    /// Returns the rasterizer.
    pub fn get_rasterizer(&self) -> &Rasterizer<u32> {
        &self.rasterizer
    }

    /// Computes the visibility based on the rasterized ids in the framebuffer.
    ///
    /// # Arguments
    /// * `visibility` - The visibility to update.
    fn compute_visibility_internal(&self, visibility: &mut Visibility) {
        let num_objects = self.scene.objects.len();
        let frame = self.rasterizer.get_frame();
        compute_visibility_from_id_buffer(visibility, frame.get_id_buffer(), num_objects);
    }

    /// Extracts the near plane from the given model-view projection matrix matrix.
    ///
    /// # Arguments
    /// * `m` - The model-view projection matrix.
    fn extract_near_plane(m: &Mat4) -> Plane {
        let tm = transpose(m);

        Plane::from_equation_with_normalization(&(tm.column(3) + tm.column(2)))
    }
}

impl OcclusionTester for RasterizerCuller {
    type IndexedSceneType = Scene;

    fn get_name() -> &'static str {
        "rasterizer_occ"
    }

    fn new(stats: crate::StatsNode, scene: Scene, options: OccOptions) -> Result<Self> {
        // compute the width == height which is the square root of the number of samples
        let s: usize = options.frame_size;
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
        self.rasterizer.clear();

        let near = projection_matrix[14] / (projection_matrix[10] - 1.0);

        let stats = self.rasterize_data(near, view_matrix, projection_matrix);

        if let Some(frame) = frame {
            *frame = self.rasterizer.get_frame();
        }

        self.compute_visibility_internal(visibility);

        stats
    }
}
