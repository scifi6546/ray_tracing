use super::{Aabb, FlipNormals, HitRecord, Hittable, Material, XYRect, XZRect, YZRect};
use crate::prelude::*;
use crate::ray_tracer::hittable::{RayAreaInfo};
use cgmath::Point3;
use std::{cell::RefCell, rc::Rc};

pub struct RenderBox {
    box_min: Point3<f32>,
    box_max: Point3<f32>,

    xyp: XYRect,
    xym: FlipNormals,
    xzp: XZRect,
    xzm: FlipNormals,
    yzp: YZRect,
    yzm: FlipNormals,
}
impl RenderBox {
    pub fn new(
        box_min: Point3<f32>,
        box_max: Point3<f32>,
        material: Rc<RefCell<dyn Material>>,
    ) -> Self {
        let xyp = XYRect {
            material: material.clone(),
            x0: box_min.x,
            x1: box_max.x,
            y0: box_min.y,
            y1: box_max.y,
            k: box_max.z,
        };

        let xym = FlipNormals {
            item: Rc::new(XYRect {
                material: material.clone(),
                x0: box_min.x,
                x1: box_max.x,
                y0: box_min.y,
                y1: box_max.y,
                k: box_min.z,
            }),
        };
        let xzp = XZRect {
            x0: box_min.x,
            x1: box_max.x,
            z0: box_min.z,
            z1: box_max.z,
            k: box_max.y,
            material: material.clone(),
        };
        let xzm = FlipNormals {
            item: Rc::new(XZRect {
                x0: box_min.x,
                x1: box_max.x,
                z0: box_min.z,
                z1: box_max.z,
                k: box_min.y,
                material: material.clone(),
            }),
        };

        let yzp = YZRect {
            y0: box_min.y,
            y1: box_max.y,
            z0: box_min.z,
            z1: box_max.z,
            k: box_max.x,
            material: material.clone(),
        };

        let yzm = FlipNormals {
            item: Rc::new(YZRect {
                y0: box_min.y,
                y1: box_max.y,
                z0: box_min.z,
                z1: box_max.z,
                k: box_min.x,
                material: material.clone(),
            }),
        };
        Self {
            box_min,
            box_max,
            xyp,
            xym,
            xzp,
            xzm,
            yzp,
            yzm,
        }
    }
}
impl RenderBox {
    fn check_hit(closest_hit: &mut Option<HitRecord>, record: Option<HitRecord>) {
        if let Some(hit) = record {
            if closest_hit.is_some() {
                let closest_hit_ref = closest_hit.as_mut().unwrap();
                if hit.t < closest_hit_ref.t {
                    *closest_hit_ref = hit;
                }
            } else {
                *closest_hit = Some(hit);
            }
        }
    }
    fn calc_origin(&self) -> Point3<f32> {
        Point3::new(
            (self.box_min.x + self.box_max.x) / 2.0,
            (self.box_min.y + self.box_max.y) / 2.0,
            (self.box_min.z + self.box_max.z) / 2.0,
        )
    }
}
impl Hittable for RenderBox {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        Self::check_hit(&mut closest_hit, self.xyp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.xym.hit(ray, t_min, t_max));

        Self::check_hit(&mut closest_hit, self.xzp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.xzm.hit(ray, t_min, t_max));

        Self::check_hit(&mut closest_hit, self.yzp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.yzm.hit(ray, t_min, t_max));

        return closest_hit;
    }

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.box_min,
            maximum: self.box_max,
        })
    }
    fn prob(&self, ray: Ray) -> f32 {
        let mut area = 0.0;
        if ray.direction.x >= 0.0 {
            area += self.yzm.prob(ray);
        } else {
            area += self.yzp.prob(ray);
        }
        if ray.direction.y >= 0.0 {
            area += self.xzm.prob(ray)
        } else {
            area += self.xzp.prob(ray);
        }
        if ray.direction.z >= 0.0 {
            area += self.xym.prob(ray);
        } else {
            area += self.xyp.prob(ray)
        }
        area
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        let face = rand_u32(0, 3);
        let to_self = self.calc_origin() - origin;
        if face == 0 {
            if to_self.x >= 0.0 {
                self.yzm.generate_ray_in_area(origin, time)
            } else {
                self.yzp.generate_ray_in_area(origin, time)
            }
        } else if face == 1 {
            if to_self.y >= 0.0 {
                self.xzm.generate_ray_in_area(origin, time)
            } else {
                self.xzp.generate_ray_in_area(origin, time)
            }
        } else if face == 2 {
            if to_self.z >= 0.0 {
                self.xym.generate_ray_in_area(origin, time)
            } else {
                self.xyp.generate_ray_in_area(origin, time)
            }
        } else {
            panic!()
        }
    }
}
