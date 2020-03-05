use cgmath::{InnerSpace, SquareMatrix, Transform};
use cgmath::{Matrix3, Matrix4, Point2, Point3, Vector3};
use obj;
use std::error::Error;
use std::path::Path;

use super::color::Color;
use super::light::Light;
use super::material::Material;
use super::ray::Ray;
use super::world::World;

enum ObjectType {
    Sphere,
    Plane,
    Triangle(Point3<f32>, Point3<f32>, Point3<f32>),
}

pub struct Object {
    object_type: ObjectType,
    world_to_object: Matrix4<f32>,
    material: Material,
}

impl Object {
    /// Returns a list of triangles built from a .obj file.
    pub fn new_mesh<P>(
        path: P,
        world_to_object: Matrix4<f32>,
        material: Material,
    ) -> Result<Vec<Self>, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let obj = obj::Obj::<obj::SimplePolygon>::load(path)?;
        let triangles: Vec<Object> = obj
            .objects
            .iter()
            .flat_map(|object| {
                debug!("Reading {} from {}", object.name, path.display());
                let triangles: Vec<Object> = object
                    .groups
                    .iter()
                    .flat_map(|group| {
                        let triangles: Vec<Object> = group
                            .polys
                            .iter()
                            .map(|poly| {
                                // TODO: Currently only triangle meshes are supported.
                                assert_eq!(poly.len(), 3);
                                let vertex_indices: Vec<usize> =
                                    poly.iter().map(|tuple| tuple.0).collect();
                                // TODO: .obj files also hold normal and material information.
                                // let texture_indices: Vec<Option<usize>> = poly.iter().map(|tuple| tuple.1).collect();
                                // let normal_indices: Vec<Option<usize>> = poly.iter().map(|tuple| tuple.2).collect();
                                let vertices: Vec<[f32; 3]> = vertex_indices
                                    .into_iter()
                                    .map(|i| obj.position[i])
                                    .collect();
                                let a = vertices[0].into();
                                let b = vertices[1].into();
                                let c = vertices[2].into();
                                Object {
                                    object_type: ObjectType::Triangle(a, b, c),
                                    world_to_object,
                                    material: material.clone(),
                                }
                            })
                            .collect();
                        triangles
                    })
                    .collect();
                triangles
            })
            .collect();
        Ok(triangles)
    }

    pub fn new_sphere(position: Point3<f32>, radius: f32, material: Material) -> Self {
        let scale = Matrix4::from_scale(radius);
        let translate = Matrix4::from_translation(position.to_homogeneous().truncate());
        let object_to_world = translate * scale;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        let object_type = ObjectType::Sphere;
        Object {
            object_type,
            world_to_object,
            material,
        }
    }

    pub fn new_plane(position: Point3<f32>, normal: Vector3<f32>, material: Material) -> Self {
        let normal = normal.normalize();
        // Pick an arbitrary orthogonal vector.
        let new_x_axis = {
            let axis: Vector3<f32> = if normal.x != 0.0 {
                (-normal.y, normal.x, 0.0).into()
            } else if normal.y != 0.0 {
                (normal.y, -normal.x, 0.0).into()
            } else {
                (normal.z, 0.0, -normal.x).into()
            };
            axis.normalize()
        };
        let new_y_axis = normal;
        let new_z_axis = new_x_axis.cross(new_y_axis).normalize();
        let rotate: Matrix4<f32> = Matrix3::from_cols(new_x_axis, new_y_axis, new_z_axis).into();
        let translate = Matrix4::from_translation(position.to_homogeneous().truncate());
        let object_to_world = translate * rotate;
        let world_to_object = Transform::inverse_transform(&object_to_world).unwrap();
        let object_type = ObjectType::Plane;
        Object {
            object_type,
            world_to_object,
            material,
        }
    }

    pub fn new_triangle(
        a: Point3<f32>,
        b: Point3<f32>,
        c: Point3<f32>,
        material: Material,
    ) -> Self {
        let world_to_object = Matrix4::identity();
        let object_type = ObjectType::Triangle(a, b, c);
        Object {
            object_type,
            world_to_object,
            material,
        }
    }

    /// If `ray` instersects this object, returns `t` such that the
    /// intersection point is at `ray.get_point_on_ray(t)`.
    ///
    /// Both `ray` and `t` are in world space coordinates.
    pub fn get_intersection(&self, ray: Ray) -> Option<f32> {
        let object_space_ray = ray.transform_using(self.world_to_object);
        let position = object_space_ray.get_point_on_ray(0.0).into();
        let direction = object_space_ray.get_direction();
        let t = match self.object_type {
            ObjectType::Sphere => {
                // A Sphere is centered at origin with radius 1
                // (thanks to the matrix transformations).
                let closest_point_to_origin =
                    position - InnerSpace::dot(position, direction) * direction;
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
            ObjectType::Plane => {
                // A Plane is at the origin and has a normal pointing up the y-axis
                // (thanks to the matrix transformations).
                let normal = (0.0, 1.0, 0.0).into();
                if InnerSpace::dot(direction, normal) < 0.0 {
                    let t = InnerSpace::dot(-position, normal) / InnerSpace::dot(direction, normal);
                    if t > 0.0 {
                        Some(t)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            ObjectType::Triangle(a, b, c) => {
                let normal = (b - a).cross(c - a).normalize();
                let point_on_plane = a.to_homogeneous().truncate();
                if InnerSpace::dot(direction, normal) < 0.0 {
                    let t = InnerSpace::dot(point_on_plane - position, normal)
                        / InnerSpace::dot(direction, normal);
                    if t > 0.0 {
                        let intersection_point: Point3<f32> =
                            object_space_ray.get_point_on_ray(t).into();
                        if InnerSpace::dot((b - a).cross(intersection_point - a), normal) >= 0.0
                            && InnerSpace::dot((c - b).cross(intersection_point - b), normal) >= 0.0
                            && InnerSpace::dot((a - c).cross(intersection_point - c), normal) >= 0.0
                        {
                            Some(t)
                        } else {
                            // TODO: Is there a rustic way to remove all these "else None"?
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        };
        t.map(|t| {
            let object_space_intersection_point = object_space_ray.get_point_on_ray(t).into();
            let object_to_world = self.world_to_object.invert().unwrap();
            let intersection_point =
                object_to_world.transform_point(object_space_intersection_point);
            ray.get_t(intersection_point)
        })
    }

    /// Returns the normal of the object at `point`.
    ///
    /// `point` is in world space coordinates.
    pub fn get_normal(&self, point: Point3<f32>) -> Vector3<f32> {
        let point = self.world_to_object.transform_point(point);
        match self.object_type {
            ObjectType::Sphere => point.to_homogeneous().truncate().normalize(),
            ObjectType::Plane => (0.0, 1.0, 0.0).into(),
            ObjectType::Triangle(a, b, c) => (b - a).cross(c - a).normalize(),
        }
    }

    /// Returns the color of the object at the point given by `incoming_ray.get_point_on_ray(t)`.
    pub fn get_color(
        &self,
        incoming_ray: Ray,
        t: f32,
        lights: Vec<&Light>,
        world: &World,
    ) -> Color {
        self.material
            .get_color(incoming_ray, t, self, lights, world)
    }

    /// Returns the uv texture coordinates of the object at `point`.
    ///
    /// `point` is in world space coordinates.
    pub fn get_uv(&self, point: Point3<f32>) -> Point2<f32> {
        let point = self.world_to_object.transform_point(point);
        match self.object_type {
            ObjectType::Sphere => {
                let theta = point.z.acos();
                let phi = point.y.atan2(point.x);
                Point2 {
                    x: theta / (2.0 * std::f32::consts::PI),
                    y: phi / std::f32::consts::PI,
                }
            }
            ObjectType::Plane => Point2 {
                x: point.x,
                y: point.z,
            },
            ObjectType::Triangle(a, b, c) => {
                let point = point.to_homogeneous().truncate();
                let u = InnerSpace::dot(b - a, point);
                let v = InnerSpace::dot(c - a, point);
                Point2 { x: u, y: v }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Object;
    use crate::material::{Material, MaterialType, TextureType};
    use crate::ray::Ray;

    #[test]
    fn test_sphere() {
        let m = Material::new(MaterialType::None, TextureType::None);
        let sphere = Object::new_sphere((1.0, 2.0, 3.0).into(), 0.25, m);
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (-1.0, 0.0, 0.0).into());
        assert!(sphere.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (1.0, 1.0, 3.0).into());
        assert!(sphere.get_intersection(ray).is_some());
    }

    #[test]
    fn test_plane() {
        let m = Material::new(MaterialType::None, TextureType::None);
        let plane = Object::new_plane((0.0, 0.0, 0.0).into(), (0.0, 1.0, 0.0).into(), m);
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(plane.get_intersection(ray).is_none());
        let ray = Ray::new((0.0, -1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(plane.get_intersection(ray).is_none());
    }

    #[test]
    fn test_triangle() {
        let m = Material::new(MaterialType::None, TextureType::None);
        let triangle = Object::new_triangle(
            (0.0, 0.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            m.clone(),
        );
        let ray = Ray::new((0.1, 0.1, 1.0).into(), (0.0, 0.0, -1.0).into());
        assert!(triangle.get_intersection(ray).is_some());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(ray).is_none());

        let triangle = Object::new_triangle(
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            m.clone(),
        );
        let ray = Ray::new((0.1, 0.1, 1.0).into(), (0.0, 0.0, -1.0).into());
        assert!(triangle.get_intersection(ray).is_none());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(ray).is_none());
    }
}
