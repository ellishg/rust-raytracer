use cgmath::{InnerSpace, Point3, Vector3};
use image;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

use super::color::Color;
use super::light::{Light, LightType};
use super::object::Object;
use super::ray::Ray;
use super::utils::{clamp, reflect, refract};
use super::world::World;

pub enum TextureType {
    Texture(Arc<image::RgbImage>),
    Flat(Color),
    None,
}

#[derive(Clone)]
pub enum MaterialType {
    Composition(Vec<(MaterialType, f32)>),
    Phong {
        diffuse: f32,
        specular: f32,
        shininess: f32,
    },
    Reflective,
    Refractive(f32),
    None,
}

#[derive(Clone)]
pub struct Material {
    material_type: MaterialType,
    texture_type: TextureType,
}

impl TextureType {
    pub fn new_texture<P>(path: P) -> Result<Self, Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        let image = image::io::Reader::open(path)?.decode()?;
        let buf = image.to_rgb();
        Ok(TextureType::Texture(Arc::new(buf)))
    }

    pub fn new_flat(color: Color) -> Self {
        TextureType::Flat(color)
    }

    fn sample(&self, object: &Object, intersection_point: Point3<f32>) -> Color {
        match self {
            TextureType::Texture(buf) => {
                // TODO: Add options for wrapping/clamping and filter type.
                let width = buf.width() as f32;
                let height = buf.height() as f32;
                let (u, v) = object.get_uv(intersection_point).into();
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
    fn clone(&self) -> Self {
        match self {
            TextureType::Texture(buf) => TextureType::Texture(Arc::clone(buf)),
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

    fn get_phong_multiple(
        light_direction: Vector3<f32>,
        normal: Vector3<f32>,
        incoming_direction: Vector3<f32>,
        diffuse: f32,
        specular: f32,
        shininess: f32,
    ) -> f32 {
        debug_assert!(f32::abs(light_direction.magnitude() - 1.) < 1e-4);
        debug_assert!(f32::abs(normal.magnitude() - 1.) < 1e-4);
        debug_assert!(f32::abs(incoming_direction.magnitude() - 1.) < 1e-4);
        let reflection_vector = reflect(light_direction, normal);
        let specular_intensity = clamp(-reflection_vector.dot(incoming_direction), 0.0, 1.0);
        let diffuse_intensity = clamp(-light_direction.dot(normal), 0.0, 1.0);
        diffuse * diffuse_intensity + specular * specular_intensity.powf(shininess)
    }

    /// Returns the color of `object` at the point given by `incoming_ray.get_point_on_ray(t)`.
    ///
    /// All arguments are in world space coordinates.
    pub fn get_color(
        &self,
        surface_color: Color,
        incoming_ray: &Ray,
        t: f32,
        object: &Object,
        lights: Vec<&Light>,
        world: &World,
        max_depth: u16,
    ) -> Color {
        match self {
            MaterialType::Composition(materials) => materials
                .iter()
                .map(|(material, coefficient)| {
                    *coefficient
                        * material.get_color(
                            surface_color,
                            incoming_ray,
                            t,
                            object,
                            lights.clone(),
                            world,
                            max_depth,
                        )
                })
                .fold((0.0, 0.0, 0.0, 0.0).into(), |acc, x| acc + x),
            MaterialType::Phong {
                diffuse,
                specular,
                shininess,
            } => {
                let intersection_point = incoming_ray.get_point_on_ray(t).into();
                let normal = object.get_normal(intersection_point);
                lights
                    .iter()
                    .map(|light| {
                        let light_color = match light.light_type {
                            LightType::Ambient => light.color,
                            LightType::Point(position) => {
                                let light_dir = intersection_point - position;
                                // TODO: Give falloff code to Light.
                                let falloff = 5.0 / (0.001 + light_dir.magnitude2());
                                let phong_multiple = MaterialType::get_phong_multiple(
                                    light_dir.normalize(),
                                    normal,
                                    incoming_ray.get_direction(),
                                    *diffuse,
                                    *specular,
                                    *shininess,
                                );
                                phong_multiple * (falloff * light.color)
                            }
                            LightType::Directional(direction) => {
                                let phong_multiple = MaterialType::get_phong_multiple(
                                    direction,
                                    normal,
                                    incoming_ray.get_direction(),
                                    *diffuse,
                                    *specular,
                                    *shininess,
                                );
                                phong_multiple * light.color
                            }
                        };
                        surface_color * light_color
                    })
                    .fold((0.0, 0.0, 0.0, 0.0).into(), |acc, x| acc + x)
            }
            MaterialType::Reflective => {
                let intersection_point = incoming_ray.get_point_on_ray(t).into();
                let normal = object.get_normal(intersection_point);
                let reflection_direction = reflect(incoming_ray.get_direction(), normal);
                let reflected_ray = Ray::new(intersection_point, reflection_direction);
                // We move the ray forward slightly so that we don't intersect the same location.
                let reflected_ray = reflected_ray.offset(1e-4);
                world.trace_ray(&reflected_ray, max_depth)
            }
            MaterialType::Refractive(refraction_index) => {
                let intersection_point = incoming_ray.get_point_on_ray(t).into();
                let normal = object.get_normal(intersection_point);
                let refraction_direction =
                    refract(incoming_ray.get_direction(), normal, *refraction_index);
                let refracted_ray = Ray::new(intersection_point, refraction_direction);
                // We move the ray forward slightly so that we don't intersect the same location.
                let refracted_ray = refracted_ray.offset(1e-4);
                world.trace_ray(&refracted_ray, max_depth)
            }
            MaterialType::None => Color::rgb(0.5, 0.5, 0.5),
        }
    }
}

impl Material {
    pub fn new(material_type: MaterialType, texture_type: TextureType) -> Self {
        Material {
            material_type,
            texture_type,
        }
    }

    /// Returns the color of `object` at the point given by `incoming_ray.get_point_on_ray(t)`.
    ///
    /// All arguments are in world space coordinates.
    pub fn get_color(
        &self,
        incoming_ray: &Ray,
        t: f32,
        object: &Object,
        lights: Vec<&Light>,
        world: &World,
        max_depth: u16,
    ) -> Color {
        let intersection_point = incoming_ray.get_point_on_ray(t).into();
        let surface_color = self.texture_type.sample(object, intersection_point);
        self.material_type.get_color(
            surface_color,
            &incoming_ray,
            t,
            object,
            lights,
            world,
            max_depth,
        )
    }
}
