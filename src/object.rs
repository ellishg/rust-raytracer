use cgmath::{InnerSpace, SquareMatrix, Transform};
use cgmath::{Matrix4, Point2, Point3, Vector3};
use obj;
use std::error::Error;
use std::path::Path;

use super::color::Color;
use super::light::Light;
use super::material::Material;
use super::ray::Ray;
use super::utils::component_wise_range;
use super::world::World;

enum ObjectType {
    Sphere(Point3<f32>, f32),
    Triangle(Point3<f32>, Point3<f32>, Point3<f32>),
    Quad(Point3<f32>, Point3<f32>, Point3<f32>, Point3<f32>),
}

pub struct Object {
    object_type: ObjectType,
    // TODO: Make `object_to_world` a reference to save memory
    object_to_world: Matrix4<f32>,
    world_to_object: Matrix4<f32>,
    material: Material,
}

impl Object {
    /// Returns a list of triangles built from a .obj file.
    pub fn new_mesh<P>(
        path: P,
        object_to_world: Matrix4<f32>,
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
                                    object_to_world: object_to_world,
                                    world_to_object: object_to_world.inverse_transform().unwrap(),
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

    pub fn new_sphere(center: Point3<f32>, radius: f32, material: Material) -> Self {
        Object {
            object_type: ObjectType::Sphere(center, radius),
            object_to_world: Matrix4::identity(),
            world_to_object: Matrix4::identity(),
            material,
        }
    }

    pub fn new_quad(
        a: Point3<f32>,
        b: Point3<f32>,
        c: Point3<f32>,
        d: Point3<f32>,
        material: Material,
    ) -> Self {
        Object {
            object_type: ObjectType::Quad(a, b, c, d),
            object_to_world: Matrix4::identity(),
            world_to_object: Matrix4::identity(),
            material,
        }
    }

    pub fn new_triangle(
        a: Point3<f32>,
        b: Point3<f32>,
        c: Point3<f32>,
        material: Material,
    ) -> Self {
        Object {
            object_type: ObjectType::Triangle(a, b, c),
            object_to_world: Matrix4::identity(),
            world_to_object: Matrix4::identity(),
            material,
        }
    }

    pub fn transform(self, object_to_world: Matrix4<f32>) -> Self {
        let object_to_world = object_to_world * self.get_object_to_world();
        Object {
            object_type: self.object_type,
            object_to_world: object_to_world,
            world_to_object: object_to_world.inverse_transform().unwrap(),
            material: self.material,
        }
    }

    /// If `ray` instersects this object, returns `t` such that the
    /// intersection point is at `ray.get_point_on_ray(t)`.
    ///
    /// Both `ray` and `t` are in world space coordinates.
    pub fn get_intersection(&self, ray: &Ray) -> Option<f32> {
        let object_space_ray = ray.transform_using(self.get_world_to_object());
        let position: Point3<f32> = object_space_ray.get_point_on_ray(0.0).into();
        let direction = object_space_ray.get_direction();
        let t = match self.object_type {
            ObjectType::Sphere(center, radius) => {
                let t = (center - position).dot(direction);
                let closest_point_to_center: Point3<f32> =
                    object_space_ray.get_point_on_ray(t).into();
                let radius_sqrd = radius.powf(2.0);
                let dist_to_center_sqrd = (center - closest_point_to_center).magnitude2();
                if dist_to_center_sqrd <= radius_sqrd {
                    let delta = (radius_sqrd - dist_to_center_sqrd).sqrt();
                    // Find the smallest positive t value.
                    vec![t - delta, t + delta]
                        .into_iter()
                        .filter(|t| t.is_sign_positive())
                        .min_by(|x, y| x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal))
                } else {
                    None
                }
            }
            ObjectType::Quad(a, b, c, d) => {
                let normal = (b - a).cross(d - a).normalize();
                if direction.dot(normal) < 0.0 {
                    let t = (a - position).dot(normal) / direction.dot(normal);
                    if t > 0.0 {
                        let intersection_point: Point3<f32> =
                            object_space_ray.get_point_on_ray(t).into();
                        let inside_quad = vec![
                            (b - a).cross(intersection_point - a),
                            (c - b).cross(intersection_point - b),
                            (d - c).cross(intersection_point - c),
                            (a - d).cross(intersection_point - d),
                        ]
                        .iter()
                        .map(|v| v.dot(normal))
                        .all(|x| x.is_sign_positive());
                        if inside_quad {
                            return Some(t);
                        }
                    }
                }
                None
            }
            ObjectType::Triangle(a, b, c) => {
                let normal = (b - a).cross(c - a).normalize();
                if direction.dot(normal) < 0.0 {
                    let t = (a - position).dot(normal) / direction.dot(normal);
                    if t > 0.0 {
                        let intersection_point: Point3<f32> =
                            object_space_ray.get_point_on_ray(t).into();
                        let inside_triangle = vec![
                            (b - a).cross(intersection_point - a),
                            (c - b).cross(intersection_point - b),
                            (a - c).cross(intersection_point - c),
                        ]
                        .iter()
                        .map(|v| v.dot(normal))
                        .all(|x| x.is_sign_positive());
                        if inside_triangle {
                            return Some(t);
                        }
                    }
                }
                None
            }
        };
        t.map(|t| {
            let object_space_intersection_point = object_space_ray.get_point_on_ray(t).into();
            let object_to_world = self.get_object_to_world();
            let intersection_point =
                object_to_world.transform_point(object_space_intersection_point);
            ray.get_t(intersection_point)
        })
    }

    /// Returns the normal of the object at `point` in world space coordinates.
    ///
    /// `point` is in world space coordinates.
    pub fn get_normal(&self, point: Point3<f32>) -> Vector3<f32> {
        let point = self.get_world_to_object().transform_point(point);
        let normal = match self.object_type {
            ObjectType::Sphere(center, _) => (point - center).normalize(),
            ObjectType::Quad(a, b, _c, d) => (b - a).cross(d - a).normalize(),
            ObjectType::Triangle(a, b, c) => (b - a).cross(c - a).normalize(),
        };
        self.get_object_to_world()
            .transform_vector(normal)
            .normalize()
    }

    /// Returns the color of the object at the point given by `incoming_ray.get_point_on_ray(t)`.
    ///
    /// All arguments are in world space coordinates.
    pub fn get_color(
        &self,
        incoming_ray: &Ray,
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
        let point = self.get_world_to_object().transform_point(point);
        match self.object_type {
            ObjectType::Sphere(center, _) => {
                let (x, y, z) = (point - center).normalize().into();
                let theta = z.acos();
                let phi = y.atan2(x);
                Point2 {
                    x: theta / (2.0 * std::f32::consts::PI),
                    y: phi / std::f32::consts::PI,
                }
            }
            ObjectType::Quad(a, b, _c, d) => {
                let u = (b - a).dot(point - a) / (b - a).magnitude2();
                let v = (d - a).dot(point - a) / (d - a).magnitude2();
                Point2 { x: u, y: v }
            }
            ObjectType::Triangle(a, b, c) => {
                let u = (b - a).dot(point - a) / (b - a).magnitude2();
                let v = (c - a).dot(point - a) / (c - a).magnitude2();
                Point2 { x: u, y: v }
            }
        }
    }

    fn get_object_to_world(&self) -> &Matrix4<f32> {
        &self.object_to_world
    }

    fn get_world_to_object(&self) -> &Matrix4<f32> {
        &self.world_to_object
    }

    /// Return the axis-aligned minimum bounding box for this object
    /// in world space coordinates.
    pub fn get_bounding_box(&self) -> (Point3<f32>, Point3<f32>) {
        let object_to_world = self.get_object_to_world();
        match self.object_type {
            ObjectType::Sphere(center, radius) => {
                let center = object_to_world.transform_point(center);
                // FIXME: Radius is not affected by transformation matrix.
                let radius: Vector3<f32> = (radius, radius, radius).into();
                (center - radius, center + radius)
            }
            ObjectType::Quad(a, b, c, d) => {
                let points = vec![a, b, c, d]
                    .into_iter()
                    .map(|point| object_to_world.transform_point(point))
                    .collect();
                component_wise_range(points)
            }
            ObjectType::Triangle(a, b, c) => {
                let points = vec![a, b, c]
                    .into_iter()
                    .map(|point| object_to_world.transform_point(point))
                    .collect();
                component_wise_range(points)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Object;
    use crate::material::{Material, MaterialType, TextureType};
    use crate::ray::Ray;
    use cgmath::{InnerSpace, Point3};

    #[test]
    fn test_sphere() {
        let m = Material::new(MaterialType::None, TextureType::None);
        let sphere = Object::new_sphere((1.0, 2.0, 3.0).into(), 0.25, m);
        let ray = Ray::new((0.0, 0.0, 0.0).into(), (-1.0, 0.0, 0.0).into());
        assert!(sphere.get_intersection(&ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (1.0, 1.0, 3.0).into());
        assert!(sphere.get_intersection(&ray).is_some());
    }

    #[test]
    fn test_quad() {
        let m = Material::new(MaterialType::None, TextureType::None);
        let quad = Object::new_quad(
            (-1.0, 0.0, 1.0).into(),
            (1.0, 0.0, 1.0).into(),
            (1.0, 0.0, -1.0).into(),
            (-1.0, 0.0, -1.0).into(),
            m,
        );
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, -1.0, 0.5).into());
        assert!(quad.get_intersection(&ray).is_some());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(quad.get_intersection(&ray).is_none());
        let ray = Ray::new((0.0, 1.0, 0.0).into(), (0.0, 0.0, 1.0).into());
        assert!(quad.get_intersection(&ray).is_none());
        let ray = Ray::new((0.0, -1.0, 0.0).into(), (0.0, 1.0, 0.0).into());
        assert!(quad.get_intersection(&ray).is_none());
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
        assert!(triangle.get_intersection(&ray).is_some());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(&ray).is_none());

        let triangle = Object::new_triangle(
            (0.0, 0.0, 0.0).into(),
            (0.0, 1.0, 0.0).into(),
            (1.0, 0.0, 0.0).into(),
            m.clone(),
        );
        let ray = Ray::new((0.1, 0.1, 1.0).into(), (0.0, 0.0, -1.0).into());
        assert!(triangle.get_intersection(&ray).is_none());
        let ray = Ray::new((1.0, 1.0, -1.0).into(), (0.0, 0.0, 1.0).into());
        assert!(triangle.get_intersection(&ray).is_none());
    }

    #[test]
    fn test_sphere_bounding_box() {
        let epsilon = 1e-4;
        let m = Material::new(MaterialType::None, TextureType::None);
        let sphere = Object::new_sphere((2.0, 0.0, 1.0).into(), 1.5, m.clone());
        let (a, b) = sphere.get_bounding_box();
        let a_actual: Point3<f32> = (0.5, -1.5, -0.5).into();
        let b_actual: Point3<f32> = (3.5, 1.5, 2.5).into();
        assert!((a - a_actual).magnitude() < epsilon);
        assert!((b - b_actual).magnitude() < epsilon);
    }

    #[test]
    fn test_quad_bounding_box() {
        let epsilon = 1e-4;
        let m = Material::new(MaterialType::None, TextureType::None);
        let quad = Object::new_quad(
            (1.0, 1.0, 1.0).into(),
            (3.0, 1.0, 1.0).into(),
            (3.0, 1.0, 0.0).into(),
            (1.0, 1.0, 0.0).into(),
            m.clone(),
        );
        let (a, b) = quad.get_bounding_box();
        let a_actual: Point3<f32> = (1.0, 1.0, 0.0).into();
        let b_actual: Point3<f32> = (3.0, 1.0, 1.0).into();
        assert!((a - a_actual).magnitude() < epsilon);
        assert!((b - b_actual).magnitude() < epsilon);
    }
}
