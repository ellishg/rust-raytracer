use super::ray::Ray;
use cgmath::Transform;
use cgmath::{Matrix4, Point3, Vector3};

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
        let camera_to_world = Transform::inverse_transform(&world_to_camera).unwrap();
        Camera {
            width,
            height,
            camera_to_world,
        }
    }

    pub fn generate_ray(&self, pixel_x: u32, pixel_y: u32) -> Ray {
        // TODO: This only works for a square screen
        // Pixel (0, 0) is in the top left corner.
        let x = (pixel_x as f32) / (self.width as f32) - 0.5;
        let y = 0.5 - (pixel_y as f32) / (self.height as f32);
        let dist = -1.0; // TODO: Something something focal length?
        let position = (x, y, dist).into();
        let direction = (x, y, dist).into();
        let ray = Ray::new(position, direction);
        // Transform ray into world space.
        ray.transform_using(self.camera_to_world)
    }
}
