use super::color::Color;
use cgmath::Point3;

pub enum LightType {
    Point(Point3<f32>), // light position
    Ambient,
}

pub struct Light {
    pub color: Color,
    pub light_type: LightType,
}

impl Light {
    pub fn new_point(position: Point3<f32>, color: Color) -> Light {
        Light {
            color,
            light_type: LightType::Point(position),
        }
    }

    pub fn new_ambient(color: Color) -> Light {
        Light {
            color,
            light_type: LightType::Ambient,
        }
    }
}
