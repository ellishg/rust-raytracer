use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix4, Point3, Vector3};
use image;
use std::error::Error;
use std::fs::File;

// `f32.clamp` is nightly-only :(
fn clamp(x: f32, low: f32, high: f32) -> f32 {
    if x < low {
        low
    } else if x > high {
        high
    } else {
        x
    }
}

#[derive(Debug, Copy, Clone)]
struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color::rgba(r, g, b, 1.0)
    }

    fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        let r = clamp(r, 0.0, 1.0);
        let g = clamp(g, 0.0, 1.0);
        let b = clamp(b, 0.0, 1.0);
        let a = clamp(a, 0.0, 1.0);
        Color { r, g, b, a }
    }

    fn get_rgb(&self) -> (u8, u8, u8) {
        (
            (self.r * 255.0) as u8,
            (self.g * 255.0) as u8,
            (self.b * 255.0) as u8,
        )
    }
}

impl std::ops::Add for Color {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        // TODO: How should we add colors?
        Color::rgba(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b,
            self.a + rhs.a,
        )
    }
}

impl std::ops::Mul for Color {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Color::rgba(
            self.r * rhs.r,
            self.g * rhs.g,
            self.b * rhs.b,
            self.a * rhs.a,
        )
    }
}
impl std::ops::Mul<f32> for Color {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self {
        // TODO: How should we multiply colors?
        Color::rgba(self.r * rhs, self.g * rhs, self.b * rhs, self.a * rhs)
    }
}
impl std::ops::Mul<Color> for f32 {
    type Output = Color;
    fn mul(self, rhs: Color) -> Color {
        rhs * self
    }
}

#[derive(Debug, Copy, Clone)]
struct Ray {
    position: Point3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    fn new(position: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray {
            position,
            direction: direction.normalize(),
        }
    }

    fn transform_using(&self, transform: Matrix4<f32>) -> Ray {
        Ray {
            position: transform.transform_point(self.position),
            direction: transform.transform_vector(self.direction),
        }
    }

    fn get_point_on_ray(&self, t: f32) -> (f32, f32, f32) {
        let p = self.position + t * self.direction;
        p.into()
    }
}

// TODO: Move each struct to their own file
struct World {
    camera: Camera,
    objects: Vec<Box<dyn Object>>,
    lights: Vec<Light>,
    background_color: Color,
}

impl World {
    fn new(camera: Camera, background_color: Color) -> World {
        World {
            camera,
            objects: vec![],
            lights: vec![],
            background_color,
        }
    }

    fn add_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    // TODO: Add options to control up sampling and down sampling
    fn render(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(filename)?;
        let png_encoder = image::png::PNGEncoder::new(file);
        let width = self.camera.width;
        let height = self.camera.height;
        let data: Vec<u8> = (0..width * height)
            .into_iter()
            .flat_map(|i| {
                let ray = self.camera.generate_ray(i % width, i / width);
                let color = self.trace_ray(ray);
                let (r, g, b) = color.get_rgb();
                vec![r, g, b]
            })
            .collect();
        png_encoder.encode(data.as_slice(), width, height, image::ColorType::Rgb8)?;
        Ok(())
    }

    fn trace_ray(&self, ray: Ray) -> Color {
        // Transform ray into world space.
        let ray = ray.transform_using(self.camera.camera_to_world);
        let closest_object = self
            .objects
            .iter()
            .filter_map(|object| match object.get_intersection(ray) {
                Some(t) => Some((object, t)),
                None => None,
            })
            // Just a hacky way to find the smallest t value.
            .min_by(|(_, t_left), (_, t_right)| {
                t_left
                    .partial_cmp(t_right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        if let Some((object, t)) = closest_object {
            // Compute the color of the object that the ray first hits.
            object.get_color(self, ray, t)
        } else {
            // If the ray hits nothing, return the background color.
            self.background_color
        }
    }
}

struct Camera {
    camera_to_world: Matrix4<f32>,
    width: u32,
    height: u32,
}

impl Camera {
    fn new(width: u32, height: u32, eye: Point3<f32>, at: Point3<f32>, up: Vector3<f32>) -> Camera {
        let world_to_camera = Matrix4::look_at(eye, at, up);
        let camera_to_world = Transform::inverse_transform(&world_to_camera).unwrap();
        Camera {
            width,
            height,
            camera_to_world,
        }
    }

    fn generate_ray(&self, pixel_x: u32, pixel_y: u32) -> Ray {
        // TODO: This only works for a square screen
        let x = (pixel_x as f32) / (self.width as f32) - 0.5;
        let y = (pixel_y as f32) / (self.height as f32) - 0.5;
        let dist = -1.0; // TODO: Something something focal length?
        let position = (x, y, dist).into();
        let direction = (x, y, dist).into();
        Ray::new(position, direction)
    }
}

// TODO: Make Material enum or trait.

// NOTE: Another option would be to make Object hold an enum of all possible types.
trait Object {
    fn get_intersection(&self, ray: Ray) -> Option<f32>;
    fn get_color(&self, world: &World, ray: Ray, t: f32) -> Color;
}

struct Sphere {
    world_to_object: Matrix4<f32>,
    color: Color,
}

impl Sphere {
    // FIXME: I don't think radius works correctly.
    fn new(position: Point3<f32>, radius: f32, color: Color) -> Box<dyn Object> {
        let scale = Matrix4::from_scale(radius);
        let translate = Matrix4::from_translation(position.to_homogeneous().truncate());
        let object_to_world = translate * scale;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        Box::new(Sphere {
            world_to_object,
            color,
        })
    }
}

impl Object for Sphere {
    fn get_intersection(&self, ray: Ray) -> Option<f32> {
        let ray = ray.transform_using(self.world_to_object);
        let position = ray.position.to_homogeneous().truncate();
        let direction = ray.direction;
        // Sphere is centered at origin with radius 1 (thanks to the matrix transformations).
        let closest_point_to_origin = position - InnerSpace::dot(position, direction) * direction;
        let dist_to_origin = InnerSpace::magnitude(closest_point_to_origin);
        if dist_to_origin <= 1.0 {
            let t = -InnerSpace::dot(position, direction);
            // TODO: Is this correct?
            let delta = (1.0 - dist_to_origin).sqrt();
            // Find the smallest positive t value.
            [t - delta, t + delta]
                .iter()
                .filter(|t| t.is_sign_positive())
                // TODO: This could probably be simplified.
                .map(|t| *t)
                .min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
        } else {
            None
        }
    }

    fn get_color(&self, world: &World, ray: Ray, t: f32) -> Color {
        let normal = {
            let ray = ray.transform_using(self.world_to_object);
            let normal: Vector3<f32> = InnerSpace::normalize(ray.get_point_on_ray(t).into());
            // Transform normal back to world space.
            InnerSpace::normalize(
                self.world_to_object
                    .inverse_transform_vector(normal)
                    .unwrap(),
            )
        };
        let light_color = world
            .lights
            .iter()
            .map(|light| {
                let intersection_point: Point3<f32> = ray.get_point_on_ray(t).into();
                let light_vector = intersection_point - light.position;
                let light_direction = InnerSpace::normalize(light_vector);
                let falloff = 1.0 / (0.001 + InnerSpace::magnitude2(light_vector));
                let intensity = clamp(
                    falloff * InnerSpace::dot(-light_direction, normal),
                    0.0,
                    1.0,
                );
                intensity * light.color
            })
            .fold(Color::rgba(0.0, 0.0, 0.0, 0.0), |acc, x| acc + x);
        self.color * light_color
    }
}

// TODO: Use enum or trait to define light types.
struct Light {
    position: Point3<f32>,
    color: Color,
}

impl Light {
    fn new(position: Point3<f32>, color: Color) -> Light {
        Light { position, color }
    }
}

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
    let object = Sphere::new((1.0, 0.5, 0.0).into(), 1.001, Color::rgb(0.0, 1.0, 0.0));
    world.add_object(object);
    // TODO: Fix coordinate system. I think +x is to the left?
    let light = Light::new((-1.0, -1.0, -1.5).into(), Color::rgb(1.0, 1.0, 1.0));
    world.add_light(light);

    world.render("foo.png").unwrap();
}
