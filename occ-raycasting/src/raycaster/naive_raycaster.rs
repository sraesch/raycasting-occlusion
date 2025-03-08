use log::{error, trace};
use nalgebra_glm::{vec4_to_vec3, Mat3x4, Mat4, Vec3, Vec4};
use serde::{Deserialize, Serialize};

use crate::{
    math::{extract_camera_pos_from_view_matrix, triangle_ray, Ray, AABB},
    rasterizer_culler::Frame,
    spatial::RayIntersectionTest,
    utils::compute_visibility_from_id_buffer,
    IndexedScene, OccOptions, OcclusionTester, Result, Scene, StatsNode, StatsNodeTrait, TestStats,
    Visibility,
};

/// A very simple ray caster without any acceleration structures
pub struct NaiveRaycaster {
    stats: StatsNode,
    options: OccOptions,
    scene: Scene,
    scene_volumes: Vec<AABB>,

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
        let x0 = extract_camera_pos_from_view_matrix(view_matrix);

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
                    let scene_volume = &self.scene_volumes[object_id];
                    let object_id = object_id as u32;

                    stats.num_volume_tests += 1;
                    if scene_volume.intersects_ray(&ray, Some(depth)).is_none() {
                        continue;
                    }

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
    type IndexedSceneType = SceneWithVolumes;

    fn get_name() -> &'static str {
        "naive_raycaster_occ"
    }

    fn new(
        stats: crate::StatsNode,
        scene_with_volumes: SceneWithVolumes,
        options: OccOptions,
    ) -> Result<Self> {
        // compute the width == height which is the square root of the number of samples
        let s: usize = options.frame_size;
        let id_buffer = vec![None; s * s];

        let scene = scene_with_volumes.scene;
        let scene_volumes = scene_with_volumes.volumes;

        Ok(Self {
            stats,
            options,
            scene,
            scene_volumes,
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

/// An indexed and optimized scene data used for occlusion testing.
#[derive(Serialize, Deserialize)]
pub struct SceneWithVolumes {
    scene: Scene,
    volumes: Vec<AABB>,
}

impl IndexedScene for SceneWithVolumes {
    fn from_read<R: std::io::Read>(reader: R) -> Result<Self> {
        let result: Self = bincode::deserialize_from(reader)
            .map_err(|e| crate::Error::DeserializationError(Box::new(e)))?;

        Ok(result)
    }

    fn write<W: std::io::Write>(&self, writer: W) -> Result<()> {
        bincode::serialize_into(writer, self)
            .map_err(|e| crate::Error::SerializationError(Box::new(e)))
    }

    fn build_acceleration_structures(scene: Scene, progress: crate::ProgressCallback) -> Self {
        let num_objects = scene.objects.len();

        let mut last_update: i32 = -1i32;
        let volumes: Vec<AABB> = scene
            .objects
            .iter()
            .enumerate()
            .map(|(i, object)| {
                let mesh = &scene.meshes[object.mesh_index as usize];
                let positions = &mesh.vertices;

                // compute the progress
                let p0 = (i * 100 / num_objects) as i32;
                if p0 != last_update {
                    last_update = p0;

                    let p = i as f32 * 100f32 / num_objects as f32;

                    progress(0, 1, p, "Computing bounding volumes...");
                }

                let aabb = AABB::from_iter(
                    positions
                        .iter()
                        .map(|p| object.transform * Vec4::new(p[0], p[1], p[2], 1.0)),
                );

                trace!("AABB: {:?} for object ID={}", aabb, i);

                aabb
            })
            .collect();

        progress(0, 1, 100f32, "Computing bounding volumes...DONE");

        SceneWithVolumes { scene, volumes }
    }
}
