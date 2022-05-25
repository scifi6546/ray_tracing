use super::{Aabb, HitRecord, Hittable, Material};
use crate::prelude::*;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};
#[derive(Clone)]
pub struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub material: Rc<RefCell<dyn Material>>,
}

impl Hittable for Sphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let rel_origin = ray.origin - self.origin;
        let a = ray.direction.dot(ray.direction);
        let half_b = rel_origin.dot(ray.direction);
        let c = rel_origin.dot(rel_origin) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_d = discriminant.sqrt();
        let mut root = (-1.0 * half_b - sqrt_d) / a;
        if root < t_min || t_max < root {
            root = (-1.0 * half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let position = ray.at(root);
        let uv = Self::get_sphere_uv((position - self.origin) / self.radius);
        Some(HitRecord::new(
            ray,
            position,
            (position - self.origin) / self.radius,
            root,
            uv,
            self.material.clone(),
        ))
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.origin - Vector3::new(self.radius, self.radius, self.radius),
            maximum: self.origin + Vector3::new(self.radius, self.radius, self.radius),
        })
    }
}
impl Sphere {
    fn get_sphere_uv(point: Vector3<f32>) -> Point2<f32> {
        let theta = (-1.0 * point.y).acos();
        let phi = (-1.0 * point.z).atan2(point.x) + f32::PI();
        Point2::new(phi / (2.0 * f32::PI()), theta / f32::PI())
    }
}
pub struct MovingSphere {
    pub center_0: Point3<f32>,
    pub center_1: Point3<f32>,
    pub time_0: f32,
    pub time_1: f32,
    pub radius: f32,
    pub material: Rc<RefCell<dyn Material>>,
}
impl MovingSphere {
    fn center(&self, time: f32) -> Point3<f32> {
        self.center_0
            + ((time - self.time_0) / (self.time_1 - self.time_0)) * (self.center_1 - self.center_0)
    }
}
impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let rel_origin = ray.origin - self.center(ray.time);
        let a = ray.direction.dot(ray.direction);
        let half_b = rel_origin.dot(ray.direction);
        let c = rel_origin.dot(rel_origin) - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return None;
        }
        let sqrt_d = discriminant.sqrt();
        let mut root = (-1.0 * half_b - sqrt_d) / a;
        if root < t_min || t_max < root {
            root = (-1.0 * half_b + sqrt_d) / a;
            if root < t_min || t_max < root {
                return None;
            }
        }
        let position = ray.at(root);
        let normal = (position - self.center(ray.time)) / self.radius;
        Some(HitRecord::new(
            ray,
            position,
            normal,
            root,
            Sphere::get_sphere_uv(normal),
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<Aabb> {
        Some(
            Aabb {
                minimum: self.center(time_0) - Vector3::new(self.radius, self.radius, self.radius),
                maximum: self.center(time_0) + Vector3::new(self.radius, self.radius, self.radius),
            }
            .surrounding_box(Aabb {
                minimum: self.center(time_1) - Vector3::new(self.radius, self.radius, self.radius),
                maximum: self.center(time_1) + Vector3::new(self.radius, self.radius, self.radius),
            }),
        )
    }
}
