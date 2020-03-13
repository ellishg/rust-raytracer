#[macro_use]
extern crate log;

mod bvh;
mod camera;
mod color;
mod light;
mod material;
mod object;
mod ray;
mod scene;
mod utils;
mod world;

use color::Color;
use scene::*;
use world::render;

fn main() {
    env_logger::init();
    let mut objects = vec![];
    let mut lights = vec![];

    let (new_objects, new_lights) = load_basic();
    objects.extend(new_objects);
    lights.extend(new_lights);

    let (new_objects, new_lights) = load_suzanne();
    objects.extend(new_objects);
    lights.extend(new_lights);

    // let (new_objects, new_lights) = load_random_spheres(30);
    // objects.extend(new_objects);
    // lights.extend(new_lights);

    render(
        default_camera(),
        objects,
        lights,
        Color::grayscale(0.2),
        1,  // samples_per_pixel
        10, // max_ray_bounces
        "out.png",
    )
    .unwrap();
}
