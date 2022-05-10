use super::Ray;
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
#[derive(Clone, Debug)]
pub struct Camera {
    origin: Point3<f32>,
    lower_left_corner: Point3<f32>,
    horizontal: Vector3<f32>,
    vertical: Vector3<f32>,
    u: Vector3<f32>,
    v: Vector3<f32>,
    lens_radius: f32,
    start_time: f32,
    end_time: f32,
}
impl Camera {
    pub fn new(
        aspect_ratio: f32,
        fov: f32,
        origin: Point3<f32>,
        look_at: Point3<f32>,
        up_vector: Vector3<f32>,
        aperture: f32,
        focus_distance: f32,
        start_time: f32,
        end_time: f32,
    ) -> Self {
        let theta = fov * f32::PI() / 180.0;
        let h = (theta / 2.0).tan();
        let world_height = 2.0 * h;

        let world_width = aspect_ratio * world_height;

        let w = (origin - look_at).normalize();
        let u = up_vector.cross(w).normalize();
        let v = w.cross(u);
        let horizontal = focus_distance * world_width * u;

        let vertical = focus_distance * world_height * v;

        Self {
            origin,
            horizontal,
            vertical,
            lower_left_corner: origin - horizontal / 2.0 - vertical / 2.0 - focus_distance * w,
            u,
            v,
            lens_radius: aperture / 2.0,
            start_time,
            end_time,
        }
    }
    pub fn get_ray(&self, u: f32, v: f32) -> Ray {
        let rd = self.lens_radius * Self::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin
                - offset,
            time: rand_f32(self.start_time, self.end_time),
        }
    }
    fn random_in_unit_disk() -> Vector3<f32> {
        loop {
            let p = Vector3::new(rand_f32(-1.0, 1.0), rand_f32(-1.0, 1.0), 0.0);
            if p.dot(p) < 1.0 {
                return p;
            }
        }
    }
    pub fn start_time(&self) -> f32 {
        self.start_time
    }
    pub fn end_time(&self) -> f32 {
        self.end_time
    }
}
