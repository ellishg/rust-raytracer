mod camera;
mod color;
mod light;
mod object;
mod ray;
mod utils;
mod world;

use camera::Camera;
use color::Color;
use light::Light;
use object::Sphere;
use world::World;

fn main() {
    let camera = Camera::new(
        500,
        500,
        (0.0, 2.0, -10.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
    );
    let mut world = World::new(camera, Color::rgb(0.2, 0.2, 0.2));
    let object = Sphere::new((0.0, 0.0, 0.0).into(), 1.0, Color::rgb(1.0, 0.0, 0.0));
    world.add_object(object);
    let object = Sphere::new((1.0, 0.5, 2.0).into(), 1.0, Color::rgb(0.0, 1.0, 0.0));
    world.add_object(object);
    let light = Light::new((2.0, 2.0, -1.5).into(), Color::rgb(1.0, 1.0, 1.0));
    world.add_light(light);

    world.render("out.png").unwrap();
}
