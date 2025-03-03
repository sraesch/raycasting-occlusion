use std::{fs::File, path::PathBuf};

use log::info;

use crate::{
    rasterizer::{gen_random_colors, Frame, RasterizerCuller},
    Error, IndexedScene, OccOptions, OcclusionSetup, OcclusionTester, Result, Scene, StatsNode,
    StatsNodeTrait, TestConfig, Visibility,
};

/// A test executor for running the occlusion tests.
pub struct TestExecutor {
    config: TestConfig,
    out_dir: PathBuf,
    scene: Scene,
}

impl TestExecutor {
    /// Creates a new test executor.
    ///
    /// # Arguments
    /// * `config` - The test configuration.
    /// * `scene` - The scene to test.
    /// * `out_dir` - The output directory.
    pub fn new(config: TestConfig, scene: Scene, out_dir: PathBuf) -> Self {
        Self {
            config,
            scene,
            out_dir,
        }
    }

    /// Runs the test executor.
    ///
    /// # Arguments
    /// * `s` - The stats node to write the test results to.
    pub fn run(&self, s: StatsNode) -> Result<()> {
        let num_threads = self.config.num_threads;

        info!("Num Test Setups: {}", self.config.setups.len());
        info!("Num Views: {}", self.config.views.len());

        info!("Initialize the test executor...");
        self.initialize().map_err(|err| {
            log::error!("Failed to initialize the test executor: {:?}", err);
            err
        })?;

        // start iterating over the setups
        for setup in self.config.setups.iter() {
            match setup {
                OcclusionSetup::Rasterizer => {
                    log::info!("Testing rasterizer setup...");
                    let options = OccOptions {
                        frame_size: self.config.frame_size,
                        num_threads,
                    };
                    if let Err(err) = self.test_setup::<RasterizerCuller>(s.clone(), options) {
                        log::error!("Failed to test the rasterizer setup: {:?}", err);
                    }
                }
            }
        }

        Ok(())
    }

    /// Tests the given setup.
    ///
    /// # Arguments
    /// * `setup` - The setup to test.
    /// * `s` - The stats node to write the test results to.
    /// * `options` - The occlusion culling options.
    fn test_setup<T: OcclusionTester>(&self, s: StatsNode, options: OccOptions) -> Result<()> {
        let s = s.get_child(T::get_name());
        let _t = s.register_timing();

        // make sure the directory for the setup exists
        let setup_dir = self.out_dir.join(T::get_name());
        std::fs::create_dir_all(&setup_dir).map_err(|err| {
            log::error!("Failed to create the setup directory: {:?}", err);
            Error::Io(err)
        })?;

        // initialize the input data
        info!("Initializing the input data...");
        let scene_data = {
            let _t2 = s.get_child("initialize").register_timing();
            T::IndexedSceneType::build_acceleration_structures(
                self.scene.clone(),
                Self::print_progress,
            )
        };

        // create the occlusion tester
        let mut tester = T::new(s.clone(), scene_data, options.clone())?;

        // determine if a frame should be written
        let mut frame = if self.config.write_frames {
            Some(Frame::new_empty(
                options.frame_size,
                options.frame_size,
                false,
            ))
        } else {
            None
        };

        // start iterating over the views
        let mut visibility = Visibility::default();
        for (view_index, view) in self.config.views.iter().enumerate() {
            info!(
                "Render view {}/{}...",
                view_index + 1,
                self.config.views.len()
            );

            let view_matrix = view.view_matrix;
            let projection_matrix = view.projection_matrix;

            tester.compute_visibility(
                &mut visibility,
                frame.as_mut(),
                view_matrix,
                projection_matrix,
            );

            if let Some(frame) = frame.as_mut() {
                let frame_path = setup_dir.join(format!("view_{}.png", view_index));
                let writer = match File::create(&frame_path) {
                    Ok(writer) => writer,
                    Err(err) => {
                        log::error!("Failed to create the frame file: {:?}", err);
                        continue;
                    }
                };

                if let Err(err) = frame.write_id_buffer_as_ppm(writer, gen_random_colors) {
                    log::error!("Failed to save the frame: {:?}", err);
                }
            }
        }

        Ok(())
    }

    /// Prints the progress of the current stage.
    ///
    /// # Arguments
    /// * `current_stage` - The current stage.
    /// * `total_stages` - The total number of stages.
    /// * `progress` - The progress of the current stage.
    /// * `msg` - The message to print.
    fn print_progress(current_stage: usize, total_stages: usize, progress: f32, msg: &str) {
        info!(
            "Stage {}/{} ({:.2}%): {}",
            current_stage + 1,
            total_stages,
            progress,
            msg
        );
    }

    /// Initializes the test executor.
    fn initialize(&self) -> Result<()> {
        // make sure the specified output directory exists and is a directory
        std::fs::create_dir_all(&self.out_dir).map_err(|err| {
            log::error!("Failed to create the output directory: {:?}", err);

            Error::Io(err)
        })?;

        Ok(())
    }
}
