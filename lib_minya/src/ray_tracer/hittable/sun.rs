use super::Hittable;
use crate::prelude::*;
use crate::ray_tracer::bvh::Aabb;
use crate::ray_tracer::hittable::{HitRecord, RayAreaInfo};
use cgmath::Point3;
#[derive(Clone)]
pub struct Sun {
    /// radius in degrees
    pub radius: f32,
}
impl Hittable for Sun {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        todo!()
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        None
    }

    fn prob(&self, ray: Ray) -> f32 {
        todo!()
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        todo!()
    }
}
