use std::time::Instant;

use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use occ_raycasting::{
    load_into_scene, rasterizer::RasterizerCuller, OcclusionTester, Scene, Stats, StatsNodeTrait,
    Visibility,
};
use options::Options;

mod options;

/// Initializes the program logging
///
/// # Arguments
/// * `filter` - The log level filter, i.e., the minimum log level to be logged.
fn initialize_logging(filter: LevelFilter) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    builder.filter_level(filter).init();
}

/// Loads the CAd files based on the provided glob pattern.
///
/// # Arguments
/// * `files` - The glob pattern for the CAD files.
fn load_cad_files_files(files: &str) -> Result<(Scene, usize)> {
    let mut scene = Scene::default();
    let mut num_read_files = 0;

    let paths = match glob::glob(files) {
        Ok(dir) => dir,
        Err(err) => {
            error!("Failed to read directory: {:?}", err);
            return Err(err.into());
        }
    };

    for entry in paths {
        match entry {
            Ok(path) => {
                info!("Loading CAD data '{}'...", path.display());

                if let Err(err) = load_into_scene(&mut scene, &path) {
                    error!("Failed to load CAD data: {:?}", err);
                    info!("Skipping CAD data...");
                } else {
                    num_read_files += 1;
                }
            }
            Err(err) => {
                error!("Failed to read entry: {:?}", err);
                info!("Skipping entry...");
            }
        }
    }

    Ok((scene, num_read_files))
}

/// Prints the scene information.
///
/// # Arguments
/// * `scene` - The scene to print the information for.
fn print_scene_info(scene: &Scene) {
    let mut num_unique_triangles = 0;
    let mut num_unique_vertices = 0;
    for mesh in scene.meshes.iter() {
        num_unique_triangles += mesh.indices.len();
        num_unique_vertices += mesh.vertices.len();
    }

    let mut num_triangles = 0;
    let mut num_vertices = 0;
    for object in scene.objects.iter() {
        let mesh = &scene.meshes[object.mesh_index as usize];
        num_triangles += mesh.indices.len();
        num_vertices += mesh.vertices.len();
    }

    info!("Scene information:");
    info!("  - Number of unique triangles: {}", num_unique_triangles);
    info!("  - Number of unique vertices: {}", num_unique_vertices);
    info!("  - Number of triangles: {}", num_triangles);
    info!("  - Number of vertices: {}", num_vertices);
}

/// Runs the program.
///
/// # Arguments
/// * `options` - The program options.
fn run_program(options: Options) -> anyhow::Result<()> {
    let s = Stats::root();
    let t_ = Instant::now();
    let scene = {
        let _t = s.get_child("loading").register_timing();

        let (scene, num_read) = load_cad_files_files(&options.input_files).map_err(|err| {
            error!("Failed to load CAD data: {:?}", err);
            err
        })?;

        info!(
            "Loaded {} CAD files in {} ms",
            num_read,
            t_.elapsed().as_secs_f64() * 1e3f64
        );

        scene
    };

    print_scene_info(&scene);

    match options.occ {
        options::OccTestSubcommand::Rasterizer(options) => {
            let _t = s.get_child("rasterizer").register_timing();
            let options = occ_raycasting::OccOptions {
                num_threads: 1,
                num_samples: options.image_size * options.image_size,
            };
            let mut c =
                RasterizerCuller::new(s.get_child("culling"), &scene, options).map_err(|err| {
                    error!("Failed to create rasterizer culler: {:?}", err);
                    err
                })?;

            // TODO: Implement the actual culling.
        }
    }

    Ok(())
}

fn main() {
    let options = Options::parse();
    initialize_logging(options.log_level.into());
    options.dump_to_log();

    match run_program(options) {
        Ok(_) => {
            info!("Stat:");
            info!("{}", format!("{}", *Stats::root().lock().unwrap()));
            info!("Program completed successfully");
        }
        Err(err) => {
            error!("Program failed: {:?}", err);
            std::process::exit(1);
        }
    }
}
