use super::color::Color;
use super::ray::Ray;
use cgmath::Point3;

// TODO: Use enum or trait to define light types.
pub struct Light {
    pub position: Point3<f32>,
    pub color: Color,
}

impl Light {
    pub fn new(position: Point3<f32>, color: Color) -> Light {
        Light { position, color }
    }

    pub fn get_light_ray(&self, point: Point3<f32>) -> Ray {
        Ray::new(self.position, point - self.position)
    }
}
