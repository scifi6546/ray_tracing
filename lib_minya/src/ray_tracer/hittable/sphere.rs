use super::{Aabb, HitRecord, Hittable, Material};

use crate::{
    prelude::{Ray, RayScalar},
    ray_tracer::{hittable::RayAreaInfo, rand_unit_vec},
};

use cgmath::{num_traits::FloatConst, prelude::*, Point2, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;

pub struct Sphere {
    pub radius: RayScalar,
    pub origin: Point3<RayScalar>,
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
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
            self.material.as_ref(),
        ))
    }
    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.origin - Vector3::new(self.radius, self.radius, self.radius),
            maximum: self.origin + Vector3::new(self.radius, self.radius, self.radius),
        })
    }
    fn prob(&self, ray: Ray) -> RayScalar {
        let area = RayScalar::PI() * self.radius.powi(2);
        let to_light = self.origin - ray.origin;

        let distance_squared = to_light.dot(to_light);

        distance_squared / area
    }

    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
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
    fn area(&self) -> RayScalar {
        RayScalar::PI() * self.radius.powi(2)
    }
    fn get_sphere_uv(point: Vector3<RayScalar>) -> Point2<RayScalar> {
        let theta = (-1.0 * point.y).acos();
        let phi = (-1.0 * point.z).atan2(point.x) + RayScalar::PI();
        Point2::new(phi / (2.0 * RayScalar::PI()), theta / RayScalar::PI())
    }
}

pub struct MovingSphere {
    pub center_0: Point3<RayScalar>,
    pub center_1: Point3<RayScalar>,
    pub time_0: RayScalar,
    pub time_1: RayScalar,
    pub radius: RayScalar,
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
    fn center(&self, time: RayScalar) -> Point3<RayScalar> {
        self.center_0
            + ((time - self.time_0) / (self.time_1 - self.time_0)) * (self.center_1 - self.center_0)
    }
}
impl Hittable for MovingSphere {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self, time_0: RayScalar, time_1: RayScalar) -> Option<Aabb> {
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
    fn prob(&self, _ray: Ray) -> RayScalar {
        todo!()
    }
    fn generate_ray_in_area(&self, _origin: Point3<RayScalar>, _time: RayScalar) -> RayAreaInfo {
        todo!()
    }
}
