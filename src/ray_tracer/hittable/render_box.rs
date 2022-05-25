use super::{Aabb, HitRecord, Hittable, Material, XYRect, XZRect, YZRect};
use crate::prelude::*;
use cgmath::Point3;
use std::{cell::RefCell, rc::Rc};
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

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: self.box_min,
            maximum: self.box_max,
        })
    }
}
