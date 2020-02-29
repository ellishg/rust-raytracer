use cgmath::InnerSpace;

use super::color::Color;
use super::light::Light;
use super::object::Object;
use super::ray::Ray;
use super::utils::{clamp, reflect};
use super::world::World;

pub trait Material<T: Object> {
    fn get_color(
        &self,
        incoming_ray: Ray,
        t: f32,
        object: &T,
        lights: Vec<&Light>,
        world: &World,
    ) -> Color;
}

pub struct Phong {
    color: Color,
    diffuse: f32,
    specular: f32,
    shininess: f32,
}

impl Phong {
    pub fn new<T: Object>(
        color: Color,
        diffuse: f32,
        specular: f32,
        shininess: f32,
    ) -> Box<dyn Material<T>> {
        Box::new(Phong {
            color,
            diffuse,
            specular,
            shininess,
        })
    }
}

impl<T: Object> Material<T> for Phong {
    fn get_color(
        &self,
        incoming_ray: Ray,
        t: f32,
        object: &T,
        lights: Vec<&Light>,
        _world: &World,
    ) -> Color {
        let intersection_point = incoming_ray.get_point_on_ray(t).into();
        let normal = object.get_normal(intersection_point);
        lights
            .iter()
            .map(|light| {
                // TODO: Either add ambient component here, or create a new light type.
                let light_ray = light.get_light_ray(intersection_point);
                // TODO: Give falloff code to Light.
                let falloff =
                    5.0 / (0.001 + InnerSpace::magnitude2(intersection_point - light.position));
                let light_color = falloff * light.color;
                let reflection_vector = reflect(light_ray.direction, normal);
                let specular_intensity = clamp(
                    InnerSpace::dot(reflection_vector, -incoming_ray.direction),
                    0.0,
                    1.0,
                );
                let diffuse_intensity =
                    clamp(InnerSpace::dot(-light_ray.direction, normal), 0.0, 1.0);
                self.color
                    * (self.diffuse * diffuse_intensity
                        + self.specular * specular_intensity.powf(self.shininess))
                    * light_color
            })
            .fold((0.0, 0.0, 0.0, 0.0).into(), |acc, x| acc + x)
    }
}
