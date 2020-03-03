use cgmath::InnerSpace;
use cgmath::Point3;
use image;
use std::error::Error;
use std::rc::Rc;

use super::color::Color;
use super::light::Light;
use super::object::Object;
use super::ray::Ray;
use super::utils::{clamp, reflect};
use super::world::World;

pub enum TextureType {
    Texture(Rc<image::RgbImage>),
    Flat(Color),
    None,
}

#[derive(Clone)]
pub enum MaterialType {
    Phong {
        diffuse: f32,
        specular: f32,
        shininess: f32,
    },
    None,
}

#[derive(Clone)]
pub struct Material {
    material: MaterialType,
    texture: TextureType,
}

impl TextureType {
    pub fn new_texture<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<std::path::Path>,
    {
        let image = image::io::Reader::open(path)?.decode()?;
        let buf = image.to_rgb();
        Ok(TextureType::Texture(Rc::new(buf)))
    }

    pub fn new_flat(color: Color) -> Self {
        TextureType::Flat(color)
    }

    fn sample<T: Object>(&self, object: &T, intersection_point: Point3<f32>) -> Color {
        match self {
            TextureType::Texture(buf) => {
                // TODO: Add options for wrapping/clamping and filter type.
                let width = buf.width() as f32;
                let height = buf.height() as f32;
                let uv = object.get_uv(intersection_point);
                let (u, v) = (uv.x, uv.y);
                // Wrap uv coordinates.
                let (u, v) = (u.rem_euclid(1.0), v.rem_euclid(1.0));
                let x = (u * width).trunc();
                let y = (v * height).trunc();
                let x = clamp(x, 0.0, width - 1.0) as u32;
                let y = clamp(y, 0.0, height - 1.0) as u32;
                let pixel = buf.get_pixel(x, y);
                let (r, g, b) = (pixel[0], pixel[1], pixel[2]);
                let r = (r as f32) / 255.0;
                let g = (g as f32) / 255.0;
                let b = (b as f32) / 255.0;
                Color::rgb(r, g, b)
            }
            TextureType::Flat(color) => *color,
            TextureType::None => Color::rgb(0.5, 0.5, 0.5),
        }
    }
}

impl Clone for TextureType {
    // TODO: Make sure this is the right thing to do.
    fn clone(&self) -> Self {
        match self {
            TextureType::Texture(buf) => TextureType::Texture(Rc::clone(buf)),
            TextureType::Flat(color) => TextureType::Flat(*color),
            TextureType::None => TextureType::None,
        }
    }
}

impl MaterialType {
    pub fn new_phong(diffuse: f32, specular: f32, shininess: f32) -> Self {
        MaterialType::Phong {
            diffuse,
            specular,
            shininess,
        }
    }
}

impl Material {
    pub fn new(material: MaterialType, texture: TextureType) -> Self {
        Material { material, texture }
    }
    pub fn get_color<T: Object>(
        &self,
        incoming_ray: Ray,
        t: f32,
        object: &T,
        lights: Vec<&Light>,
        _world: &World,
    ) -> Color {
        match self.material {
            MaterialType::Phong {
                diffuse,
                specular,
                shininess,
            } => {
                let intersection_point = incoming_ray.get_point_on_ray(t).into();
                let normal = object.get_normal(intersection_point);
                let surface_color = self.texture.sample(object, intersection_point);
                lights
                    .iter()
                    .map(|light| {
                        // TODO: Either add ambient component here, or create a new light type.
                        let light_ray = light.get_light_ray(intersection_point);
                        // TODO: Give falloff code to Light.
                        let falloff = 5.0
                            / (0.001 + InnerSpace::magnitude2(intersection_point - light.position));
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
                            * (diffuse * diffuse_intensity
                                + specular * specular_intensity.powf(shininess))
                            * light_color
                    })
                    .fold((0.0, 0.0, 0.0, 0.0).into(), |acc, x| acc + x)
            }
            MaterialType::None => Color::rgb(0.5, 0.5, 0.5),
        }
    }
}
