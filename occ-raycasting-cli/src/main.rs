use std::{fs::File, io::BufReader, path::Path, time::Instant};

use anyhow::Result;
use clap::Parser;
use log::{error, info, LevelFilter};
use occ_raycasting::{load_into_scene, Scene, Stats, StatsNode, StatsNodeTrait, TestConfig};
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

/// Loads the CAD files based on the provided glob pattern into the scene.
///
/// # Arguments
/// * `scene` - The scene to load the CAD files into.
/// * `files` - The glob pattern for the CAD files.
fn load_cad_files_files(scene: &mut Scene, files: &str) -> Result<usize> {
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

                if let Err(err) = load_into_scene(scene, &path) {
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

    Ok(num_read_files)
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

/// Loads the test configuration from the provided path.
///
/// # Arguments
/// * `path` - The path to the configuration file.
fn load_config<P: AsRef<Path>>(path: P) -> Result<TestConfig> {
    let file = File::open(path).map_err(|err| {
        error!("Failed to open file: {:?}", err);
        err
    })?;

    let config = TestConfig::read(BufReader::new(file)).map_err(|err| {
        error!("Failed to read config: {:?}", err);
        err
    })?;

    info!("Loaded config: {:?}", config);

    Ok(config)
}

/// Loads the scene from the provided input files.
///
/// # Arguments
/// * `s` - The stats node to register the timing with.
/// * `input` - The input files to load the scene from.
fn load_scene(s: StatsNode, input: &[String]) -> Result<Scene> {
    let mut scene = Scene::default();

    let t_ = Instant::now();
    let _t = s.get_child("loading").register_timing();

    let mut num_read = 0;
    for f in input.iter() {
        num_read += load_cad_files_files(&mut scene, f).map_err(|err| {
            error!("Failed to load CAD data: {:?}", err);
            err
        })?;
    }

    info!(
        "Loaded {} CAD files in {} ms",
        num_read,
        t_.elapsed().as_secs_f64() * 1e3f64
    );

    Ok(scene)
}

/// Runs the program.
///
/// # Arguments
/// * `options` - The program options.
fn run_program(options: Options) -> anyhow::Result<()> {
    let s = Stats::root();

    let config = load_config(&options.config)?;
    let scene = load_scene(s.get_child("scene"), &config.input).map_err(|err| {
        error!("Failed to load scene: {:?}", err);
        err
    })?;

    print_scene_info(&scene);

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
