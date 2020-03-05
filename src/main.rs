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
use material::{Material, MaterialType, TextureType};
use object::Object;
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

    let texture = TextureType::new_texture("media/texture.png").unwrap();

    // Create a textured plane.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let object = Object::new_plane(
        (0.0, -1.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        Material::new(phong, texture.clone()),
    );
    world.add_object(object);

    // Create a red sphere.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat((1.0, 0.0, 0.0).into());
    let object = Object::new_sphere((0.0, 0.0, 0.0).into(), 1.0, Material::new(phong, color));
    world.add_object(object);

    // Create a textured sphere.
    let phong = MaterialType::new_phong(0.4, 0.6, 1.8);
    let object = Object::new_sphere(
        (1.0, 0.5, -2.0).into(),
        1.5,
        Material::new(phong, texture.clone()),
    );
    world.add_object(object);

    // Create a yellow triangle.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat((1.0, 1.0, 0.0).into());
    let object = Object::new_triangle(
        (-2.0, 0.0, 1.0).into(),
        (-1.0, 0.0, 1.0).into(),
        (-2.0, 2.0, 1.0).into(),
        Material::new(phong, color),
    );
    world.add_object(object);

    // Create a textured triangle.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let object = Object::new_triangle(
        (-0.5, 1.0, 1.0).into(),
        (0.5, 1.0, 1.0).into(),
        (0.0, 2.0, 1.0).into(),
        Material::new(phong, texture.clone()),
    );
    world.add_object(object);

    let light = Light::new((1.0, 2.0, 2.5).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);
    let light = Light::new((-4.0, 2.0, 2.0).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);

    world.render("out.png").unwrap();
}
