use super::ray::Ray;
use cgmath::{Matrix4, Point3, SquareMatrix, Vector3};
use rand::Rng;

pub struct Camera {
    camera_to_world: Matrix4<f32>,
    pub width: u32,
    pub height: u32,
}

impl Camera {
    pub fn new(
        width: u32,
        height: u32,
        eye: Point3<f32>,
        at: Point3<f32>,
        up: Vector3<f32>,
    ) -> Camera {
        let world_to_camera = Matrix4::look_at(eye, at, up);
        let camera_to_world = world_to_camera.invert().unwrap();
        Camera {
            width,
            height,
            camera_to_world,
        }
    }

    /// Generates a ray for the given pixel location.
    /// pixel_x should be in (0, width)
    /// pixel_y should be in (0, height)
    /// rng can be None if no randomness should be added, else a rng
    pub fn generate_ray<R: Rng>(&self, pixel_x: u32, pixel_y: u32, rng: Option<&mut R>) -> Ray {
        // TODO: This only works for a square screen
        let (dx, dy) = match rng {
            None => { (0., 0.) }
            Some(rng) => { (rng.gen::<f32>() / 2., rng.gen::<f32>() / 2.) }
        };
        // Pixel (0, 0) is in the top left corner.
        let x = (pixel_x as f32 + dx) / (self.width as f32) - 0.5;
        let y = 0.5 - (pixel_y as f32 + dy) / (self.height as f32);
        let dist = -1.0; // TODO: Something something focal length?
        let position = (x, y, dist).into();
        let direction = (x, y, dist).into();
        let ray = Ray::new(position, direction);
        // Transform ray into world space.
        ray.transform_using(&self.camera_to_world)
    }
}
