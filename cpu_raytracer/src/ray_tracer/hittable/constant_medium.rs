use super::{Aabb, HitRecord, Hittable, Material, RayAreaInfo};
use crate::prelude::*;
use cgmath::{Point2, Point3, Vector3};
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
        let mut hit1 = self
            .boundary
            .hit(ray, -1.0 * 10000000000.0, 10000000000.0)?;

        let mut hit2 = self.boundary.hit(ray, hit1.t + 0.0001, 10000000000.0)?;

        if hit1.t < t_min {
            hit1.t = t_min;
        }
        if hit2.t > t_max {
            hit2.t = t_max
        }

        if hit1.t >= hit2.t {
            return None;
        }
        if hit1.t < 0.0 {
            hit1.t = 0.0;
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
    fn prob(&self, ray: Ray) -> f32 {
        todo!()
    }
    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        todo!()
    }
}