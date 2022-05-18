use super::{HitRecord, Hittable, Light, AABB};
use crate::prelude::*;
use cgmath::Point3;
use std::rc::Rc;

pub struct FlipNormals {
    pub item: Rc<dyn Light>,
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

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        self.item.bounding_box(time_0, time_1)
    }
}
impl Light for FlipNormals {
    fn prob(&self, ray: Ray) -> f32 {
        self.item.prob(ray)
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> Ray {
        self.item.generate_ray_in_area(origin, time)
    }
}