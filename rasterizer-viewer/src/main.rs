mod camera;
mod camera_data;
mod options;

use clap::Parser;
use log::{LevelFilter, info};
use macroquad::{
    color,
    window::{clear_background, next_frame},
};
use occ_raycasting::{
    OccOptions, OcclusionTester, Scene, Stats, load_into_scene,
    math::{AABB, mat3x4_to_mat4, transform_vec3},
    rasterizer_culler::RasterizerCuller,
};
use options::Options;

/// Initializes the program logging
///
/// # Arguments
/// * `filter` - The log level filter, i.e., the minimum log level to be logged.
fn initialize_logging(filter: LevelFilter) {
    let mut builder = pretty_env_logger::formatted_timed_builder();

    builder.filter_level(filter).init();
}

fn compute_scene_volume(scene: &Scene) -> AABB {
    let mut volume = AABB::default();

    for object in scene.objects.iter() {
        let t = object.transform;
        let t = mat3x4_to_mat4(&t);
        let m = &scene.meshes[object.mesh_index as usize];

        volume.extend_iter(m.vertices.iter().map(|v| transform_vec3(&t, v)));
    }

    volume
}

#[macroquad::main("Rasterizer Viewer")]
async fn main() {
    let options = Options::parse();
    initialize_logging(options.log_level.into());
    options.dump_to_log();

    let mut scene = Scene::default();
    info!("Load CAD file {:?}...", options.input);
    if let Err(err) = load_into_scene(&mut scene, &options.input) {
        info!("Failed to load CAD file: {:?}", err);
        std::process::exit(1);
    }
    info!("Load CAD file {:?}...DONE", options.input);

    info!("Computing scene volume...");
    let volume = compute_scene_volume(&scene);
    info!("Scene volume: {:?}", volume);

    let occ_options = OccOptions {
        frame_size: options.size,
        num_threads: 1,
    };
    let mut rasterizer = match RasterizerCuller::new(Stats::root(), scene, occ_options) {
        Ok(r) => r,
        Err(err) => {
            info!("Failed to create rasterizer culler: {:?}", err);
            std::process::exit(1);
        }
    };

    let mut camera = camera::Camera::new();
    camera.focus(&volume).unwrap();

    loop {
        clear_background(color::BLACK);

        rasterizer.clear();
        rasterizer.rasterize_data(
            camera.get_data().get_model_matrix(),
            camera.get_data().get_projection_matrix(),
        );

        

        next_frame().await
    }
}
