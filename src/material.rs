use cgmath::InnerSpace;
use image;
use std::error::Error;

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

struct Texture {
    buf: image::RgbImage,
}

impl Texture {
    pub fn new<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<std::path::Path>,
    {
        let image = image::io::Reader::open(path)?.decode()?;
        let buf = image.to_rgb();
        Ok(Texture { buf })
    }

    pub fn sample(&self, u: f32, v: f32) -> Color {
        let width = self.buf.width() as f32;
        let height = self.buf.height() as f32;
        let (u, v) = (clamp(u, 0.0, 1.0), clamp(v, 0.0, 1.0));
        let x = (u * width).trunc() as u32;
        let y = (v * height).trunc() as u32;
        let pixel = self.buf.get_pixel(x, y);
        let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
        let r = (r as f32) / 255.0;
        let g = (g as f32) / 255.0;
        let b = (b as f32) / 255.0;
        Color::rgb(r, g, b)
    }
}

#[cfg(test)]
mod tests {
    use crate::material::Texture;

    #[test]
    fn test_sample_texture() {
        let texture = Texture::new("media/texture.png").unwrap();
        assert_eq!(texture.sample(0.0, 0.0).get_rgb(), (64, 64, 64));
        assert_eq!(
            texture.sample(16.0 / 255.0, 15.0 / 255.0).get_rgb(),
            (229, 22, 177)
        );
        assert_eq!(
            texture.sample(16.0 / 255.0, 15.0 / 255.0).get_rgb(),
            (229, 22, 177)
        );
        assert_eq!(
            texture.sample(46.0 / 255.0, 79.0 / 255.0).get_rgb(),
            (22, 229, 229)
        );
    }
}
