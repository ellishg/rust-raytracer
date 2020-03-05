#[macro_use]
extern crate log;

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
use world::World;
use scene::*;

fn main() {
    env_logger::init();
    let mut world = World::new(default_camera(), Color::grayscale(0.2));
    load_basic(&mut world);
    // load_suzanne(&mut world);
    world.render("out.png", 1).unwrap();
}
