use super::{Aabb, HitRecord, Hittable, Material};
use crate::prelude::*;
use crate::ray_tracer::hittable::RayAreaInfo;
use crate::ray_tracer::rand_unit_vec;
use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;
use std::{cell::RefCell, rc::Rc};

pub struct Sphere {
    pub radius: f32,
    pub origin: Point3<f32>,
    pub material: Box<dyn Material>,
}
impl Clone for Sphere {
    fn clone(&self) -> Self {
        Self {
            radius: self.radius,
            origin: self.origin,
            material: clone_box(self.material.deref()),
        }
    }
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
            clone_box(self.material.deref()),
        ))
    }
    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.origin - Vector3::new(self.radius, self.radius, self.radius),
            maximum: self.origin + Vector3::new(self.radius, self.radius, self.radius),
        })
    }
    fn prob(&self, ray: Ray) -> f32 {
        let area = f32::PI() * self.radius.powi(2);
        let to_light = self.origin - ray.origin;

        let distance_squared = to_light.dot(to_light);

        distance_squared / area
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let sphere_direction = rand_unit_vec();
        let end_point = self.origin + self.radius * sphere_direction;
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: sphere_direction,
            area: self.area(),
            direction: end_point - origin,
            end_point,
        }
    }
}

impl Sphere {
    fn area(&self) -> f32 {
        f32::PI() * self.radius.powi(2)
    }
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
    pub material: Box<dyn Material>,
}
impl Clone for MovingSphere {
    fn clone(&self) -> Self {
        Self {
            center_0: self.center_0,
            center_1: self.center_1,
            time_0: self.time_0,
            time_1: self.time_1,
            radius: self.radius,
            material: clone_box(self.material.deref()),
        }
    }
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
            clone_box(self.material.deref()),
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
    fn prob(&self, _ray: Ray) -> f32 {
        todo!()
    }
    fn generate_ray_in_area(&self, _origin: Point3<f32>, _time: f32) -> RayAreaInfo {
        todo!()
    }
}
