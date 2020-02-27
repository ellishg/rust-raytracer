use cgmath::{InnerSpace, MetricSpace, Transform};
use cgmath::{Matrix3, Matrix4, Point3, Vector3};

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

pub struct Plane {
    world_to_object: Matrix4<f32>,
    color: Color,
}

impl Plane {
    pub fn new(position: Point3<f32>, normal: Vector3<f32>, color: Color) -> Box<Plane> {
        let normal = normal.normalize();
        // Pick an arbitrary orthogonal vector.
        let new_x_axis = {
            let axis: Vector3<f32> = if normal.x != 0.0 {
                (-normal.y / normal.x, 1.0, 0.0).into()
            } else if normal.y != 0.0 {
                (1.0, -normal.x / normal.y, 0.0).into()
            } else {
                (1.0, 0.0, -normal.x / normal.z).into()
            };
            axis.normalize()
        };
        let new_y_axis = normal;
        let new_z_axis = new_x_axis.cross(new_y_axis).normalize();
        let rotate: Matrix4<f32> = Matrix3::from_cols(new_x_axis, new_y_axis, new_z_axis).into();
        let translate = Matrix4::from_translation(position.to_homogeneous().truncate());
        let object_to_world = rotate * translate;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        Box::new(Plane {
            world_to_object,
            color,
        })
    }
}

impl Object for Plane {
    fn get_intersection(&self, ray: Ray) -> Option<f32> {
        let ray = ray.transform_using(self.world_to_object);
        let normal = (0.0, 1.0, 0.0).into();
        let position = ray.position.to_homogeneous().truncate();
        let direction = ray.direction;
        if InnerSpace::dot(direction, normal) == 0.0 {
            None
        } else {
            let t = InnerSpace::dot(-position, normal) / InnerSpace::dot(direction, normal);
            if t > 0.0 {
                Some(t)
            } else {
                None
            }
        }
    }

    fn get_color(&self, world: &World, ray: Ray, t: f32) -> Color {
        let normal = (0.0, 1.0, 0.0).into();
        let light_color = world
            .lights
            .iter()
            .map(|light| {
                // TODO: Move some of this work to World.
                let intersection_point: Point3<f32> = ray.get_point_on_ray(t).into();
                let light_ray = Ray::new(light.position, intersection_point - light.position);
                if let Some((_, t)) = world.get_closest_intersection(light_ray) {
                    if intersection_point.distance(light_ray.get_point_on_ray(t).into()) > 0.1 {
                        return Color::rgba(0.0, 0.0, 0.0, 0.0);
                    }
                }
                let falloff =
                    5.0 / (0.001 + InnerSpace::magnitude2(intersection_point - light.position));
                let intensity = clamp(
                    falloff * InnerSpace::dot(-light_ray.direction, normal),
                    0.0,
                    1.0,
                );
                intensity * light.color
            })
            .fold(Color::rgba(0.0, 0.0, 0.0, 0.0), |acc, x| acc + x);
        self.color * light_color
    }
}
