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
struct BVH_Node {
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
}
impl BVH_Node {
    pub fn new(objects: Vec<Box<dyn Hittable>>, start: usize) -> Self {
        todo!()
    }
}
