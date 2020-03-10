use cgmath::MetricSpace;
use cgmath::{Point3, Vector4};
use image;
use std::error::Error;
use std::path::Path;
use time;

use super::camera::Camera;
use super::color::Color;
use super::light::Light;
use super::object::Object;
use super::ray::Ray;
use rand::rngs::ThreadRng;

pub struct World {
    camera: Camera,
    objects: Vec<Object>,
    pub lights: Vec<Light>,
    background_color: Color,
    rng: ThreadRng,
}

impl World {
    pub fn new(camera: Camera, background_color: Color) -> World {
        World {
            camera,
            objects: vec![],
            lights: vec![],
            background_color,
            rng: rand::thread_rng(),
        }
    }

    pub fn add_object(&mut self, object: Object) {
        self.objects.push(object);
    }

    pub fn add_light(&mut self, light: Light) {
        self.lights.push(light);
    }

    /// Render the world scene to a png file with the given filename.
    /// The screen is treated as a width 1 square centered at the camera eye.
    /// TODO: Add options to control up sampling and down sampling
    pub fn render<P>(&mut self, path: P, samples_per_pixel: u16) -> Result<(), Box<dyn Error>>
    where
        P: AsRef<Path>,
    {
        assert!(samples_per_pixel != 0);
        let instant = time::Instant::now();

        // TODO: Make this multi-threaded.
        let pixels: Vec<Vec<Color>> = (0..self.camera.width)
            .into_iter()
            .map(|x| {
                (0..self.camera.height)
                    .into_iter()
                    .map(|y| {
                        let rgb_sum = (0..samples_per_pixel)
                            .into_iter()
                            .map(|_| {
                                let rng = if samples_per_pixel == 1 {
                                    None
                                } else {
                                    Some(&mut self.rng)
                                };
                                let ray = self.camera.generate_ray(x, y, rng);
                                let color = self.trace_ray(ray);
                                color.to_vec()
                            })
                            .fold(Vector4::new(0., 0., 0., 0.), |acc, x| acc + x);
                        let res = rgb_sum / samples_per_pixel.into();
                        Color::rgba(res.x, res.y, res.z, res.w)
                    })
                    .collect()
            })
            .collect();
        let image = image::ImageBuffer::from_fn(self.camera.width, self.camera.height, |x, y| {
            let (r, g, b) = pixels[x as usize][y as usize].get_rgb();
            let pixel: image::Rgb<_> = [r, g, b].into();
            pixel
        });
        image.save(path)?;

        debug!(
            "Rendered image in {} seconds.",
            instant.elapsed().as_seconds_f32()
        );
        Ok(())
    }

    pub fn get_closest_intersection(&self, ray: Ray) -> Option<(&Object, f32)> {
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
        if let Some((object, t)) = self.get_closest_intersection(ray) {
            // Compute the color of the object that the ray first hits.
            let intersection_point: Point3<f32> = ray.get_point_on_ray(t).into();
            let illuminating_lights = self
                .lights
                .iter()
                .filter(|light| {
                    let light_ray = light.get_light_ray(intersection_point);
                    if let Some((_, t)) = self.get_closest_intersection(light_ray) {
                        // TODO: Figure out a better way to detect shadows.
                        // TODO: This should be in light struct.
                        let epsilon_squared = 0.1;
                        if intersection_point.distance2(light_ray.get_point_on_ray(t).into())
                            > epsilon_squared
                        {
                            return false;
                        }
                    }
                    true
                })
                .collect();
            object.get_color(ray, t, illuminating_lights, self)
        } else {
            // If the ray hits nothing, return the background color.
            self.background_color
        }
    }
}
