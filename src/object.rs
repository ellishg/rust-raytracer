use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix3, Matrix4, Point3, Vector3};

use super::color::Color;
use super::light::Light;
use super::material::Material;
use super::ray::Ray;
use super::world::World;

// NOTE: Another option would be to make Object hold an enum of all possible types.
pub trait Object {
    fn get_intersection(&self, ray: Ray) -> Option<f32>;
    fn get_normal(&self, point: Point3<f32>) -> Vector3<f32>;
    fn get_color(&self, incoming_ray: Ray, t: f32, lights: Vec<&Light>, world: &World) -> Color;
}

pub struct Sphere {
    world_to_object: Matrix4<f32>,
    material: Box<dyn Material<Sphere>>,
}

impl Sphere {
    pub fn new(
        position: Point3<f32>,
        radius: f32,
        material: Box<dyn Material<Sphere>>,
    ) -> Box<dyn Object> {
        let scale = Matrix4::from_scale(radius);
        let translate = Matrix4::from_translation(position.to_homogeneous().truncate());
        let object_to_world = translate * scale;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        Box::new(Sphere {
            world_to_object,
            material,
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

    fn get_normal(&self, point: Point3<f32>) -> Vector3<f32> {
        let point = self.world_to_object.transform_point(point);
        point.to_homogeneous().truncate().normalize()
    }

    fn get_color(&self, incoming_ray: Ray, t: f32, lights: Vec<&Light>, world: &World) -> Color {
        self.material
            .get_color(incoming_ray, t, &self, lights, world)
    }
}

pub struct Plane {
    world_to_object: Matrix4<f32>,
    material: Box<dyn Material<Plane>>,
}

impl Plane {
    pub fn new(
        position: Point3<f32>,
        normal: Vector3<f32>,
        material: Box<dyn Material<Plane>>,
    ) -> Box<Plane> {
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
        let object_to_world = translate * rotate;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        Box::new(Plane {
            world_to_object,
            material,
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
        if InnerSpace::dot(direction, normal) >= 0.0 {
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

    fn get_normal(&self, _point: Point3<f32>) -> Vector3<f32> {
        self.get_normal()
    }

    fn get_color(&self, incoming_ray: Ray, t: f32, lights: Vec<&Light>, world: &World) -> Color {
        self.material
            .get_color(incoming_ray, t, &self, lights, world)
    }
}

pub struct Triangle {
    a: Point3<f32>,
    b: Point3<f32>,
    c: Point3<f32>,
    material: Box<dyn Material<Triangle>>,
}

impl Triangle {
    pub fn new(
        a: Point3<f32>,
        b: Point3<f32>,
        c: Point3<f32>,
        material: Box<dyn Material<Triangle>>,
    ) -> Box<dyn Object> {
        Box::new(Triangle { a, b, c, material })
    }

    fn get_normal(&self) -> Vector3<f32> {
        (self.b - self.a).cross(self.c - self.a).normalize()
    }
}

impl Object for Triangle {
    fn get_intersection(&self, ray: Ray) -> Option<f32> {
        let normal = self.get_normal();
        let point_on_plane = self.a.to_homogeneous().truncate();
        let position = ray.position.to_homogeneous().truncate();
        let direction = ray.direction;
        if InnerSpace::dot(direction, normal) >= 0.0 {
            None
        } else {
            let t = InnerSpace::dot(point_on_plane - position, normal)
                / InnerSpace::dot(direction, normal);
            if t > 0.0 {
                let intersection_point: Point3<f32> = ray.get_point_on_ray(t).into();
                if InnerSpace::dot((self.b - self.a).cross(intersection_point - self.a), normal)
                    >= 0.0
                    && InnerSpace::dot((self.c - self.b).cross(intersection_point - self.b), normal)
                        >= 0.0
                    && InnerSpace::dot((self.a - self.c).cross(intersection_point - self.c), normal)
                        >= 0.0
                {
                    Some(t)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    fn get_normal(&self, _point: Point3<f32>) -> Vector3<f32> {
        self.get_normal()
    }

    fn get_color(&self, incoming_ray: Ray, t: f32, lights: Vec<&Light>, world: &World) -> Color {
        self.material
            .get_color(incoming_ray, t, &self, lights, world)
    }
}

#[cfg(test)]
mod tests {
    use super::{Object, Plane, Sphere, Triangle};
    use crate::material::Phong;
    use crate::ray::Ray;

    #[test]
    fn test_sphere() {
        let c = (1.0, 0.0, 0.0).into();
        let m = Phong::new(c, 1.0, 1.0, 1.0);
        let sphere = Sphere::new((1.0, 2.0, 3.0).into(), 0.25, m);
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (-1.0, 0.0, 0.0).into());
        assert!(sphere.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (1.0, 1.0, 3.0).into());
        assert!(sphere.get_intersection(ray).is_some());
    }

    #[test]
    fn test_plane() {
        let c = (1.0, 0.0, 0.0).into();
        let m = Phong::new(c, 1.0, 1.0, 1.0);
        let plane = Plane::new((0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into(), m);
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, -1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_none());
    }

    #[test]
    fn test_triangle() {
        let c = (1.0, 0.0, 0.0).into();
        let m = Phong::new(c, 1.0, 1.0, 1.0);
        let triangle = Triangle::new(
            (0.0, 0.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            m,
        );
        let ray = Ray::new((0.1, 0.1, 1.0).into(), (0.0, 0.0, -1.0).into());
        assert!(triangle.get_intersection(ray).is_some());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(ray).is_none());

        let triangle = Triangle::new(
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            c,
        );
        let ray = Ray::new((0.1, 0.1, 1.0).into(), (0.0, 0.0, -1.0).into());
        assert!(triangle.get_intersection(ray).is_none());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(ray).is_none());
    }
}
