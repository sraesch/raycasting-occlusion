use bincode::de;
use log::{error, trace};
use nalgebra_glm::{vec3_to_vec4, vec4_to_vec3, Mat3x4, Mat4, Vec3, Vec4};

use crate::{
    math::{extract_camera_pos_from_view_matrix, mat3x4_to_mat4, triangle_ray, Ray},
    rasterizer_culler::Frame,
    scene,
    utils::compute_visibility_from_id_buffer,
    OccOptions, OcclusionTester, Result, Scene, StatsNode, StatsNodeTrait, TestStats, Visibility,
};

/// A very simple ray caster without any acceleration structures
pub struct NaiveRaycaster {
    stats: StatsNode,
    options: OccOptions,
    scene: Scene,

    /// The id buffer of the rasterizer.
    pub id_buffer: Vec<Option<u32>>,
}

impl NaiveRaycaster {
    /// Maps window coordinates to object coordinates and returns them.
    ///
    /// # Arguments
    /// * `frame_size` - The width and height of the frame.
    /// * `inv_pmmat` - The inverse of the multiplied projection and model view matrix.
    /// * `win` - The window coordinates to be mapped
    fn un_project(frame_size: usize, inv_pmmat: &Mat4, win: &Vec3) -> Vec3 {
        let frame_size = frame_size as f32;

        // determine normalized coordinates between -1 and 1
        let mut v = Vec4::new(
            win[0] / frame_size * 2.0 - 1.0,
            win[1] / frame_size * 2.0 - 1.0,
            2.0 * win[2] - 1.0,
            1.0,
        );

        v = inv_pmmat * v;

        if v[3] != 0f32 {
            vec4_to_vec3(&v) / v[3]
        } else {
            vec4_to_vec3(&v)
        }
    }

    /// Computes the visibility based on the rasterized ids in the framebuffer.
    ///
    /// # Arguments
    /// * `visibility` - The visibility to update.
    fn compute_visibility_internal(&self, visibility: &mut Visibility) {
        let num_objects = self.scene.objects.len();
        let id_buffer = &self.id_buffer;
        compute_visibility_from_id_buffer(visibility, id_buffer, num_objects);
    }

    /// Raycasts the data.
    ///
    /// # Arguments
    /// * `view_matrix` - The view matrix.
    /// * `projection_matrix` - The projection matrix.
    fn raycast_data(&mut self, view_matrix: &Mat4, projection_matrix: &Mat4) -> TestStats {
        let pmmat = projection_matrix * view_matrix;
        let mut stats = TestStats::default();

        // extract camera position
        let x0 = extract_camera_pos_from_view_matrix(&view_matrix);

        // compute matrix for defining the rays
        let inv_pmmat = match pmmat.try_inverse() {
            Some(m) => m,
            None => {
                error!("Combined projection and model matrix are not invertible!!!");
                return stats;
            }
        };

        let s = self.stats.get_child("rasterize");
        let _t = s.register_timing();

        let id_buffer = &mut self.id_buffer;
        let scene = &self.scene;

        // cast the rays
        for y in 0..self.options.frame_size {
            for x in 0..self.options.frame_size {
                let mut depth = f32::MAX;

                // create ray for current cell
                let x1: Vec3 = Self::un_project(
                    self.options.frame_size,
                    &inv_pmmat,
                    &Vec3::new(x as f32 + 0.5f32, y as f32 + 0.5f32, 1f32),
                );

                let ray = Ray::from_pos(&x0, &x1);

                for (object_id, object) in scene.objects.iter().enumerate() {
                    let object_id = object_id as u32;

                    let mesh = &scene.meshes[object.mesh_index as usize];
                    let positions = &mesh.vertices;

                    for t in mesh.indices.iter() {
                        stats.num_triangles += 1;

                        let p0 = Self::transform(&object.transform, &positions[t[0] as usize]);
                        let p1 = Self::transform(&object.transform, &positions[t[1] as usize]);
                        let p2 = Self::transform(&object.transform, &positions[t[2] as usize]);

                        if let Some(d) = triangle_ray(&p0, &p1, &p2, &ray, Some(depth)) {
                            if depth > d {
                                depth = d;
                                id_buffer[y * self.options.frame_size + x] = Some(object_id);
                            }
                        }
                    }
                }
            }
        }

        stats
    }

    /// Takes the 3D vector and transforms it with the given matrix.
    ///
    /// # Arguments
    /// * `v` - The 3D vector to convert.
    #[inline]
    fn transform(m: &Mat3x4, v: &Vec3) -> Vec3 {
        m * Vec4::new(v[0], v[1], v[2], 1.0)
    }
}

impl OcclusionTester for NaiveRaycaster {
    type IndexedSceneType = Scene;

    fn get_name() -> &'static str {
        "naive_raycaster_occ"
    }

    fn new(stats: crate::StatsNode, scene: Scene, options: OccOptions) -> Result<Self> {
        // compute the width == height which is the square root of the number of samples
        let s: usize = options.frame_size;
        let id_buffer = vec![None; s * s];

        Ok(Self {
            stats,
            options,
            scene,
            id_buffer,
        })
    }

    fn compute_visibility(
        &mut self,
        visibility: &mut Visibility,
        frame: Option<&mut Frame>,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> TestStats {
        self.id_buffer.fill(None);
        let stats = self.raycast_data(&view_matrix, &projection_matrix);

        if let Some(frame) = frame {
            frame.get_id_buffer_mut().copy_from_slice(&self.id_buffer);
        }

        self.compute_visibility_internal(visibility);

        stats
    }
}
