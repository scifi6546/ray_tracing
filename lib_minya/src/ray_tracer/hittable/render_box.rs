use super::{Aabb, HitRecord, Hittable, Material, XYRect, XZRect, YZRect};
use crate::prelude::*;
use crate::ray_tracer::hittable::RayAreaInfo;
use cgmath::Point3;
use dyn_clone::clone_box;
use std::ops::Deref;

#[derive(Clone)]
pub struct RenderBox {
    box_min: Point3<RayScalar>,
    box_max: Point3<RayScalar>,

    xyp: XYRect,
    xym: XYRect,
    xzp: XZRect,
    xzm: XZRect,
    yzp: YZRect,
    yzm: YZRect,
}
impl RenderBox {
    pub fn new(
        box_min: Point3<RayScalar>,
        box_max: Point3<RayScalar>,
        material: Box<dyn Material>,
    ) -> Self {
        let xyp = XYRect::new(
            box_min.x,
            box_max.x,
            box_min.y,
            box_max.y,
            box_max.z,
            clone_box(material.deref()),
            false,
        );

        let xym = XYRect::new(
            box_min.x,
            box_max.x,
            box_min.y,
            box_max.y,
            box_min.z,
            clone_box(material.deref()),
            true,
        );
        let xzp = XZRect::new(
            box_min.x,
            box_max.x,
            box_min.z,
            box_max.z,
            box_max.y,
            clone_box(material.deref()),
            false,
        );
        let xzm = XZRect::new(
            box_min.x,
            box_max.x,
            box_min.z,
            box_max.z,
            box_min.y,
            clone_box(material.deref()),
            true,
        );

        let yzp = YZRect::new(
            box_min.y,
            box_max.y,
            box_min.z,
            box_max.z,
            box_max.x,
            clone_box(material.deref()),
            false,
        );

        let yzm = YZRect::new(
            box_min.y,
            box_max.y,
            box_min.z,
            box_max.z,
            box_min.x,
            clone_box(material.deref()),
            true,
        );
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
    fn calc_origin(&self) -> Point3<RayScalar> {
        Point3::new(
            (self.box_min.x + self.box_max.x) / 2.0,
            (self.box_min.y + self.box_max.y) / 2.0,
            (self.box_min.z + self.box_max.z) / 2.0,
        )
    }
}
impl Hittable for RenderBox {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        let mut closest_hit: Option<HitRecord> = None;
        Self::check_hit(&mut closest_hit, self.xyp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.xym.hit(ray, t_min, t_max));

        Self::check_hit(&mut closest_hit, self.xzp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.xzm.hit(ray, t_min, t_max));

        Self::check_hit(&mut closest_hit, self.yzp.hit(ray, t_min, t_max));
        Self::check_hit(&mut closest_hit, self.yzm.hit(ray, t_min, t_max));

        return closest_hit;
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.box_min,
            maximum: self.box_max,
        })
    }
    fn prob(&self, ray: Ray) -> RayScalar {
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

    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
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
