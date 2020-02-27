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

    fn trace_ray(&self, ray: Ray) -> Color {
        // Transform ray into world space.
        let ray = ray.transform_using(self.camera.camera_to_world);
        let closest_object = self
            .objects
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
            });
        if let Some((object, t)) = closest_object {
            // Compute the color of the object that the ray first hits.
            object.get_color(self, ray, t)
        } else {
            // If the ray hits nothing, return the background color.
            self.background_color
        }
    }
}
