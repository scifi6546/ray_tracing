use super::{Aabb, HitRecord, Hittable, Material, RayAreaInfo};
use crate::prelude::*;
use cgmath::{prelude::*, Point2, Point3, Vector3};
use dyn_clone::clone_box;
use std::ops::Deref;

pub struct XYRect {
    pub material: Box<dyn Material>,
    pub x0: RayScalar,
    pub x1: RayScalar,
    pub y0: RayScalar,
    pub y1: RayScalar,
    pub k: RayScalar,
    normal_flip: RayScalar,
}

impl Clone for XYRect {
    fn clone(&self) -> Self {
        Self {
            material: clone_box(self.material.deref()),
            x0: self.x0,
            x1: self.x1,
            y0: self.y0,
            y1: self.y1,
            k: self.k,
            normal_flip: self.normal_flip,
        }
    }
}
impl XYRect {
    pub const NORMAL: Vector3<RayScalar> = Vector3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub fn new(
        x0: RayScalar,
        x1: RayScalar,
        y0: RayScalar,
        y1: RayScalar,
        z: RayScalar,
        material: Box<dyn Material>,
        flip_normals: bool,
    ) -> Self {
        Self {
            x0,
            x1,
            y0,
            y1,
            k: z,
            material,
            normal_flip: match flip_normals {
                true => -1.0,
                false => 1.0,
            },
        }
    }
    fn area(&self) -> RayScalar {
        (self.x1 - self.x0) * (self.y1 - self.y0)
    }
}
impl Hittable for XYRect {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
            self.normal_flip * Self::NORMAL,
            t,
            uv,
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.x0, self.y0, self.k - 0.001),
            maximum: Point3::new(self.x1, self.y1, self.k + 0.001),
        })
    }
    fn prob(&self, ray: Ray) -> RayScalar {
        let center =
            0.5 * (Point3::new(self.x0, self.y0, self.k) + Vector3::new(self.x1, self.y1, self.k));
        let to_light = center - ray.origin;
        let cos_alpha = to_light.normalize().z.abs();
        if cos_alpha < 0.00001 {
            return 0.0;
        }

        let distance_squared = to_light.dot(to_light);

        distance_squared / (cos_alpha * self.area())
    }

    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        let end_point = Point3::new(
            rand_scalar(self.x0, self.x1),
            rand_scalar(self.y0, self.y1),
            self.k,
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: self.normal_flip * Self::NORMAL,
            area: self.area(),
            direction: end_point - origin,
            end_point,
        }
    }
    fn name(&self) -> String {
        "XY Rectangle".to_string()
    }
}

pub struct XZRect {
    pub x0: RayScalar,
    pub x1: RayScalar,
    pub z0: RayScalar,
    pub z1: RayScalar,
    pub k: RayScalar,
    normal_flip: RayScalar,
    pub material: Box<dyn Material>,
}

impl Clone for XZRect {
    fn clone(&self) -> Self {
        Self {
            x0: self.x0,
            x1: self.x1,
            z0: self.z0,
            z1: self.z1,
            k: self.k,
            normal_flip: self.normal_flip,
            material: clone_box(self.material.deref()),
        }
    }
}
impl XZRect {
    pub const NORMAL: Vector3<RayScalar> = Vector3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub fn new(
        x0: RayScalar,
        x1: RayScalar,
        z0: RayScalar,
        z1: RayScalar,
        y: RayScalar,
        material: Box<dyn Material>,
        flip_normals: bool,
    ) -> Self {
        Self {
            x0,
            x1,
            z0,
            z1,
            k: y,
            normal_flip: match flip_normals {
                true => -1.0,
                false => 1.0,
            },
            material,
        }
    }
}
impl Hittable for XZRect {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
            self.normal_flip * Self::NORMAL,
            t,
            Point2::new(
                (x - self.x0) / (self.x1 - self.x0),
                (z - self.z0) / (self.z1 - self.z0),
            ),
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.x0, self.k - 0.001, self.z0),
            maximum: Point3::new(self.x1, self.k + 0.001, self.z1),
        })
    }
    fn prob(&self, ray: Ray) -> RayScalar {
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

    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        let end_point = Point3::new(
            rand_scalar(self.x0, self.x1),
            self.k,
            rand_scalar(self.z0, self.z1),
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: self.normal_flip * Self::NORMAL,
            area: (self.x1 - self.x0) * (self.z1 - self.z0),
            direction: end_point - origin,
            end_point,
        }
    }
    fn name(&self) -> String {
        "XZ Rectangle".to_string()
    }
}

pub struct YZRect {
    pub y0: RayScalar,
    pub y1: RayScalar,
    pub z0: RayScalar,
    pub z1: RayScalar,
    pub k: RayScalar,
    pub material: Box<dyn Material>,
    normal_flip: RayScalar,
}

impl Clone for YZRect {
    fn clone(&self) -> Self {
        Self {
            y0: self.y0,
            y1: self.y1,
            z0: self.z0,
            z1: self.z1,
            k: self.k,
            material: clone_box(self.material.deref()),
            normal_flip: self.normal_flip,
        }
    }
}
impl YZRect {
    pub const NORMAL: Vector3<RayScalar> = Vector3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub fn new(
        y0: RayScalar,
        y1: RayScalar,
        z0: RayScalar,
        z1: RayScalar,
        x: RayScalar,
        material: Box<dyn Material>,
        flip_normals: bool,
    ) -> Self {
        Self {
            y0,
            y1,
            z0,
            z1,
            k: x,
            material,
            normal_flip: match flip_normals {
                true => -1.0,
                false => 1.0,
            },
        }
    }
    fn area(&self) -> RayScalar {
        (self.y1 - self.y0) * (self.z1 - self.z0)
    }
}
impl Hittable for YZRect {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
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
            self.normal_flip * Self::NORMAL,
            t,
            Point2::new(
                (y - self.y0) / (self.y1 - self.y0),
                (z - self.z0) / (self.z1 - self.z0),
            ),
            self.material.as_ref(),
        ))
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(self.k - 0.001, self.y0, self.z0),
            maximum: Point3::new(self.k + 0.001, self.y1, self.z1),
        })
    }
    fn prob(&self, ray: Ray) -> RayScalar {
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

    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        let end_point = Point3::new(
            self.k,
            rand_scalar(self.y0, self.y1),
            rand_scalar(self.z0, self.z1),
        );
        let direction = (end_point - origin).normalize();
        RayAreaInfo {
            to_area: Ray {
                origin,
                direction,
                time,
            },
            normal: self.normal_flip * Self::NORMAL,
            area: self.area(),
            direction: end_point - origin,
            end_point,
        }
    }
    fn name(&self) -> String {
        "YZ Rectangle".to_string()
    }
}
