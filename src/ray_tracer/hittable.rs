mod constant_medium;
mod flip_normals;
mod rotation;

use super::{Material, Ray, AABB};

use crate::prelude::{p_max, p_min, rand_f32};
use cgmath::{num_traits::FloatConst, prelude::*, InnerSpace, Point2, Point3, Vector2, Vector3};

pub use constant_medium::ConstantMedium;
pub use flip_normals::FlipNormals;
pub use rotation::RotateY;
use std::ops::RemAssign;
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
    pub material: Rc<RefCell<dyn Material>>,
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
        let uv = Self::get_sphere_uv(((position - self.origin) / self.radius));
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
pub struct XYRect {
    pub material: Rc<RefCell<dyn Material>>,
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub k: f32,
}
impl Hittable for XYRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.z) / ray.direction.z;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let y = ray.origin.y + t * ray.direction.y;
        if x < self.x0 || x > self.x1 || y < self.y0 || y > self.y1 {
            return None;
        }
        let uv = Point2::new(
            (x - self.x0) / (self.x1 - self.x0),
            (y - self.y0) / (self.y1 - self.y0),
        );

        Some(HitRecord::new(
            ray,
            ray.at(t),
            Vector3::new(0.0, 0.0, 1.0),
            t,
            uv,
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(AABB {
            minimum: Point3::new(self.x0, self.y0, self.k - 0.001),
            maximum: Point3::new(self.x1, self.y1, self.k + 0.001),
        })
    }
}
pub struct XZRect {
    pub x0: f32,
    pub x1: f32,
    pub z0: f32,
    pub z1: f32,
    pub k: f32,
    pub material: Rc<RefCell<dyn Material>>,
}
impl Hittable for XZRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.y) / ray.direction.y;
        if t < t_min || t > t_max {
            return None;
        }
        let x = ray.origin.x + t * ray.direction.x;
        let z = ray.origin.z + t * ray.direction.z;
        if x < self.x0 || x > self.x1 || z < self.z0 || z > self.z1 {
            return None;
        }
        Some(HitRecord::new(
            ray,
            ray.at(t),
            Vector3::new(0.0, 1.0, 0.0),
            t,
            Point2::new(
                (x - self.x0) / (self.x1 - self.x0),
                (z - self.z0) / (self.z1 - self.z0),
            ),
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(AABB {
            minimum: Point3::new(self.x0, self.k - 0.001, self.z0),
            maximum: Point3::new(self.x1, self.k + 0.001, self.z1),
        })
    }
}
pub struct YZRect {
    pub y0: f32,
    pub y1: f32,
    pub z0: f32,
    pub z1: f32,
    pub k: f32,
    pub material: Rc<RefCell<dyn Material>>,
}
impl Hittable for YZRect {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let t = (self.k - ray.origin.x) / ray.direction.x;
        if t < t_min || t > t_max {
            return None;
        }
        let y = ray.origin.y + t * ray.direction.y;
        let z = ray.origin.z + t * ray.direction.z;
        if y < self.y0 || y > self.y1 || z < self.z0 || z > self.z1 {
            return None;
        }

        Some(HitRecord::new(
            ray,
            ray.at(t),
            Vector3::new(1.0, 0.0, 0.0),
            t,
            Point2::new(
                (y - self.y0) / (self.y1 - self.y0),
                (z - self.z0) / (self.z1 - self.z0),
            ),
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(AABB {
            minimum: Point3::new(self.k - 0.001, self.y0, self.z0),
            maximum: Point3::new(self.k + 0.001, self.y1, self.z1),
        })
    }
}
pub struct RenderBox {
    box_min: Point3<f32>,
    box_max: Point3<f32>,
    sides: Vec<Box<dyn Hittable>>,
}
impl RenderBox {
    pub fn new(
        box_min: Point3<f32>,
        box_max: Point3<f32>,
        material: Rc<RefCell<dyn Material>>,
    ) -> Self {
        Self {
            box_min,
            box_max,
            sides: vec![
                Box::new(XYRect {
                    material: material.clone(),
                    x0: box_min.x,
                    x1: box_max.x,
                    y0: box_min.y,
                    y1: box_max.y,
                    k: box_max.z,
                }),
                Box::new(XYRect {
                    material: material.clone(),
                    x0: box_min.x,
                    x1: box_max.x,
                    y0: box_min.y,
                    y1: box_max.y,
                    k: box_min.z,
                }),
                Box::new(XZRect {
                    x0: box_min.x,
                    x1: box_max.x,
                    z0: box_min.z,
                    z1: box_max.z,
                    k: box_max.y,
                    material: material.clone(),
                }),
                Box::new(XZRect {
                    x0: box_min.x,
                    x1: box_max.x,
                    z0: box_min.z,
                    z1: box_max.z,
                    k: box_min.y,
                    material: material.clone(),
                }),
                Box::new(YZRect {
                    y0: box_min.y,
                    y1: box_max.y,
                    z0: box_min.z,
                    z1: box_max.z,
                    k: box_max.x,
                    material: material.clone(),
                }),
                Box::new(YZRect {
                    y0: box_min.y,
                    y1: box_max.y,
                    z0: box_min.z,
                    z1: box_max.z,
                    k: box_min.x,
                    material: material.clone(),
                }),
            ],
        }
    }
}
impl Hittable for RenderBox {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        self.sides
            .iter()
            .filter_map(|s| s.hit(ray, t_min, t_max))
            .reduce(|acc, x| if acc.t < x.t { acc } else { x })
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        Some(AABB {
            minimum: self.box_min,
            maximum: self.box_max,
        })
    }
}
pub struct Translate {
    pub item: Rc<dyn Hittable>,
    pub offset: Vector3<f32>,
}
impl Hittable for Translate {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let moved_ray = Ray {
            origin: ray.origin - self.offset,
            direction: ray.direction,
            time: ray.time,
        };
        if let Some(mut record) = self.item.hit(&moved_ray, t_min, t_max) {
            Some(HitRecord::new(
                &moved_ray,
                record.position + self.offset,
                record.normal,
                record.t,
                record.uv,
                record.material.clone(),
            ))
        } else {
            None
        }
    }

    fn bounding_box(&self, time_0: f32, time_1: f32) -> Option<AABB> {
        if let Some(b) = self.item.bounding_box(time_0, time_1) {
            Some(AABB {
                minimum: b.minimum + self.offset,
                maximum: b.maximum + self.offset,
            })
        } else {
            None
        }
    }
}
