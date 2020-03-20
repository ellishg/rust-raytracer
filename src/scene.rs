use cgmath::{Deg, Matrix4, SquareMatrix};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use super::camera::Camera;
use super::color::Color;
use super::light::Light;
use super::material::{Material, MaterialType, TextureType};
use super::object::Object;

pub fn load_basic() -> (Vec<Object>, Vec<Light>) {
    let mut objects = vec![];
    let mut lights = vec![];

    let texture = TextureType::new_texture("media/texture.png").unwrap();

    // Create a textured plane.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let object = Object::new_quad(
        (-5.0, -1.0, 5.0).into(),
        (5.0, -1.0, 5.0).into(),
        (5.0, -1.0, -5.0).into(),
        (-5.0, -1.0, -5.0).into(),
        Material::new(phong, texture.clone()),
    );
    objects.push(object);

    // Create a textured sphere.
    let phong = MaterialType::new_phong(0.4, 0.6, 1.8);
    let object = Object::new_sphere(
        (1.0, 0.5, -2.0).into(),
        1.5,
        Material::new(phong, texture.clone()),
    );
    let rotate = Matrix4::from_angle_x(Deg(90.0));
    let translate = Matrix4::from_translation((1.0, 0.5, -2.0).into());
    let object_to_world = translate * rotate * translate.invert().unwrap();
    let object = object.transform(object_to_world);
    objects.push(object);

    // Create a mirror sphere
    let mirror = MaterialType::Reflective;
    let phong = MaterialType::new_phong(0.4, 0.6, 1.8);
    let material_type = MaterialType::Composition(vec![(mirror, 0.4), (phong, 0.6)]);
    let color = TextureType::new_flat(Color::blue());
    let object = Object::new_sphere(
        (-2.0, 0.0, -2.0).into(),
        1.0,
        Material::new(material_type, color),
    );
    objects.push(object);

    // Create a transparent sphere
    // The index of refraction for glass is about 1.69.
    let transparent = MaterialType::Refractive(1.3);
    let phong = MaterialType::new_phong(0.4, 0.6, 1.8);
    let material_type = MaterialType::Composition(vec![(transparent, 0.8), (phong, 0.2)]);
    let color = TextureType::new_flat(Color::green());
    let object = Object::new_sphere(
        (1.0, -0.25, 1.0).into(),
        0.75,
        Material::new(material_type, color),
    );
    objects.push(object);

    // Create a yellow triangle.
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat(Color::yellow());
    let object = Object::new_triangle(
        (-2.0, 0.0, 1.0).into(),
        (-1.0, 0.0, 1.0).into(),
        (-2.0, 2.0, 1.0).into(),
        Material::new(phong, color),
    );
    objects.push(object);

    let light = Light::new_point((2.0, 3.0, 0.5).into(), (1.0, 1.0, 1.0).into());
    lights.push(light);

    let light = Light::new_cone(
        (-1.5, 2.0, 0.0).into(),
        (1.0, -1.0, 0.0).into(),
        Deg(50.0),
        Color::red(),
    );
    lights.push(light);

    let light = Light::new_cone(
        (1.5, 2.0, 0.0).into(),
        (-1.0, -1.0, 0.0).into(),
        Deg(50.0),
        Color::blue(),
    );
    lights.push(light);

    let light = Light::new_cone(
        (0.0, 2.0, 1.5).into(),
        (0.0, -1.0, -1.0).into(),
        Deg(50.0),
        Color::green(),
    );
    lights.push(light);

    let light = Light::new_ambient(Color::grayscale(0.2));
    lights.push(light);

    (objects, lights)
}

/// loads `num_spheres` randomly placed on a plane with a single light source
pub fn load_random_spheres(num_spheres: u16) -> (Vec<Object>, Vec<Light>) {
    let mut objects = vec![];
    let mut lights = vec![];

    // ground plane
    let phong = MaterialType::new_phong(1.0, 0.0, 0.2);
    let color = TextureType::new_flat(Color::grayscale(0.3));
    let object = Object::new_quad(
        (-100.0, 0.0, 10.0).into(),
        (10.0, 0.0, 10.0).into(),
        (10.0, 0.0, -10.0).into(),
        (-100.0, 0.0, -10.0).into(),
        Material::new(phong, color),
    );
    objects.push(object);

    // back plane
    let back_pos = -8.0;
    let height = 4.;
    let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
    let color = TextureType::new_flat(Color::blue() * Color::grayscale(0.5));
    let object = Object::new_quad(
        (10.0, -1.0, back_pos).into(),
        (10.0, height, back_pos).into(),
        (-10.0, height, back_pos).into(),
        (-10.0, -1.0, back_pos).into(),
        Material::new(phong, color),
    );
    objects.push(object);

    let mut rng = StdRng::seed_from_u64(248);
    for _ in 0..num_spheres {
        let phong = MaterialType::new_phong(1.0, 0.0, 1.0);
        let r: f32 = rng.gen_range(0.5, 1.);
        let g: f32 = rng.gen_range(0.5, 1.);
        let b: f32 = rng.gen_range(0.5, 1.);
        let color = TextureType::new_flat(Color::rgb(r, g, b));

        let x: f32 = rng.gen_range(-4., 4.0);
        let y: f32 = rng.gen_range(0.1, 0.2);
        let z: f32 = rng.gen_range(-6., 3.5);
        let r: f32 = rng.gen_range(0.05, y);

        let object = Object::new_sphere((x, y, z).into(), r, Material::new(phong, color));
        objects.push(object);
    }

    // add some reflective spheres in the middle
    let mut add_sphere = |point, color| {
        let phong = MaterialType::new_phong(1.0, 0.6, 1.0);
        let mirror = MaterialType::Reflective;
        let object = Object::new_sphere(
            point,
            0.5,
            Material::new(
                MaterialType::Composition(vec![(mirror, 0.6), (phong, 0.4)]),
                TextureType::new_flat(color),
            ),
        );
        objects.push(object);
    };
    add_sphere((0., 0.5, 0.).into(), Color::red());
    add_sphere((1.5, 0.5, 0.5).into(), Color::blue());
    add_sphere((-1.5, 0.5, -0.5).into(), Color::yellow());

    let light = Light::new_point((1.0, 2.0, 2.5).into(), Color::white());
    lights.push(light);
    let light = Light::new_point((-2.0, 2.0, 1.).into(), Color::white());
    lights.push(light);
    let light = Light::new_ambient(Color::grayscale(0.1));
    lights.push(light);
    let light = Light::new_directional((-0.2, -1., -0.9).into(), Color::grayscale(0.3));
    lights.push(light);

    (objects, lights)
}

pub fn load_suzanne() -> (Vec<Object>, Vec<Light>) {
    let mut objects = vec![];

    let mirror = MaterialType::Reflective;
    let phong = MaterialType::new_phong(0.2, 0.8, 1.0);
    let color = TextureType::new_flat(Color::red());
    let translation = Matrix4::from_translation((0.0, 0.0, 0.0).into());
    let scale = Matrix4::from_nonuniform_scale(1.0, 1.0, 1.0);
    let rotation = Matrix4::from_angle_y(Deg(20.0))
        * Matrix4::from_angle_z(Deg(35.0))
        * Matrix4::from_angle_x(Deg(-20.0));
    let object_to_world = translation * scale * rotation;
    let mesh = Object::new_mesh(
        "media/Suzanne.obj",
        object_to_world,
        Material::new(
            MaterialType::Composition(vec![(mirror, 0.6), (phong, 0.4)]),
            color,
        ),
    )
    .unwrap();
    for triangle in mesh {
        objects.push(triangle);
    }

    (objects, vec![])
}

/// Creates a camera in a default location with a square viewport
/// with side length `pixel_width`.
pub fn default_camera(pixel_width: u32) -> Camera {
    Camera::new(
        pixel_width,
        pixel_width,
        (0.0, 1.5, 5.0).into(),
        (0.0, 0.0, 0.0).into(),
        (0.0, 1.0, 0.0).into(),
    )
}
