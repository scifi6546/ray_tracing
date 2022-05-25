use super::{Aabb, HitRecord, Hittable, Material};
use crate::prelude::*;
use cgmath::{Point2, Vector3};
use std::{cell::RefCell, rc::Rc};
pub struct ConstantMedium {
    boundary: Rc<dyn Hittable>,
    phase_function: Rc<RefCell<dyn Material>>,
    neg_inv_density: f32,
}
impl ConstantMedium {
    pub fn new(
        boundary: Rc<dyn Hittable>,
        phase_function: Rc<RefCell<dyn Material>>,
        density: f32,
    ) -> Self {
        Self {
            boundary,
            phase_function,
            neg_inv_density: -1.0 / density,
        }
    }
}
impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let hit_res1 = self.boundary.hit(ray, -1.0 * 10000000000.0, 10000000000.0);
        if hit_res1.is_none() {
            return None;
        }
        let mut hit1 = hit_res1.unwrap();
        let hit_res2 = self.boundary.hit(ray, hit1.t + 0.0001, 10000000000.0);
        if hit_res2.is_none() {
            return None;
        }
        let mut hit2 = hit_res2.unwrap();

        if hit1.t < t_min {
            hit1.t = t_min;
        }
        if hit2.t > t_max {
            hit2.t = t_max
        }

        if hit1.t >= hit2.t {
            if debug() {
                println!(
                    "none, hit1.t = {}, hit2.t = {}, t_min: {} t_max: {}",
                    hit1.t, hit2.t, t_min, t_max
                );
            }
            return None;
        }
        if hit1.t < 0.0 {
            hit1.t = 0.0;
        }

        if debug() {
            println!(
                "hit!!!   hit1.t = {}, hit2.t = {}, t_min: {} t_max: {}",
                hit1.t, hit2.t, t_min, t_max
            );
        }
        let ray_length = {
            let d = ray.direction;
            (d.x * d.x + d.y * d.y + d.z * d.z).sqrt()
        };

        let distance_inside_boundary = (hit2.t - hit1.t) * ray_length;
        let hit_distance = self.neg_inv_density * rand_f32(0.0, 1.0).ln();
        if hit_distance > distance_inside_boundary {
            return None;
        }
        let t = hit1.t + hit_distance / ray_length;
        let position = ray.at(t);
        if debug() {
            println!(
                "hit distance: {} t: {} position: <{},{},{}>",
                hit_distance, t, position.x, position.y, position.z
            );
        }
        Some(HitRecord {
            position,
            normal: Vector3::new(1.0, 0.0, 0.0),
            t,
            front_face: false,
            uv: Point2::new(0.0, 0.0),
            material: self.phase_function.clone(),
        })
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        self.boundary.bounding_box(time_0, time_1)
    }
}
