use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix4, Point3, Vector3};

use super::color::Color;
use super::ray::Ray;
use super::utils::clamp;
use super::world::World;

// TODO: Make Material enum or trait.

// NOTE: Another option would be to make Object hold an enum of all possible types.
pub trait Object {
    fn get_intersection(&self, ray: Ray) -> Option<f32>;
    fn get_color(&self, world: &World, ray: Ray, t: f32) -> Color;
}

pub struct Sphere {
    world_to_object: Matrix4<f32>,
    color: Color,
}

impl Sphere {
    // FIXME: I don't think radius works correctly.
    pub fn new(position: Point3<f32>, radius: f32, color: Color) -> Box<dyn Object> {
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