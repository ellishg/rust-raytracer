use cgmath::Point3;
use super::color::Color;

// TODO: Use enum or trait to define light types.
pub struct Light {
    pub position: Point3<f32>,
    pub color: Color,
}

impl Light {
    pub fn new(position: Point3<f32>, color: Color) -> Light {
        Light { position, color }
    }
}