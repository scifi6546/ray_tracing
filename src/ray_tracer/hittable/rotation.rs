use super::{HitRecord, Hittable, AABB};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, Point3, Vector3};
use std::rc::Rc;
pub struct RotateY {
    item: Rc<dyn Hittable>,
    sin_theta: f32,
    cos_theta: f32,

    item_box: Option<AABB>,
}
impl RotateY {
    pub fn new(item: Rc<dyn Hittable>, angle: f32) -> Self {
        let radians = angle * f32::PI() / 180.0;
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let item_box = if let Some(item_box) = item.bounding_box(0.0, 1.0) {
            let mut min = Point3::new(f32::MAX, f32::MAX, f32::MAX);
            let mut max = Point3::new(f32::MIN, f32::MIN, f32::MIN);
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x =
                            i as f32 * item_box.maximum.x + (1.0 - i as f32) * item_box.minimum.x;
                        let y =
                            j as f32 * item_box.maximum.y + (1.0 - j as f32) * item_box.minimum.y;
                        let z =
                            k as f32 * item_box.minimum.z + (1.0 - k as f32) * item_box.minimum.z;
                        let new_x = cos_theta * x + sin_theta * z;
                        let new_z = -sin_theta * x + cos_theta * z;
                        let tester = Vector3::new(new_x, y, new_z);
                        for c in 0..3 {
                            min[c] = p_min(min[c], tester[c]);
                            max[c] = p_max(max[c], tester[c]);
                        }
                    }
                }
            }
            Some(AABB {
                maximum: max,
                minimum: min,
            })
        } else {
            None
        };
        Self {
            item,
            sin_theta,
            cos_theta,
            item_box,
        }
    }
}
impl Hittable for RotateY {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut origin = ray.origin;
        let mut direction = ray.direction;
        origin.x = self.cos_theta * ray.origin.x - self.sin_theta * ray.origin.z;
        origin.z = self.sin_theta * ray.origin.x + self.cos_theta * ray.origin.z;

        direction.x = self.cos_theta * ray.direction.x - self.sin_theta * ray.direction.z;
        direction.z = self.sin_theta * ray.direction.x + self.cos_theta * ray.direction.z;

        let rotated_ray = Ray {
            origin,
            direction,
            time: ray.time,
        };
        if let Some(hit) = self.item.hit(&rotated_ray, t_min, t_max) {
            let mut position = hit.position;
            let mut normal = hit.normal;
            position.x = self.cos_theta * hit.position.x + self.sin_theta * hit.position.z;
            position.z = -1.0 * self.sin_theta * hit.position.x + self.cos_theta * hit.position.z;

            normal.x = self.cos_theta * hit.normal.x + self.sin_theta * hit.normal.z;
            normal.z = -1.0 * self.sin_theta * hit.normal.x + self.cos_theta * hit.normal.z;

            Some(HitRecord::new(
                &rotated_ray,
                position,
                normal,
                rotated_ray.time,
                hit.uv,
                hit.material.clone(),
            ))
        } else {
            None
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        self.item_box
    }
}
