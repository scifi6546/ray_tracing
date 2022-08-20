use super::{Aabb, HitRecord, Hittable};
use crate::prelude::*;
use crate::ray_tracer::hittable::RayAreaInfo;
use cgmath::Point3;
use dyn_clone::clone_box;
use std::ops::Deref;
use std::rc::Rc;
pub struct FlipNormals {
    pub item: Box<dyn Hittable>,
}
impl Clone for FlipNormals {
    fn clone(&self) -> Self {
        Self {
            item: clone_box(self.item.deref()),
        }
    }
}
impl Hittable for FlipNormals {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if let Some(mut hit) = self.item.hit(ray, t_min, t_max) {
            hit.normal = -1.0 * hit.normal;
            hit.front_face = !hit.front_face;
            Some(hit)
        } else {
            None
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        self.item.bounding_box(time_0, time_1)
    }
    fn prob(&self, ray: Ray) -> f32 {
        self.item.prob(ray)
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        self.item.generate_ray_in_area(origin, time)
    }
}
