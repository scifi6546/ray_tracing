use super::{Material, Ray, AABB};

//use cgmath::num_traits::FloatConst;
use cgmath::{num_traits::FloatConst, prelude::*, InnerSpace, Point2, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub trait Hittable {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord>;
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB>;
}
#[derive(Clone)]
pub struct HitRecord {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub t: f32,
    pub front_face: bool,
    pub uv: Point2<f32>,
    pub(crate) material: Rc<RefCell<dyn Material>>,
}

impl HitRecord {
    pub fn new(
        ray: &Ray,
        position: Point3<f32>,
        normal: Vector3<f32>,
        t: f32,
        uv: Point2<f32>,
        material: Rc<RefCell<dyn Material>>,
    ) -> Self {
        let front_face = ray.direction.dot(normal) < 0.0;
        Self {
            position,
            normal: if front_face { normal } else { -1.0 * normal },
            t,
            front_face,
            uv,
            material,
        }
    }
}

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
        let uv = Self::get_sphere_uv(position);
        Some(HitRecord::new(
            ray,
            position,
            (position - self.origin) / self.radius,
            root,
            uv,
            self.material.clone(),
        ))
    }
    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(AABB {
            minimum: self.origin - Vector3::new(self.radius, self.radius, self.radius),
            maximum: self.origin + Vector3::new(self.radius, self.radius, self.radius),
        })
    }
}
impl Sphere {
    fn get_sphere_uv(point: Point3<f32>) -> Point2<f32> {
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
        Some(HitRecord::new(
            ray,
            position,
            (position - self.center(ray.time)) / self.radius,
            root,
            todo!("uv"),
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(
            AABB {
                minimum: self.center(time_0) - Vector3::new(self.radius, self.radius, self.radius),
                maximum: self.center(time_0) + Vector3::new(self.radius, self.radius, self.radius),
            }
            .surrounding_box(AABB {
                minimum: self.center(time_1) - Vector3::new(self.radius, self.radius, self.radius),
                maximum: self.center(time_1) + Vector3::new(self.radius, self.radius, self.radius),
            }),
        )
    }
}
