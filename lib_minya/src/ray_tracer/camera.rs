use super::ray_tracer_info::{Entity, EntityField};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, InnerSpace, Point3, Quaternion, Vector3};
use std::collections::HashMap;
/// info used to construct camera
#[derive(Clone, Debug, PartialEq)]
pub struct CameraInfo {
    pub aspect_ratio: RayScalar,
    pub fov: RayScalar,
    pub origin: Point3<RayScalar>,
    pub look_at: Point3<RayScalar>,
    pub up_vector: Vector3<RayScalar>,
    pub aperture: RayScalar,
    pub focus_distance: RayScalar,
    pub start_time: RayScalar,
    pub end_time: RayScalar,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Camera {
    origin: Point3<RayScalar>,
    lower_left_corner: Point3<RayScalar>,
    horizontal: Vector3<RayScalar>,
    vertical: Vector3<RayScalar>,
    u: Vector3<RayScalar>,
    v: Vector3<RayScalar>,
    look_at: Point3<RayScalar>,
    lens_radius: RayScalar,
    start_time: RayScalar,
    end_time: RayScalar,
    up_vector: Vector3<RayScalar>,
    focus_distance: RayScalar,
    world_width: RayScalar,
    world_height: RayScalar,
}
impl Camera {
    pub fn new(info: CameraInfo) -> Self {
        let theta = info.fov * RayScalar::PI() / 180.0;
        let h = (theta / 2.0).tan();
        let world_height = 2.0 * h;

        let world_width = info.aspect_ratio * world_height;

        let (w, u, v) = Self::calculate_w_u_v(info.origin, info.look_at, info.up_vector);
        let horizontal = info.focus_distance * world_width * u;

        let vertical = info.focus_distance * world_height * v;

        Self {
            origin: info.origin,
            horizontal,
            vertical,
            lower_left_corner: info.origin
                - horizontal / 2.0
                - vertical / 2.0
                - info.focus_distance * w,
            u,
            v,
            lens_radius: info.aperture / 2.0,
            start_time: info.start_time,
            end_time: info.end_time,
            look_at: info.look_at,
            up_vector: info.up_vector,
            focus_distance: info.focus_distance,
            world_width,
            world_height,
        }
    }
    fn calculate_w_u_v(
        origin: Point3<RayScalar>,
        look_at: Point3<RayScalar>,
        up_vector: Vector3<RayScalar>,
    ) -> (Vector3<RayScalar>, Vector3<RayScalar>, Vector3<RayScalar>) {
        let w = (origin - look_at).normalize();
        let u = up_vector.cross(w).normalize();
        let v = w.cross(u);
        (w, u, v)
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
    fn set_look_at(&mut self, look_at: Point3<RayScalar>) {
        let (w, u, v) = Self::calculate_w_u_v(self.origin, look_at, self.up_vector);
        let horizontal = self.focus_distance * self.world_width * u;

        let vertical = self.focus_distance * self.world_height * v;
        self.lower_left_corner =
            self.origin - self.horizontal / 2.0 - vertical / 2.0 - self.focus_distance * w;
    }
}
impl Entity for Camera {
    fn name(&self) -> String {
        "camera".to_string()
    }
    fn fields(&self) -> HashMap<String, EntityField> {
        let mut map = HashMap::new();
        map.insert("origin".to_string(), EntityField::Point3(self.origin));
        map.insert("look_at".to_string(), EntityField::Point3(self.look_at));
        map
    }
    fn set_field(&mut self, key: String, value: EntityField) {
        match key.as_str() {
            "origin" => match value {
                EntityField::Point3(p) => self.origin = p,
                _ => panic!("invalid field type"),
            },
            "look_at" => match value {
                EntityField::Point3(p) => self.set_look_at(p),
                _ => panic!("invalid field type"),
            },
            _ => panic!("invalid field: {}", key),
        };
    }
}
