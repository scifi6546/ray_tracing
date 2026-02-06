use super::{
    super::{Aabb, HitRecord, Hittable, RayAreaInfo},
    hit_info::HitInfo,
    FastOctTree, Voxel,
};

use crate::prelude::{Ray, RayScalar};
use cgmath::{prelude::*, Point2, Point3, Vector3};
impl Hittable for FastOctTree<Voxel> {
    fn hit(&self, ray: &Ray, t_min: RayScalar, t_max: RayScalar) -> Option<HitRecord> {
        let aabb = self.bounding_box(t_min, t_max).unwrap();
        if aabb.hit(*ray, t_min, t_max) {
            if let Some(hit_info) = self.trace_ray(Ray {
                origin: ray.origin,
                time: ray.time,
                direction: ray.direction.normalize(),
            }) {
                match hit_info {
                    HitInfo::Solid {
                        hit_value,
                        hit_position,
                        normal,
                    } => {
                        let t = ((hit_position - ray.origin).magnitude2()
                            / ray.direction.magnitude2())
                        .sqrt();

                        if t > t_min && t < t_max {
                            Some(HitRecord::new_template(
                                ray,
                                hit_position,
                                normal,
                                t,
                                Point2::new(0.5, 0.5),
                                hit_value,
                            ))
                        } else {
                            None
                        }
                    }
                    HitInfo::Volume {
                        hit_value,
                        hit_position,
                    } => {
                        let t = ((hit_position - ray.origin).magnitude2()
                            / ray.direction.magnitude2())
                        .sqrt();
                        if t > t_min && t < t_max {
                            Some(HitRecord::new_template(
                                ray,
                                hit_position,
                                Vector3::unit_x(),
                                t,
                                Point2::origin(),
                                hit_value,
                            ))
                        } else {
                            None
                        }
                    }
                }
            } else {
                None
            }
        } else {
            None
        }
    }
    fn bounding_box(&self, _time_0: RayScalar, _time_1: RayScalar) -> Option<Aabb> {
        let size = self.world_size();
        if size > 0 {
            Some(Aabb {
                minimum: Point3::new(0., 0., 0.),
                maximum: Point3::new(size as RayScalar, size as RayScalar, size as RayScalar),
            })
        } else {
            None
        }
    }
    fn prob(&self, ray: Ray) -> RayScalar {
        todo!("prob")
    }
    fn generate_ray_in_area(&self, origin: Point3<RayScalar>, time: RayScalar) -> RayAreaInfo {
        todo!("generate in area")
    }
}
#[cfg(test)]
mod test {
    use super::super::SolidVoxel;
    use crate::prelude::RgbColor;

    use super::*;
    #[test]
    fn aabb_empty() {
        let empty = FastOctTree::new();
        assert!(empty.bounding_box(0., 1.).is_none())
    }
    #[test]
    fn aabb_size_1() {
        let mut tree = FastOctTree::new();
        tree.set(
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::WHITE,
            }),
            Point3::new(0, 0, 0),
        );
        assert!(tree.bounding_box(0., 1.).unwrap().approx_eq(Aabb {
            maximum: Point3::new(1., 1., 1.),
            minimum: Point3::new(0., 0., 0.)
        }));
    }
}
