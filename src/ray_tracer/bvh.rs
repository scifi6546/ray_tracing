use super::Ray;
use crate::prelude::*;
use crate::ray_tracer::hittable::Hittable;
use cgmath::Point3;
use std::cmp::{max, min};

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
struct BvhNode {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
}
impl BvhNode {
    pub fn new(objects: Vec<Box<dyn Hittable>>, start: usize, end: usize) -> Self {
        let axis = rand_u32(0, 2);
        let span = end - start;

        todo!()
    }
    fn box_compare(a: Box<dyn Hittable>, b: Box<dyn Hittable>, axis: usize) -> bool {
        let a_box = a.bounding_box(0.0, 0.0);
        let b_box = b.bounding_box(0.0, 0.0);
        if a_box.is_none() || b_box.is_none() {
            panic!("bvh node does not have bounding box")
        } else {
            let a_box = a_box.unwrap();
            let b_box = b_box.unwrap();
            a_box.minimum[axis] < b_box.minimum[axis]
        }
    }
    fn box_x_compare(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> bool {
        Self::box_compare(a, b, 0)
    }
    fn box_y_compare(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> bool {
        Self::box_compare(a, b, 1)
    }
    fn box_z_compare(a: Box<dyn Hittable>, b: Box<dyn Hittable>) -> bool {
        Self::box_compare(a, b, 2)
    }
}
