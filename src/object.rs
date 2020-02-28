use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix3, Matrix4, Point3, Vector3};

use super::color::Color;
use super::ray::Ray;
use super::utils::clamp;

// TODO: Make Material enum or trait.

// NOTE: Another option would be to make Object hold an enum of all possible types.
pub trait Object {
    fn get_intersection(&self, ray: Ray) -> Option<f32>;
    fn get_light_intensity(&self, point: Point3<f32>, light_vector: Vector3<f32>) -> f32;
    fn get_color(&self, light_color: Color, ray: Ray, t: f32) -> Color;
    fn get_normal(&self, point: Point3<f32>) -> Vector3<f32>;
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

    fn get_light_intensity(&self, point: Point3<f32>, light_vector: Vector3<f32>) -> f32 {
        let normal = self.get_normal(point);
        let light_vector = self
            .world_to_object
            .transform_vector(light_vector)
            .normalize();
        // TODO: This is material code.
        clamp(InnerSpace::dot(-light_vector, normal), 0.0, 1.0)
    }

    fn get_color(&self, light_color: Color, _ray: Ray, _t: f32) -> Color {
        self.color * light_color
    }

    fn get_normal(&self, point: Point3<f32>) -> Vector3<f32> {
        let point = self.world_to_object.transform_point(point);
        point.to_homogeneous().truncate().normalize()
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

    fn get_normal(&self) -> Vector3<f32> {
        (0.0, 1.0, 0.0).into()
    }
}

impl Object for Plane {
    fn get_intersection(&self, ray: Ray) -> Option<f32> {
        let ray = ray.transform_using(self.world_to_object);
        let normal = self.get_normal();
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

    fn get_light_intensity(&self, _point: Point3<f32>, light_vector: Vector3<f32>) -> f32 {
        let light_vector = self
            .world_to_object
            .transform_vector(light_vector)
            .normalize();
        let normal = self.get_normal();
        // TODO: This is material code.
        clamp(InnerSpace::dot(-light_vector, normal), 0.0, 1.0)
    }

    fn get_color(&self, light_color: Color, _ray: Ray, _t: f32) -> Color {
        self.color * light_color
    }

    fn get_normal(&self, _point: Point3<f32>) -> Vector3<f32> {
        self.get_normal()
    }
}

#[cfg(test)]
mod tests {
    use super::{Object, Plane, Sphere};
    use crate::color::Color;
    use crate::ray::Ray;

    #[test]
    fn test_sphere() {
        let c = Color::rgb(1.0, 0.0, 0.0);
        let sphere = Sphere::new((1.0, 2.0, 3.0).into(), 0.25, c);
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (-1.0, 0.0, 0.0).into());
        assert!(sphere.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (1.0, 1.0, 3.0).into());
        assert!(sphere.get_intersection(ray).is_some());
    }

    #[test]
    fn test_plane() {
        let c = Color::rgb(1.0, 0.0, 0.0);
        let plane = Plane::new((0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into(), c);
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, -1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_some());
    }
}
