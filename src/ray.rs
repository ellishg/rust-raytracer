use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix4, Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    position: Point3<f32>,
    direction: Vector3<f32>,
}

impl Ray {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray {
            position,
            direction: direction.normalize(),
        }
    }

    pub fn transform_using(&self, transform: &Matrix4<f32>) -> Ray {
        Ray::new(
            transform.transform_point(self.position),
            transform.transform_vector(self.direction),
        )
    }

    pub fn get_point_on_ray(&self, t: f32) -> (f32, f32, f32) {
        let p = self.position + t * self.direction;
        p.into()
    }

    // Returns the `t` that makes `ray.get_point_on_ray(t)` the closest to `point`.
    pub fn get_t(&self, point: Point3<f32>) -> f32 {
        (point - self.position).dot(self.direction)
    }

    pub fn get_direction(&self) -> Vector3<f32> {
        self.direction
    }

    /// Move the ray forward by `epsilon` units.
    ///
    /// Useful if we want to make sure that the new ray does not
    /// intersect an object at its origin.
    pub fn offset(self, epsilon: f32) -> Ray {
        let position = self.position + epsilon * self.direction;
        Ray::new(position, self.direction)
    }
}
