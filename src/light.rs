use super::bvh::Bvh;
use super::color::Color;
use super::ray::Ray;
use cgmath::{Deg, InnerSpace, MetricSpace, Point3, Vector3};

pub enum Light {
    Ambient(f32, Color),
    Point(Point3<f32>, Color),
    Cone(Point3<f32>, Vector3<f32>, Deg<f32>, Color),
}

impl Light {
    pub fn is_ambient(&self) -> bool {
        if let Light::Ambient(_, _) = self {
            true
        } else {
            false
        }
    }

    pub fn get_color(&self) -> Color {
        match self {
            Light::Ambient(_, color) => *color,
            Light::Point(_, color) => *color,
            Light::Cone(_, _, _, color) => *color,
        }
    }

    pub fn get_position(&self) -> Option<Point3<f32>> {
        match self {
            Light::Ambient(_, _) => None,
            Light::Point(position, _) => Some(*position),
            Light::Cone(position, _, _, _) => Some(*position),
        }
    }

    pub fn get_direction(&self, point: Point3<f32>) -> Option<Vector3<f32>> {
        let direction = match self {
            Light::Ambient(_, _) => None,
            Light::Point(position, _) => Some(point - position),
            Light::Cone(position, _, _, _) => Some(point - position),
        };
        direction.map(|direction| direction.normalize())
    }

    fn get_falloff(&self, distance_sqrd: f32) -> Option<f32> {
        match self {
            Light::Ambient(_, _) => None,
            Light::Point(_, _) => {
                // TODO: Remove constants here.
                Some(5.0 / (0.001 + distance_sqrd))
            }
            Light::Cone(_, _, _, _) => {
                // TODO: This is wrong.
                Some(5.0 / (0.001 + distance_sqrd))
            }
        }
    }

    pub fn reaches_point(&self, point: Point3<f32>, bvh: &Bvh) -> bool {
        match self {
            Light::Ambient(_, _) => true,
            Light::Point(_, _) => {
                let light_position = self.get_position().unwrap();
                let light_direction = self.get_direction(point).unwrap();
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
            Light::Cone(_, direction, angle, _) => {
                let direction = direction.normalize();
                let light_position = self.get_position().unwrap();
                let light_direction = self.get_direction(point).unwrap();
                if direction.angle(light_direction) > (*angle).into() {
                    false
                } else {
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
            }
        }
    }

    pub fn get_intensity_at(&self, point: Point3<f32>) -> f32 {
        if let Light::Ambient(intensity, _) = self {
            *intensity
        } else {
            let distance_sqrd = (self.get_position().unwrap() - point).magnitude2();
            self.get_falloff(distance_sqrd).unwrap()
        }
    }
}
