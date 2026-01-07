use super::{
    super::{Aabb, HitRecord, Hittable, RayAreaInfo},
    Tree,
};
use crate::prelude::{Ray, RayScalar};
use cgmath::Point3;
impl Hittable for Tree {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        todo!("hit")
    }
    fn bounding_box(&self, time_0: RayScalar, time_1: RayScalar) -> Option<Aabb> {
        todo!("bounding box")
    }
    fn prob(&self, ray: Ray) -> RayScalar {
        todo!("prob")
    }
    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        todo!("generate in area")
    }
}
