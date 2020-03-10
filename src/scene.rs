use cgmath::{Deg, Matrix4, Transform};
use rand::Rng;

use super::color::Color;
use super::camera::Camera;
use super::light::Light;
use super::object::Object;
use super::material::{Material, MaterialType, TextureType};
use super::world::World;

pub fn load_basic(world: &mut World) -> () {
    let texture = TextureType::new_texture("media/texture.png").unwrap();

    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let object = Object::new_plane(
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        Material::new(phong, texture.clone()),
    );
    world.add_object(object);

    let phong = MaterialType::new_phong(0.5, 0.5, 1.0);
    let color = TextureType::new_flat(Color::green());
    let object = Object::new_plane(
        (0.0, 0.0, -4.0).into(),
        (0.0, 0.0, 1.0).into(),
        Material::new(phong, color),
    );
    world.add_object(object);

    let phong = MaterialType::new_phong(0.4, 0.6, 1.8);
    let object = Object::new_sphere(
        (1.0, 0.5, -2.0).into(),
        1.5,
        Material::new(phong, texture.clone()),
    );
    world.add_object(object);

    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat(Color::yellow());
    let object = Object::new_triangle(
        (-2.0, 0.0, 1.0).into(),
        (-1.0, 0.0, 1.0).into(),
        (-2.0, 2.0, 1.0).into(),
        Material::new(phong, color),
    );
    world.add_object(object);

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
}

/// loads `num_spheres` randomly placed on a plane with a single light source
pub fn load_random_spheres(world: &mut World, num_spheres: u16) {
    // ground plane
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat(Color::grayscale(0.2));
    let object = Object::new_plane(
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
        Material::new(phong, color),
    );
    world.add_object(object);

    // back plane
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat(Color::grayscale(0.8));
    let object = Object::new_plane(
        (0.0, 0.0, -3.0).into(),
        (0.0, 0.0, 1.0).into(),
        Material::new(phong, color),
    );
    world.add_object(object);

    let mut rng = rand::thread_rng();
    for _ in 0..num_spheres {
        let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
        let r: f32 = rng.gen_range(0.2, 1.);
        let g: f32 = rng.gen_range(0.2, 1.);
        let b: f32 = rng.gen_range(0.2, 1.);
        let a: f32 = rng.gen_range(0.5, 1.);
        let color = TextureType::new_flat(Color::rgba(r, g, b, a));

        let x: f32 = rng.gen_range(-2.5, 2.5);
        let y: f32 = rng.gen_range(0., 2.);
        let z: f32 = rng.gen_range(-3., 3.);
        let r: f32 = rng.gen_range(0.05, 0.2);

        let object = Object::new_sphere(
            (x, y, z).into(),
            r,
            Material::new(phong, color),
        );
        world.add_object(object);
    }

    let light = Light::new((1.0, 2.0, 2.5).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);
    let light = Light::new((-2.0, 2.0, 1.).into(), (1.0, 1.0, 1.0).into());
    world.add_light(light);
}

pub fn load_suzanne(world: &mut World) {
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

}

pub fn default_camera() -> Camera {
    Camera::new(
        500,
        500,
        (0.0, 1.5, 5.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
    )
}
