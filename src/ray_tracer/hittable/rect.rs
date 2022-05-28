use super::{Aabb, HitRecord, Hittable, Light, Material, RayAreaInfo};
use crate::prelude::*;
use cgmath::{prelude::*, Point2, Point3, Vector3};
use std::{cell::RefCell, rc::Rc};

pub struct XYRect {
    pub material: Rc<RefCell<dyn Material>>,
    pub x0: f32,
    pub x1: f32,
    pub y0: f32,
    pub y1: f32,
    pub k: f32,
}
impl XYRect {
    pub const NORMAL: Vector3<f32> = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    fn area(&self) -> f32 {
        (self.x1 - self.x0) * (self.y1 - self.y0)
    }
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
            Self::NORMAL,
            t,
            uv,
            self.material.clone(),
        ))
    }

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.x0, self.y0, self.k - 0.001),
            maximum: Point3::new(self.x1, self.y1, self.k + 0.001),
        })
    }
}
impl Light for XYRect {
    fn prob(&self, ray: Ray) -> f32 {
        let center =
            0.5 * (Point3::new(self.x0, self.y0, self.k) + Vector3::new(self.x1, self.y1, self.k));
        let to_light = center - ray.origin;
        let cos_alpha = to_light.normalize().z.abs();
        if cos_alpha < 0.00001 {
            return 0.0;
        }
        let area = (self.x1 - self.x0) * (self.y1 - self.y0);

        let distance_squared = to_light.dot(to_light);

        distance_squared / (cos_alpha * self.area())
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let end_point = Point3::new(
            rand_f32(self.x0, self.x1),
            rand_f32(self.y0, self.y1),
            self.k,
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: Self::NORMAL,
            area: self.area(),
            direction: end_point - origin,
        }
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
impl XZRect {
    pub const NORMAL: Vector3<f32> = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
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

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.x0, self.k - 0.001, self.z0),
            maximum: Point3::new(self.x1, self.k + 0.001, self.z1),
        })
    }
}
impl Light for XZRect {
    fn prob(&self, ray: Ray) -> f32 {
        let center =
            0.5 * (Point3::new(self.x0, self.k, self.z0) + Vector3::new(self.x1, self.k, self.z1));
        let to_light = center - ray.origin;
        let cos_alpha = to_light.normalize().y.abs();
        if cos_alpha < 0.00001 {
            return 0.0;
        }
        let area = (self.x1 - self.x0) * (self.z1 - self.z0);

        let distance_squared = to_light.dot(to_light);

        distance_squared / (cos_alpha * area)
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let end_point = Point3::new(
            rand_f32(self.x0, self.x1),
            self.k,
            rand_f32(self.z0, self.z1),
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: Self::NORMAL,
            area: (self.x1 - self.x0) * (self.z1 - self.z0),
            direction: end_point - origin,
        }
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
impl YZRect {
    pub const NORMAL: Vector3<f32> = Vector3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    fn area(&self) -> f32 {
        (self.y1 - self.y0) * (self.z1 - self.z0)
    }
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

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.k - 0.001, self.y0, self.z0),
            maximum: Point3::new(self.k + 0.001, self.y1, self.z1),
        })
    }
}
impl Light for YZRect {
    fn prob(&self, ray: Ray) -> f32 {
        let center =
            0.5 * (Point3::new(self.k, self.y0, self.z0) + Vector3::new(self.k, self.y1, self.z1));
        let to_light = center - ray.origin;
        let cos_alpha = to_light.normalize().x.abs();
        if cos_alpha < 0.00001 {
            return 0.0;
        }

        let distance_squared = to_light.dot(to_light);

        distance_squared / (cos_alpha * self.area())
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let end_point = Point3::new(
            self.k,
            rand_f32(self.y0, self.y1),
            rand_f32(self.z0, self.z1),
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: Self::NORMAL,
            area: self.area(),
            direction: end_point - origin,
        }
    }
}
