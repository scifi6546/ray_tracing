use super::{Aabb, HitRay, HitRecord, Hittable, Material, MaterialEffect, RayAreaInfo};
use crate::prelude::*;
use cgmath::{prelude::*, Point2, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;

pub struct ConstantMedium {
    boundary: Box<dyn Hittable>,
    phase_function: Box<dyn Material>,
    neg_inv_density: RayScalar,
}
impl Clone for ConstantMedium {
    fn clone(&self) -> Self {
        Self {
            boundary: clone_box(self.boundary.deref()),
            phase_function: clone_box(self.phase_function.deref()),
            neg_inv_density: self.neg_inv_density,
        }
    }
}
impl ConstantMedium {
    pub fn new(
        boundary: Box<dyn Hittable>,
        phase_function: Box<dyn Material>,
        density: RayScalar,
    ) -> Self {
        Self {
            boundary,
            phase_function,
            neg_inv_density: -1.0 / density,
        }
    }
}
impl Hittable for ConstantMedium {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
        let hit_distance = self.neg_inv_density * rand_scalar(0.0, 1.0).ln();
        if hit_distance > distance_inside_boundary {
            return None;
        }
        let t = hit1.t + hit_distance / ray_length;
        let position = ray.at(t);
        let hit_ray = HitRay {
            position,
            direction: ray.direction,
            normal: Vector3::unit_x(),
            front_face: true,
            uv: Point2::origin(),
        };
        let lighting = self.phase_function.emmit(&hit_ray);
        let material_effect = if lighting.is_some() {
            MaterialEffect::Emmit(lighting.unwrap())
        } else {
            let diffuse = self.phase_function.scatter(*ray, &hit_ray);
            if diffuse.is_some() {
                MaterialEffect::Scatter(diffuse.unwrap())
            } else {
                MaterialEffect::NoEmmit
            }
        };

        Some(HitRecord {
            position,
            normal: Vector3::new(1.0, 0.0, 0.0),
            t,
            front_face: false,
            uv: Point2::new(0.0, 0.0),
            material_effect,
        })
    }

    fn bounding_box(&self, time_0: RayScalar, time_1: RayScalar) -> Option<Aabb> {
        self.boundary.bounding_box(time_0, time_1)
    }
    fn prob(&self, ray: Ray) -> RayScalar {
        self.boundary.prob(ray)
    }
    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        self.boundary.generate_ray_in_area(origin, time)
    }
    fn name(&self) -> String {
        "Constant Medium".to_string()
    }
}
