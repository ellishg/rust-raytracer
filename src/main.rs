mod camera;
mod color;
mod light;
mod material;
mod object;
mod ray;
mod utils;
mod world;

use camera::Camera;
use light::Light;
use material::{Phong, Texture};
use object::{Plane, Sphere, Triangle};
use std::rc::Rc;
use world::World;

fn main() {
    let camera = Camera::new(
        500,
        500,
        (0.0, 2.0, 6.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
    );
    let mut world = World::new(camera, (0.2, 0.2, 0.2).into());
    let texture = Rc::new(Texture::new("media/texture.png").unwrap());
    let object = Plane::new(
        (0.0, -1.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        Phong::new(
            (0.0, 0.0, 1.0).into(),
            Some(Rc::clone(&texture)),
            1.0,
            0.0,
            1.0,
        ),
    );
    world.add_object(object);
    let object = Sphere::new(
        (0.0, 0.0, 0.0).into(),
        1.0,
        Phong::new((1.0, 0.0, 0.0).into(), None, 0.8, 0.2, 2.0),
    );
    world.add_object(object);
    let object = Sphere::new(
        (1.0, 0.0, -1.0).into(),
        1.0,
        Phong::new(
            (0.0, 1.0, 0.0).into(),
            Some(Rc::clone(&texture)),
            0.4,
            0.6,
            1.8,
        ),
    );
    world.add_object(object);
    let object = Triangle::new(
        (-2.0, 0.0, 1.0).into(),
        (-1.0, 0.0, 1.0).into(),
        (-2.0, 2.0, 1.0).into(),
        Phong::new((1.0, 1.0, 0.0).into(), None, 1.0, 0.0, 1.0),
    );
    world.add_object(object);
    let object = Triangle::new(
        (-0.5, 1.0, 1.0).into(),
        (0.5, 1.0, 1.0).into(),
        (0.0, 2.0, 1.0).into(),
        Phong::new(
            (1.0, 1.0, 0.0).into(),
            Some(Rc::clone(&texture)),
            1.0,
            0.0,
            1.0,
        ),
    );
    world.add_object(object);

    let light = Light::new((2.0, 2.0, 1.5).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);
    let light = Light::new((-5.0, 2.0, 2.0).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);

    world.render("out.png").unwrap();
}
