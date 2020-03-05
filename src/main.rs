mod camera;
mod color;
mod light;
mod material;
mod object;
mod ray;
mod utils;
mod world;

use camera::Camera;
use cgmath::{Deg, Matrix4, Transform};
use color::Color;
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

    let mut world = World::new(camera, Color::grayscale(0.2));

    let texture = TextureType::new_texture("media/texture.png").unwrap();

    // Create a textured plane.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let object = Object::new_plane(
        (0.0, -1.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        Material::new(phong, texture.clone()),
    );
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
    let color = TextureType::new_flat(Color::yellow());
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

    // Import a triangle mesh.
    let phong = MaterialType::new_phong(0.2, 0.8, 1.0);
    let color = TextureType::new_flat(Color::red());
    let translation = Matrix4::from_translation((0.0, 0.0, 0.0).into());
    let scale = Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0);
    let rotation = Matrix4::from_angle_y(Deg(20.0)) * Matrix4::from_angle_z(Deg(-45.0));
    let object_to_world = translation * scale * rotation;
    let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
    for triangle in Object::new_mesh(
        "media/Suzanne.obj",
        world_to_object,
        Material::new(phong, color),
    )
    .unwrap()
    {
        world.add_object(triangle);
    }

    let light = Light::new((1.0, 2.0, 2.5).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);
    let light = Light::new((-4.0, 2.0, 2.0).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);

    world.render("out.png", 1).unwrap();
}
