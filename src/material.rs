use cgmath::InnerSpace;
use cgmath::Point2;
use image;
use std::error::Error;
use std::rc::Rc;

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
    texture: Option<Rc<Texture>>,
    diffuse: f32,
    specular: f32,
    shininess: f32,
}

impl Phong {
    pub fn new<T: Object>(
        color: Color,
        texture: Option<Rc<Texture>>,
        diffuse: f32,
        specular: f32,
        shininess: f32,
    ) -> Box<dyn Material<T>> {
        Box::new(Phong {
            color,
            texture,
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
        let surface_color = {
            if let Some(texture) = &self.texture {
                let uv = object.get_uv(intersection_point);
                texture.sample(uv)
            } else {
                self.color
            }
        };
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
                surface_color
                    * (self.diffuse * diffuse_intensity
                        + self.specular * specular_intensity.powf(self.shininess))
                    * light_color
            })
            .fold((0.0, 0.0, 0.0, 0.0).into(), |acc, x| acc + x)
    }
}

pub struct Texture {
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

    // TODO: Add options for wrapping/clamping and filter type.
    pub fn sample(&self, point: Point2<f32>) -> Color {
        let (u, v) = (point.x, point.y);
        let width = self.buf.width() as f32;
        let height = self.buf.height() as f32;
        let (u, v) = (u.rem_euclid(1.0), v.rem_euclid(1.0));
        let x = (u * (width)).trunc();
        let y = (v * (height)).trunc();
        let x = clamp(x, 0.0, width - 1.0) as u32;
        let y = clamp(y, 0.0, height - 1.0) as u32;
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
        assert_eq!(texture.sample((0.0, 0.0).into()).get_rgb(), (64, 64, 64));
        assert_eq!(
            texture
                .sample((16.0 / 255.0, 15.0 / 255.0).into())
                .get_rgb(),
            (229, 22, 177)
        );
        assert_eq!(
            texture
                .sample((16.0 / 255.0, 15.0 / 255.0).into())
                .get_rgb(),
            (229, 22, 177)
        );
        assert_eq!(
            texture
                .sample((46.0 / 255.0, 79.0 / 255.0).into())
                .get_rgb(),
            (22, 229, 229)
        );
        assert_eq!(texture.sample((1.0, 1.0).into()).get_rgb(), (64, 64, 64));
        assert_eq!(texture.sample((2.0, 2.0).into()).get_rgb(), (64, 64, 64));
        assert_eq!(texture.sample((-1.0, -1.0).into()).get_rgb(), (64, 64, 64));
    }
}
