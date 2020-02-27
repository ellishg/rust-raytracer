use cgmath::{InnerSpace, Transform};
use cgmath::{Matrix4, Point3, Vector3};

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub position: Point3<f32>,
    pub direction: Vector3<f32>,
}

impl Ray {
    pub fn new(position: Point3<f32>, direction: Vector3<f32>) -> Ray {
        Ray {
            position,
            direction: direction.normalize(),
        }
    }

    pub fn transform_using(&self, transform: Matrix4<f32>) -> Ray {
        Ray {
            position: transform.transform_point(self.position),
            direction: transform.transform_vector(self.direction),
        }
    }

    pub fn get_point_on_ray(&self, t: f32) -> (f32, f32, f32) {
        let p = self.position + t * self.direction;
        p.into()
    }
}
