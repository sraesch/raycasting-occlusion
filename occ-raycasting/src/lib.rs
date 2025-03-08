mod config;
mod error;
mod executor;
mod math;
pub mod rasterizer_culler;
pub mod raycaster;
mod scene;
pub mod spatial;
mod stats;
mod utils;

pub use config::*;
pub use error::*;
pub use executor::*;
use nalgebra_glm::Mat4;
use rasterizer_culler::Frame;
pub use scene::*;
pub use stats::*;

/// A list of the objects with their ids and their visibility. The per object visibility is a value
/// between 0 and 1, where 0 means that the object is not visible and 1 means that the object is
/// fully covering the screen.
/// The list is sorted by the visibility of the objects, where the first element is the most visible
/// object.
pub type Visibility = Vec<(u32, f32)>;

/// The options for an occlusion testing.
#[derive(Clone)]
pub struct OccOptions {
    /// The number of threads to be used for the testing
    pub num_threads: usize,

    /// The size of the occlusion test frame.
    pub frame_size: usize,
}

/// Resulting stats about the occlusion testing.
#[derive(Clone, Copy, Default)]
pub struct TestStats {
    /// The number of triangles processed, i.e., that could not be avoided through acceleration
    /// structures or other means.
    pub num_triangles: usize,
}

impl std::ops::Add<Self> for TestStats {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            num_triangles: self.num_triangles + rhs.num_triangles,
        }
    }
}

impl std::ops::AddAssign<Self> for TestStats {
    fn add_assign(&mut self, rhs: Self) {
        self.num_triangles += rhs.num_triangles;
    }
}

impl Default for OccOptions {
    fn default() -> Self {
        Self {
            num_threads: 1,
            frame_size: 256,
        }
    }
}

/// A general progress callback function to give updates on the progress.
///
/// # Arguments
/// * `current_stage` - The current stage of the progress starting at 0.
/// * `total_stages` - The total number of stages.
/// * `progress` - The progress of the current stage in percent.
/// * `msg` - The message to display.
pub type ProgressCallback = fn(current_stage: usize, total_stages: usize, progress: f32, msg: &str);

/// An indexed and optimized scene data used for occlusion testing.
pub trait IndexedScene: Sized {
    /// Creates a new indexed scene from the given reader.
    ///
    /// # Arguments
    /// * `reader` - The reader to read the scene data from.
    fn from_read<R: std::io::Read>(reader: R) -> Result<Self>;

    /// Writes the indexed scene to the given writer.
    ///
    /// # Arguments
    /// * `writer` - The writer to write the scene data to.
    fn write<W: std::io::Write>(&self, writer: W) -> Result<()>;

    /// Builds the acceleration structures from the scene.
    ///
    /// # Arguments
    /// * `scene`- The scene to build the acceleration structures from.
    fn build_acceleration_structures(scene: Scene, progress: ProgressCallback) -> Self;
}

/// A trait for the occlusion testing.
pub trait OcclusionTester: Sized {
    /// The indexed scene type used for the occlusion testing.
    type IndexedSceneType: IndexedScene;

    /// Creates and returns a new occlusion tester instance.
    ///
    /// # Arguments
    /// * `stats` - The stats node into which the culler registers all its times.
    /// * `scene_data` - The scene data to be used for the occlusion testing.
    /// * `options` - The culler options.
    fn new(
        stats: StatsNode,
        scene_data: Self::IndexedSceneType,
        options: OccOptions,
    ) -> Result<Self>;

    /// Returns the name of the occlusion tester.
    fn get_name() -> &'static str;

    /// Computes a frame using culling and determines the visible ids of the objects.
    ///
    /// # Arguments
    /// * `visibility` - A mutable reference for returning the visibility of the objects.
    /// * `frame` - Optionally a mutable reference onto the frame to return the resulting pixels.
    /// * `view_matrix` - The camera view matrix.
    /// * `projection_matrix` - The camera projection matrix.
    fn compute_visibility(
        &mut self,
        visibility: &mut Visibility,
        frame: Option<&mut Frame>,
        view_matrix: Mat4,
        projection_matrix: Mat4,
    ) -> TestStats;
}

impl IndexedScene for Scene {
    fn from_read<R: std::io::Read>(reader: R) -> Result<Self> {
        bincode::deserialize_from(reader).map_err(|e| Error::DeserializationError(Box::new(e)))
    }

    fn write<W: std::io::Write>(&self, writer: W) -> Result<()> {
        bincode::serialize_into(writer, self).map_err(|e| Error::SerializationError(Box::new(e)))
    }

    fn build_acceleration_structures(scene: Scene, progress: crate::ProgressCallback) -> Self {
        progress(0, 1, 100.032, "Building acceleration structures ... DONE");
        scene
    }
}
