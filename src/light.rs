use super::color::Color;
use super::ray::Ray;
use cgmath::Point3;

enum LightType {
    Point(Point3<f32>),
    Ambient,
}

pub struct Light {
    pub color: Color,
    pub light_type: LightType,
}

impl Light {
    pub fn new_point(position: Point3<f32>, color: Color) -> Light {
        Light { color, light_type: LightType::Point(position) }
    }

    pub fn new_ambient(color: Color) -> Light {
        Light { color, light_type: LightType::Ambient }
    }

    pub fn get_light_ray(&self, point: Point3<f32>) -> Option<Ray> {
        match self.light_type {
            LightType::Ambient => None,
            LightType::Point(position) => {
                Some(Ray::new(position, point - position))
            }
        }
    }
}
