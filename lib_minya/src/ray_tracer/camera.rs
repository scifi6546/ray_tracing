use crate::prelude::*;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Vector3};
#[derive(Clone, Debug)]
pub struct Camera {
    origin: Point3<RayScalar>,
    lower_left_corner: Point3<RayScalar>,
    horizontal: Vector3<RayScalar>,
    vertical: Vector3<RayScalar>,
    u: Vector3<RayScalar>,
    v: Vector3<RayScalar>,
    lens_radius: RayScalar,
    start_time: RayScalar,
    end_time: RayScalar,
}
impl Camera {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        aspect_ratio: RayScalar,
        fov: RayScalar,
        origin: Point3<RayScalar>,
        look_at: Point3<RayScalar>,
        up_vector: Vector3<RayScalar>,
        aperture: RayScalar,
        focus_distance: RayScalar,
        start_time: RayScalar,
        end_time: RayScalar,
    ) -> Self {
        let theta = fov * RayScalar::PI() / 180.0;
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
    pub fn get_ray(&self, u: RayScalar, v: RayScalar) -> Ray {
        let rd = self.lens_radius * Self::random_in_unit_disk();
        let offset = self.u * rd.x + self.v * rd.y;
        Ray {
            origin: self.origin,
            direction: self.lower_left_corner + u * self.horizontal + v * self.vertical
                - self.origin
                - offset,
            time: rand_scalar(self.start_time, self.end_time),
        }
    }
    fn random_in_unit_disk() -> Vector3<RayScalar> {
        loop {
            let p = Vector3::new(rand_scalar(-1.0, 1.0), rand_scalar(-1.0, 1.0), 0.0);
            if p.dot(p) < 1.0 {
                return p;
            }
        }
    }
    pub fn start_time(&self) -> RayScalar {
        self.start_time
    }
    pub fn end_time(&self) -> RayScalar {
        self.end_time
    }
}
