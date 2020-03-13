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

use clap::{App, Arg};

fn main() {
    env_logger::init();

    let cl_args = App::new("Rust Ray Tracer")
        .version("0.1")
        .author("Ellis Hoag <ellis.sparky.hoag@gmail.com>, Leo Mehr <leomehr@stanford.edu>")
        .arg(
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .help("File to save rendered image as .png")
                .required(false)
                .default_value("out.png")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("threads")
                .short("t")
                .long("threads")
                .help("Number of threads to use")
                .required(false)
                .default_value("4"),
        )
        .arg(
            Arg::with_name("samples_per_pixel")
                .short("s")
                .long("samples_per_pixel")
                .help("Number of rays to cast per pixel")
                .required(false)
                .default_value("1"),
        )
        .arg(
            Arg::with_name("max_ray_bounces")
                .short("b")
                .long("max_ray_bounces")
                .help("Max number of times to bounce a ray. Used for reflection and refraction.")
                .required(false)
                .default_value("10"),
        )
        .get_matches();

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

    let samples_per_pixel = cl_args
        .value_of("samples_per_pixel")
        .unwrap()
        .parse()
        .unwrap();
    let max_ray_bounces = cl_args
        .value_of("max_ray_bounces")
        .unwrap()
        .parse()
        .unwrap();
    let num_threads = cl_args.value_of("threads").unwrap().parse().unwrap();

    render(
        default_camera(),
        objects,
        lights,
        Color::grayscale(0.2),
        samples_per_pixel,
        max_ray_bounces,
        cl_args.value_of("file").unwrap(),
        num_threads,
    )
    .unwrap();
}
