use cgmath::{MetricSpace, Vector4};
use image;
use std::error::Error;
use std::path::Path;
use std::sync::{mpsc, Arc};
use time;
use threadpool::ThreadPool;

use super::bvh::Bvh;
use super::camera::Camera;
use super::color::Color;
use super::light::Light;
use super::object::Object;
use super::ray::Ray;

/// Render to a png file with the given filename.
pub fn render<P>(
    camera: Camera,
    objects: Vec<Object>,
    lights: Vec<Light>,
    background_color: Color,
    samples_per_pixel: u16,
    max_ray_bounces: u16,
    path: P,
    num_threads: usize,
) -> Result<(), Box<dyn Error>>
where
    P: AsRef<Path>,
{
    assert!(samples_per_pixel != 0);
    let instant = time::Instant::now();

    let world = World::new(camera, objects, lights, background_color);
    let world = Arc::new(world);

    let (width, height) = (world.camera.width, world.camera.height);

    let pool = ThreadPool::new(num_threads);
    let (tx, rx) = mpsc::channel();
    for x in 0..width {
        let tx = tx.clone();
        let world = Arc::clone(&world);
        pool.execute(move || {
            let colors = (0..height).map(|y| {
                let mut rng = {
                    if samples_per_pixel == 1 {
                        None
                    } else {
                        Some(rand::thread_rng())
                    }
                };

                let rgb_sum = (0..samples_per_pixel)
                    .into_iter()
                    .map(|_| {
                        let ray = world.camera.generate_ray(x, y, rng.as_mut());
                        let color = world.trace_ray(&ray, max_ray_bounces);
                        color.to_vec()
                    })
                    .fold(Vector4::new(0., 0., 0., 0.), |acc, x| acc + x);
                let res = rgb_sum / samples_per_pixel.into();
                Color::rgba(res.x, res.y, res.z, res.w)
            }).collect();
            tx.send((x, colors)).unwrap();
        });
    }

    let mut pixels = vec![vec![Color::black(); height as usize]; width as usize];
    for _ in 0..width {
        let (x, colors) = rx.recv()?;
        pixels[x as usize] = colors;
    }

    let image = image::ImageBuffer::from_fn(width, height, |x, y| {
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

pub struct World {
    camera: Camera,
    bvh: Bvh,
    lights: Vec<Light>,
    background_color: Color,
}

impl World {
    pub fn new(
        camera: Camera,
        objects: Vec<Object>,
        lights: Vec<Light>,
        background_color: Color,
    ) -> World {
        let bvh = Bvh::new(objects, 10);
        World {
            camera,
            bvh,
            lights,
            background_color,
        }
    }

    /// Trace a ray in the world and return the color it should produce.
    /// `max_depth` is the maximum number of bounces we should compute for this ray.
    pub fn trace_ray(&self, ray: &Ray, max_depth: u16) -> Color {
        if max_depth == 0 {
            self.background_color
        } else if let Some((object, t)) = self.bvh.get_closest_intersection(ray) {
            // Compute the color of the object that the ray first hits.
            let intersection_point = ray.get_point_on_ray(t).into();
            let illuminating_lights = self
                .lights
                .iter()
                .filter(|light| {
                    let light_ray = light.get_light_ray(intersection_point);
                    let light_to_object_t =
                        intersection_point.distance(light_ray.get_point_on_ray(0.0).into());
                    // TODO: Shadows don't work correctly with reflective or refractive surfaces.
                    if let Some((_, shadow_t)) = self.bvh.get_closest_intersection(&light_ray) {
                        let epsilon = 1e-4;
                        let is_in_shadow = shadow_t + epsilon < light_to_object_t;
                        !is_in_shadow
                    } else {
                        false
                    }
                })
                .collect();
            object.get_color(&ray, t, illuminating_lights, self, max_depth - 1)
        } else {
            // If the ray hits nothing, return the background color.
            self.background_color
        }
    }
}
