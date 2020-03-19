use super::color::Color;
use cgmath::{InnerSpace, Point3, Vector3};

/// Ambient light has no position or direction
/// Point lights illumate from a single position
/// Directional lights represent parallel rays coming from infinitely far away.
pub enum LightType {
    Ambient,
    Point(Point3<f32>),        // position
    Directional(Vector3<f32>), // direction of parallel light rays
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

    pub fn new_directional(direction: Vector3<f32>, color: Color) -> Light {
        Light {
            color,
            light_type: LightType::Directional(direction.normalize()),
        }
    }
}
