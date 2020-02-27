use cgmath::Point3;
use cgmath::{InnerSpace, MetricSpace};
use image;
use std::error::Error;
use std::fs::File;

use super::camera::Camera;
use super::color::Color;
use super::light::Light;
use super::object::Object;
use super::ray::Ray;

pub struct World {
    camera: Camera,
    objects: Vec<Box<dyn Object>>,
    pub lights: Vec<Light>,
    background_color: Color,
}

impl World {
    pub fn new(camera: Camera, background_color: Color) -> World {
        World {
            camera,
            objects: vec![],
            lights: vec![],
            background_color,
        }
    }

    pub fn add_object(&mut self, object: Box<dyn Object>) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    // TODO: Add options to control up sampling and down sampling
    pub fn render(&self, filename: &str) -> Result<(), Box<dyn Error>> {
        let file = File::create(filename)?;
        let png_encoder = image::png::PNGEncoder::new(file);
        let width = self.camera.width;
        let height = self.camera.height;
        let data: Vec<u8> = (0..width * height)
            .into_iter()
            .flat_map(|i| {
                let ray = self.camera.generate_ray(i % width, i / width);
                let color = self.trace_ray(ray);
                let (r, g, b) = color.get_rgb();
                vec![r, g, b]
            })
            .collect();
        png_encoder.encode(data.as_slice(), width, height, image::ColorType::Rgb8)?;
        Ok(())
    }

    pub fn get_closest_intersection(&self, ray: Ray) -> Option<(&Box<dyn Object>, f32)> {
        self.objects
            .iter()
            .filter_map(|object| match object.get_intersection(ray) {
                Some(t) => Some((object, t)),
                None => None,
            })
            // Just a hacky way to find the smallest t value.
            .min_by(|(_, t_left), (_, t_right)| {
                t_left
                    .partial_cmp(t_right)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
    }

    fn trace_ray(&self, ray: Ray) -> Color {
        // Transform ray into world space.
        let ray = ray.transform_using(self.camera.camera_to_world);
        if let Some((object, t)) = self.get_closest_intersection(ray) {
            // Compute the color of the object that the ray first hits.
            let intersection_point: Point3<f32> = ray.get_point_on_ray(t).into();
            let total_light_color = self
                .lights
                .iter()
                .filter(|light| {
                    let light_ray = light.get_light_ray(intersection_point);
                    if let Some((_, t)) = self.get_closest_intersection(light_ray) {
                        // TODO: Figure out a better way to detect shadows.
                        let epsilon_squared = 0.1;
                        if intersection_point.distance2(light_ray.get_point_on_ray(t).into())
                            > epsilon_squared
                        {
                            return false;
                        }
                    }
                    true
                })
                .map(|light| {
                    let light_ray = light.get_light_ray(intersection_point);
                    // TODO: Give falloff code to Light.
                    let falloff =
                        5.0 / (0.001 + InnerSpace::magnitude2(intersection_point - light.position));
                    let intensity =
                        object.get_light_intensity(intersection_point, light_ray.direction);
                    falloff * intensity * light.color
                })
                .fold(Color::rgba(0.0, 0.0, 0.0, 0.0), |acc, x| acc + x);
            object.get_color(total_light_color, ray, t)
        } else {
            // If the ray hits nothing, return the background color.
            self.background_color
        }
    }
}
