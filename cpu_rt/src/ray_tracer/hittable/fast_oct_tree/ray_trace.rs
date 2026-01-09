use super::{hit_info::HitInfo, leafable::Leafable, voxel::Voxel, FastOctTree};
use crate::prelude::{Ray, RayScalar};
use cgmath::{Point3, Vector3};
impl FastOctTree<Voxel> {
    fn ray_iteration(
        &self,
        block_coordinates: Point3<i32>,
        ray: Ray,
        initial_normal: Vector3<RayScalar>,
    ) -> Option<HitInfo<Voxel>> {
        todo!("ray iteration")
    }
    pub fn trace_ray(&self, ray: Ray) -> Option<HitInfo<Voxel>> {
        struct PlaneIntersection {
            normal_axis: usize,
            intersect_time: RayScalar,
            normal_vector: Vector3<RayScalar>,
            intersect_position: Point3<RayScalar>,
            block_coordinate: Point3<i32>,
        }

        if ray.origin.x >= 0.0
            && ray.origin.x <= self.world_size() as RayScalar
            && ray.origin.y >= 0.0
            && ray.origin.y <= self.world_size() as RayScalar
            && ray.origin.z >= 0.0
            && ray.origin.z <= self.world_size() as RayScalar
        {
            self.ray_iteration(
                Point3::new(
                    ray.origin.x as i32,
                    ray.origin.y as i32,
                    ray.origin.z as i32,
                ),
                Ray {
                    origin: ray.origin,
                    direction: ray.direction,
                    time: ray.time,
                },
                Vector3::unit_x(),
            )
        } else {
            let mut solutions = (0..3)
                .flat_map(|normal_axis| {
                    let zero_intersect_time = ray.intersect_axis(normal_axis, 0.);
                    let zero_intersect_position = ray.at(zero_intersect_time);
                    let size_intersect_time =
                        ray.intersect_axis(normal_axis, self.world_size() as RayScalar);
                    let size_intersect_position = ray.at(size_intersect_time);
                    [
                        PlaneIntersection {
                            normal_axis,
                            intersect_time: zero_intersect_time,
                            normal_vector: Vector3::new(
                                if normal_axis == 0 { -1.0 } else { 0.0 },
                                if normal_axis == 1 { -1.0 } else { 0.0 },
                                if normal_axis == 2 { -1.0 } else { 0.0 },
                            ),
                            intersect_position: zero_intersect_position,
                            block_coordinate: Point3::new(
                                zero_intersect_position.x as i32,
                                zero_intersect_position.y as i32,
                                zero_intersect_position.z as i32,
                            ),
                        },
                        PlaneIntersection {
                            normal_axis,
                            intersect_time: size_intersect_time,
                            normal_vector: Vector3::new(
                                if normal_axis == 0 { 1.0 } else { 0.0 },
                                if normal_axis == 1 { 1.0 } else { 0.0 },
                                if normal_axis == 2 { 1.0 } else { 0.0 },
                            ),
                            intersect_position: size_intersect_position,
                            block_coordinate: Point3::new(
                                if normal_axis == 0 {
                                    size_intersect_position.x as i32 - 1
                                } else {
                                    size_intersect_position.x as i32
                                },
                                if normal_axis == 1 {
                                    size_intersect_position.y as i32 - 1
                                } else {
                                    size_intersect_position.y as i32
                                },
                                if normal_axis == 2 {
                                    size_intersect_position.z as i32 - 1
                                } else {
                                    size_intersect_position.z as i32
                                },
                            ),
                        },
                    ]
                })
                .filter(|intersection| {
                    ((intersection.intersect_position[0] >= 0.
                        && intersection.intersect_position[0] < self.world_size() as RayScalar)
                        || intersection.normal_axis == 0)
                        && ((intersection.intersect_position[1] >= 0.
                            && intersection.intersect_position[1] < self.world_size() as RayScalar)
                            || intersection.normal_axis == 1)
                        && ((intersection.intersect_position[2] >= 0.
                            && intersection.intersect_position[2] < self.world_size() as RayScalar)
                            || intersection.normal_axis == 2)
                })
                .collect::<Vec<_>>();
            solutions.sort_by(|a, b| a.intersect_time.partial_cmp(&b.intersect_time).unwrap());
            if let Some(intersection) = solutions.first() {
                let block = self
                    .get(intersection.block_coordinate.map(|v| v as u32).map(|v| v))
                    .unwrap();
                match block {
                    Voxel::Solid(solid_material) => Some(HitInfo::Solid {
                        hit_position: intersection.intersect_position,
                        hit_value: solid_material.to_material(),
                        normal: intersection.normal_vector,
                    }),
                    Voxel::Volume(_) => self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                        intersection.normal_vector,
                    ),
                    Voxel::Empty => self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                        intersection.normal_vector,
                    ),
                }
            } else {
                None
            }
        }
    }
}
