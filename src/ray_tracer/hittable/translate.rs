use super::{HitRecord, Hittable, AABB};
use crate::prelude::*;
use cgmath::Vector3;
use std::rc::Rc;
pub struct Translate {
    pub item: Rc<dyn Hittable>,
    pub offset: Vector3<f32>,
}
impl Hittable for Translate {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray {
            origin: ray.origin - self.offset,
            direction: ray.direction,
            time: ray.time,
        };
        if let Some(mut record) = self.item.hit(&moved_ray, t_min, t_max) {
            Some(HitRecord::new(
                &moved_ray,
                record.position + self.offset,
                record.normal,
                record.t,
                record.uv,
                record.material.clone(),
            ))
        } else {
            None
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        if let Some(b) = self.item.bounding_box(time_0, time_1) {
            Some(AABB {
                minimum: b.minimum + self.offset,
                maximum: b.maximum + self.offset,
            })
        } else {
            None
        }
    }
}
