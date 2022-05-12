use super::Ray;
use crate::prelude::*;
use crate::ray_tracer::hittable::Hittable;
use cgmath::Point3;

use crate::ray_tracer::HitRecord;
use std::{
    cmp::{max, min, Ordering},
    rc::Rc,
};
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub minimum: Point3<f32>,
    pub maximum: Point3<f32>,
}
impl AABB {
    pub fn hit(&self, ray: Ray, t_min: f32, t_max: f32) -> bool {
        for a in 0..3 {
            let inv_d = 1.0 / ray.direction[a];
            let mut t0 = (self.minimum[a] - ray.origin[a]) * inv_d;
            let mut t1 = (self.maximum[a] - ray.origin[a]) * inv_d;
            if inv_d < 0.0 {
                let t = t1;
                t1 = t0;
                t0 = t;
            }
            let t_min = if t0 > t_min { t0 } else { t_min };
            let t_max = if t1 < t_max { t1 } else { t_max };
            if t_max < t_min {
                return false;
            }
        }
        return true;
    }
    pub fn surrounding_box(self, box1: AABB) -> Self {
        Self {
            minimum: Point3 {
                x: p_min(self.minimum.x, box1.minimum.x),
                y: p_min(self.minimum.y, box1.minimum.y),
                z: p_min(self.minimum.z, box1.minimum.z),
            },
            maximum: Point3 {
                x: p_max(self.maximum.x, box1.maximum.x),
                y: p_max(self.maximum.y, box1.maximum.y),
                z: p_max(self.maximum.z, box1.maximum.z),
            },
        }
    }
}
pub struct BvhNode {
    left: Rc<dyn Hittable>,
    right: Rc<dyn Hittable>,
    bounding_box: AABB,
}
impl BvhNode {
    pub fn new(
        objects: Vec<Rc<dyn Hittable>>,
        start: usize,
        end: usize,
        time_0: f32,
        time_1: f32,
    ) -> Self {
        let axis = rand_u32(0, 2);
        let span = end - start;
        let comparator = if axis == 0 {
            |a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>| {
                Self::box_x_compare((*a).clone(), (*b).clone())
            }
        } else if axis == 1 {
            |a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>| {
                Self::box_y_compare((*a).clone(), (*b).clone())
            }
        } else {
            |a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>| {
                Self::box_z_compare((*a).clone(), (*b).clone())
            }
        };

        let (left, right) = if span == 1 {
            (objects[start].clone(), objects[start].clone())
        } else if span == 2 {
            if comparator(&objects[start].clone(), &objects[start + 1].clone()) == Ordering::Less {
                (objects[start].clone(), objects[start + 1].clone())
            } else {
                (objects[start + 1].clone(), objects[start].clone())
            }
        } else {
            let mut s_vec = (start..end).map(|i| objects[i].clone()).collect::<Vec<_>>();
            s_vec.sort_by(comparator);
            let middle = s_vec.len() / 2;
            let left: Rc<dyn Hittable> = Rc::new(BvhNode::new(
                (0..middle).map(|i| s_vec[i].clone()).collect(),
                0,
                middle,
                time_0,
                time_1,
            ));
            let right_objects = (middle..s_vec.len())
                .map(|i| s_vec[i].clone())
                .collect::<Vec<_>>();
            let right_objects_len = right_objects.len();
            let right: Rc<dyn Hittable> = Rc::new(BvhNode::new(
                right_objects,
                0,
                right_objects_len,
                time_0,
                time_1,
            ));
            (left, right)
        };
        let left_box = left
            .bounding_box(time_0, time_1)
            .expect("no bounding box for object");
        let right_box = right
            .bounding_box(time_0, time_1)
            .expect("no bounding box for object");
        let bounding_box = left_box.surrounding_box(right_box);

        Self {
            left,
            right,
            bounding_box,
        }
    }
    fn box_compare(a: Rc<dyn Hittable>, b: Rc<dyn Hittable>, axis: usize) -> Ordering {
        let a_box = a.bounding_box(0.0, 0.0);
        let b_box = b.bounding_box(0.0, 0.0);
        if a_box.is_none() || b_box.is_none() {
            panic!("bvh node does not have bounding box")
        } else {
            let a_box = a_box.unwrap();
            let b_box = b_box.unwrap();
            if a_box.minimum[axis] < b_box.minimum[axis] {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        }
    }
    fn box_x_compare(a: Rc<dyn Hittable>, b: Rc<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 0)
    }
    fn box_y_compare(a: Rc<dyn Hittable>, b: Rc<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 1)
    }
    fn box_z_compare(a: Rc<dyn Hittable>, b: Rc<dyn Hittable>) -> Ordering {
        Self::box_compare(a, b, 2)
    }
}
impl Hittable for BvhNode {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        if !self.bounding_box.hit(ray.clone(), t_min, t_max) {
            None
        } else {
            if let Some(left_hit) = self.left.hit(ray, t_min, t_max) {
                if let Some(right_hit) = self.right.hit(ray, t_min, left_hit.t) {
                    Some(right_hit)
                } else {
                    Some(left_hit)
                }
            } else if let Some(hit) = self.right.hit(ray, t_min, t_max) {
                Some(hit)
            } else {
                None
            }
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(self.bounding_box.clone())
    }
}
