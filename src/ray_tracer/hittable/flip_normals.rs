use super::{HitRecord, Hittable, AABB};
use crate::prelude::*;
use std::rc::Rc;
pub struct FlipNormals {
    pub item: Rc<dyn Hittable>,
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
