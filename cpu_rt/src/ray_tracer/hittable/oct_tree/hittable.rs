use super::OctTreeHitInfo;
use crate::prelude::{Ray, RayScalar};
use crate::ray_tracer::{
    bvh::Aabb,
    hittable::{HitRecord, Hittable, OctTree, RayAreaInfo, VoxelMaterial},
};
use cgmath::{prelude::*, Point2, Point3, Vector3};

impl Hittable for OctTree<VoxelMaterial> {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        let aabb = self.bounding_box(0., 1.).unwrap();
        if aabb.hit(*ray, t_min, t_max) {
            if let Some(hit_info) = self.trace_ray(Ray {
                origin: ray.origin,
                time: ray.time,
                direction: ray.direction.normalize(),
            }) {
                match hit_info {
                    OctTreeHitInfo::Solid {
                        hit_value,
                        depth,
                        hit_position,
                        normal,
                    } => Some(HitRecord::new(
                        ray,
                        hit_position,
                        normal,
                        depth,
                        Point2::new(0.5, 0.5),
                        hit_value,
                    )),
                    OctTreeHitInfo::Volume {
                        hit_value,
                        depth,
                        hit_position,
                    } => Some(HitRecord::new(
                        ray,
                        hit_position,
                        Vector3::unit_x(),
                        depth,
                        Point2::origin(),
                        &hit_value,
                    )),
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(0.0, 0.0, 0.0),
            maximum: Point3::new(
                self.size as RayScalar,
                self.size as RayScalar,
                self.size as RayScalar,
            ),
        })
    }

    fn prob(&self, _ray: Ray) -> RayScalar {
        todo!()
    }

    fn generate_ray_in_area(&self, _origin: Point3<RayScalar>, _time: RayScalar) -> RayAreaInfo {
        todo!()
    }
    fn name(&self) -> String {
        "Oct Tree".to_string()
    }
}
