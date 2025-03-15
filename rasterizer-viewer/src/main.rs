mod camera;
mod camera_data;
mod options;

use ::rand::{Rng, SeedableRng};
use clap::Parser;
use log::{LevelFilter, info};
use macroquad::{
    color,
    prelude::*,
    texture::{DrawTextureParams, Image, Texture2D, draw_texture_ex},
    window::{clear_background, next_frame},
};
use occ_raycasting::{
    OccOptions, OcclusionTester, Scene, Stats, load_into_scene,
    math::{AABB, mat3x4_to_mat4, transform_vec3},
    rasterizer_culler::RasterizerCuller,
};
use options::Options;
use rand_chacha::ChaCha8Rng;

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

/// Generate and returns the specified number of random colors.
/// Repeated calls always return the same colors
///
/// # Arguments
/// * `num_colors` - The number of colors to generate
pub fn gen_random_colors(num_colors: usize) -> Vec<[u8; 3]> {
    let mut r = ChaCha8Rng::seed_from_u64(2);

    (0..num_colors)
        .map(move |_| {
            [
                r.random_range(0..0x100) as u8,
                r.random_range(0..0x100) as u8,
                r.random_range(0..0x100) as u8,
            ]
        })
        .collect()
}

/// Generates an image from the id buffer.
///
/// # Arguments
/// * `in_ids` - The id buffer
/// * `out_pixels_rgba` - The output image buffer in RGBA format.
/// * `palette` - The color palette to use to map the ids to colors.
fn gen_image_from_id_buffer(
    in_ids: &[Option<u32>],
    out_pixels_rgba: &mut [[u8; 4]],
    palette: &[[u8; 3]],
) {
    // resize the image buffer if necessary
    let num_pixels = in_ids.len();
    assert_eq!(out_pixels_rgba.len(), num_pixels);

    // fill pixel in the image buffer

    for (id, pixel) in in_ids.iter().zip(out_pixels_rgba.iter_mut()) {
        let color = match id {
            Some(id) => palette[(id % palette.len() as u32) as usize],
            None => [0, 0, 0],
        };

        pixel[0] = color[0];
        pixel[1] = color[1];
        pixel[2] = color[2];
        pixel[3] = 0xFF;
    }
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
    let num_objects = scene.objects.len();
    let volume = compute_scene_volume(&scene);
    info!("Number of objects: {}", num_objects);
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

    let mut image = Image::gen_image_color(options.size as u16, options.size as u16, color::BLACK);
    let texture = Texture2D::from_image(&image);
    let palette = gen_random_colors(num_objects);

    loop {
        clear_background(color::BLACK);

        rasterizer.clear();
        rasterizer.rasterize_data(
            camera.get_data().get_model_matrix(),
            camera.get_data().get_projection_matrix(),
        );

        let r = rasterizer.get_rasterizer();
        gen_image_from_id_buffer(r.id_buffer.as_slice(), image.get_image_data_mut(), &palette);
        texture.update(&image);

        draw_texture_ex(
            &texture,
            0f32,
            0f32,
            color::WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        next_frame().await
    }
}
