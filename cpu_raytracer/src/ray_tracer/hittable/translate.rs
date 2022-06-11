use super::{Aabb, HitRecord, Hittable,RayAreaInfo};
use crate::prelude::*;
use cgmath::{Vector3,Point3};

pub struct Translate<T: Hittable> {
    pub item: T,
    pub offset: Vector3<f32>,
}
impl<T: Hittable> Hittable for Translate<T> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray {
            origin: ray.origin - self.offset,
            direction: ray.direction,
            time: ray.time,
        };
        if let Some(record) = self.item.hit(&moved_ray, t_min, t_max) {
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

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        self.item.bounding_box(time_0, time_1).map(|b| Aabb {
            minimum: b.minimum + self.offset,
            maximum: b.maximum + self.offset,
        })
    }
    fn prob(&self, ray: Ray) -> f32{
        self.item.prob( Ray {
            origin: ray.origin - self.offset,
            direction: ray.direction,
            time: ray.time,
        })
    }
    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo{
        self.item.generate_ray_in_area(origin-self.offset,time)
    }
}
