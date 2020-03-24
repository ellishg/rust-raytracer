use super::bvh::Bvh;
use super::color::Color;
use super::ray::Ray;
use super::utils::clamp;
use cgmath::{Deg, InnerSpace, MetricSpace, Point3, Vector3};

/// Ambient light has no position or direction
/// Point lights illumate from a single position
/// Directional lights represent parallel rays coming from infinitely far away.
/// Cone lights are a point source in a certain direction, but only illuminate
///     within an angle of the direction.
pub enum LightType {
    Ambient,
    Point(Point3<f32>),                        // position
    Directional(Vector3<f32>),                 // direction of parallel light rays
    Cone(Point3<f32>, Vector3<f32>, Deg<f32>), // position, direction, angle
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

    pub fn new_cone(
        position: Point3<f32>,
        direction: Vector3<f32>,
        angle: Deg<f32>,
        color: Color,
    ) -> Light {
        Light {
            color,
            light_type: LightType::Cone(position, direction.normalize(), angle),
        }
    }

    pub fn get_falloff_at(&self, point: Point3<f32>) -> f32 {
        match self.light_type {
            LightType::Point(position) => {
                let distance_sqrd = (position - point).magnitude2();
                // TODO: Remove constants here.
                5.0 / (0.001 + distance_sqrd)
            }
            LightType::Cone(position, direction, angle) => {
                let cone_falloff = {
                    // TODO: Make this a field.
                    let falloff_range: Deg<f32> = Deg(8.0);
                    let angle_to_point: Deg<f32> = direction.angle(point - position).into();
                    let angle_delta = angle - angle_to_point;
                    clamp(angle_delta / falloff_range, 0.0, 1.0)
                };
                let distance_sqrd = (position - point).magnitude2();
                // TODO: Remove constants here.
                cone_falloff * 5.0 / (0.001 + distance_sqrd)
            }
            _ => unreachable!(),
        }
    }

    fn in_shadow(
        point: Point3<f32>,
        light_position: Point3<f32>,
        light_direction: Vector3<f32>,
        bvh: &Bvh,
    ) -> bool {
        let light_ray = Ray::new(light_position, light_direction);
        let light_to_point_t = point.distance(light_position);
        // TODO: Shadows don't work correctly with reflective or refractive surfaces.
        if let Some((_, shadow_t)) = bvh.get_closest_intersection(&light_ray) {
            let epsilon = 1e-4;
            let is_in_shadow = shadow_t + epsilon < light_to_point_t;
            !is_in_shadow
        } else {
            false
        }
    }

    pub fn reaches_point(&self, point: Point3<f32>, bvh: &Bvh) -> bool {
        match self.light_type {
            LightType::Ambient => true,
            LightType::Point(light_position) => {
                let light_direction = point - light_position;
                Light::in_shadow(point, light_position, light_direction, bvh)
            }
            LightType::Directional(direction) => {
                // Checks whether a ray starting from the intersection point, going in
                // the opposite direction of the light, hits another object.
                let object_to_light = Ray::new(point, -direction);
                let object_to_light = object_to_light.offset(1e-4);
                bvh.get_closest_intersection(&object_to_light).is_none()
            }
            LightType::Cone(light_position, direction, angle) => {
                let light_direction = point - light_position;
                if direction.angle(light_direction) > angle.into() {
                    false
                } else {
                    Light::in_shadow(point, light_position, light_direction, bvh)
                }
            }
        }
    }
}
