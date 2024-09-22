use super::{Leafable, OctTree, OctTreeHitInfo, OctTreeNode};
use crate::prelude::{Ray, RayScalar};
use log::info;
use std::{
    cmp::{max, min},
    ops::Neg,
};

use cgmath::{prelude::*, Point3, Vector3};

fn min_idx_vec(v: Vector3<RayScalar>) -> usize {
    let mut min_val = v.x;
    let mut min_idx = 0;

    if min_val > v.y {
        min_val = v.y;
        min_idx = 1;
    }
    if min_val > v.z {
        return 2;
    }
    return min_idx;
}
impl<T: Leafable> OctTree<T> {
    pub fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        self.root_node.trace_ray(ray)
    }
}
impl<T: Leafable> OctTreeNode<T> {
    fn in_range(&self, position: Point3<i32>) -> bool {
        let is_good = position.map(|v| v >= 0 && v < self.size as i32);
        is_good[0] && is_good[1] && is_good[2]
    }
    /// takes in ray with pos at border or inside of self
    fn trace_v2(&self, ray: Ray, start_pos: Option<Point3<u32>>) -> Option<OctTreeHitInfo<T>> {
        /// gets the distance between pos and the next voxel for every axis
        fn get_distance(
            position: Point3<RayScalar>,
            direction: Vector3<RayScalar>,
            chunk_size: u32,
        ) -> Vector3<RayScalar> {
            let mut out_vector = Vector3::<RayScalar>::zero();

            //x
            if direction.x.is_sign_positive() {
                let x_remainder = position.x.rem_euclid(chunk_size as RayScalar);

                out_vector.x = (chunk_size as RayScalar - x_remainder) / direction.x.abs();
            } else {
                let x_remainder = position.x.rem_euclid(chunk_size as RayScalar);
                out_vector.x = x_remainder / direction.x.abs();
            };

            //y
            if direction.y.is_sign_positive() {
                let y_remainder = position.y.rem_euclid(chunk_size as RayScalar);
                out_vector.y = (chunk_size as RayScalar - y_remainder) / direction.y.abs();
            } else {
                let y_remainder = position.y.rem_euclid(chunk_size as RayScalar);

                out_vector.y = y_remainder / direction.y.abs()
            };

            //y
            if direction.z.is_sign_positive() {
                let z_remainder = position.z.rem_euclid(chunk_size as RayScalar);
                out_vector.z = (chunk_size as RayScalar - z_remainder) / direction.z.abs();
            } else {
                let z_remainder = position.z.rem_euclid(chunk_size as RayScalar);

                out_vector.z = z_remainder / direction.z.abs();
            };
            out_vector
        }
        /// gets thhe index of the minumum axis
        fn get_minimum_axis(vector: Vector3<RayScalar>) -> usize {
            if vector.x.is_sign_negative()
                || vector.y.is_sign_negative()
                || vector.z.is_sign_negative()
            {
                info!(
                    "axis negetive, value: [{},{},{}]",
                    vector.x, vector.y, vector.z
                )
            }
            let mut out_idx = 0;
            if vector.y < vector.x {
                out_idx = 1;
            }
            if vector.z < vector.y {
                out_idx = 2;
            }
            out_idx
        }
        let mut voxel_pos = if let Some(p) = start_pos {
            p
        } else {
            ray.origin.map(|p| p as u32)
        };

        let mut current_position = ray.origin;
        let direction = ray.direction.normalize();
        loop {
            let chunk = self.get_homogenous_chunk(voxel_pos);
            if chunk.is_none() {
                return None;
            }
            let leaf = chunk.unwrap().leaf_value().unwrap();
            if let Some(hit_value) = leaf.try_solid() {
                return Some(OctTreeHitInfo {
                    hit_value,
                    hit_position: current_position,
                    depth: (current_position).distance(ray.origin),
                    normal: -Vector3::unit_z(),
                });
            }
            let chunk_size = chunk.unwrap().size;
            let distances = get_distance(current_position, direction, chunk_size);
            let minimum_axis = get_minimum_axis(distances);
            if minimum_axis == 0 {
                if ray.direction.x.is_sign_positive() {
                    voxel_pos.x += chunk_size;
                } else {
                    voxel_pos.x -= chunk_size;
                }

                current_position += distances.x * direction;
            } else if minimum_axis == 1 {
                if ray.direction.y.is_sign_positive() {
                    voxel_pos.y += chunk_size;
                } else {
                    voxel_pos.y -= chunk_size;
                }

                current_position += distances.y * direction;
            } else if minimum_axis == 2 {
                if ray.direction.z.is_sign_positive() {
                    voxel_pos.z += chunk_size;
                } else {
                    voxel_pos.z -= chunk_size;
                }

                current_position += distances.z * direction;
            } else {
                panic!("invalid axis")
            }
        }
    }
    fn trace(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        let mut voxel_pos = ray.origin.map(|e| e as i32);
        fn calculate_size(chunk_size: i32, direction: Vector3<RayScalar>) -> Vector3<RayScalar> {
            chunk_size as RayScalar / direction.map(|e| e.abs())
        }

        let move_size = 1;
        let mut current_pos = ray.origin;

        let step_size = calculate_size(move_size, ray.direction);
        let mut step_dir = Vector3::<RayScalar>::zero();
        let mut next_dist = Vector3::zero();
        if ray.direction.x < 0.0 {
            step_dir.x = -1.0;
            next_dist.x =
                -1.0 * (ray.origin.x.rem_euclid(move_size as RayScalar)) / ray.direction.x;
        } else {
            step_dir.x = 1.0;
            next_dist.x = (1.0 - ray.origin.x.rem_euclid(move_size as RayScalar)) / ray.direction.x;
        }

        if ray.direction.y < 0.0 {
            step_dir.y = -1.0;
            next_dist.y = (ray.origin.y.rem_euclid(move_size as RayScalar).neg()) / ray.direction.y;
        } else {
            step_dir.y = 1.0;
            next_dist.y = (1.0 - ray.origin.y.rem_euclid(move_size as RayScalar)) / ray.direction.y;
        }
        if ray.direction.z < 0.0 {
            step_dir.z = -1.0;
            next_dist.z = (ray.origin.z.rem_euclid(move_size as RayScalar).neg()) / ray.direction.z;
        } else {
            step_dir.z = 1.0;
            next_dist.z = (1.0 - ray.origin.z.rem_euclid(move_size as RayScalar)) / ray.direction.z;
        }

        loop {
            let min_idx = min_idx_vec(next_dist);
            let normal = if min_idx == 0 {
                //min_idx = 0
                voxel_pos.x += if step_dir.x.is_sign_positive() {
                    move_size
                } else {
                    -move_size
                };
                current_pos += ray.direction * next_dist.x;
                next_dist = next_dist.map(|f| f - next_dist.x);
                next_dist.x += step_size.x;
                Vector3::new(step_dir.x.neg(), 0.0, 0.0).normalize()
            } else if min_idx == 1 {
                //min_idx = 1
                voxel_pos.y += if step_dir.y.is_sign_positive() {
                    move_size
                } else {
                    -move_size
                };
                current_pos += ray.direction * next_dist.y;
                next_dist = next_dist.map(|f| f - next_dist.y);
                next_dist.y += step_size.y;
                Vector3::new(0.0, step_dir.y.neg(), 0.0).normalize()
            } else if min_idx == 2 {
                //min_idx = 2
                voxel_pos.z += if step_dir.z.is_sign_positive() {
                    move_size
                } else {
                    -move_size
                };
                current_pos += ray.direction * next_dist.z;
                next_dist = next_dist.map(|f| f - next_dist.z);
                next_dist.z += step_size.z;
                Vector3::new(0.0, 0.0, step_dir.z.neg()).normalize()
            } else {
                panic!("invalid min_idx")
            };

            if self.in_range(voxel_pos.map(|v| v as i32)) {
                if let Some(chunk) = self.get_homogenous_chunk(voxel_pos.map(|v| v as u32)) {
                    let leaf = chunk.leaf_value().unwrap();
                    if let Some(solid_voxel) = leaf.try_solid() {
                        return Some(OctTreeHitInfo {
                            hit_value: solid_voxel,
                            depth: ray.origin.distance(current_pos),
                            hit_position: current_pos,
                            normal,
                        });
                    } else {
                        let x_size = if ray.direction.x < 0. {
                            (voxel_pos.x % chunk.size as i32) - 1
                        } else {
                            (chunk.size as i32 - (voxel_pos.x % chunk.size as i32)) - 1
                        };
                        let y_size = if ray.direction.y < 0. {
                            voxel_pos.y % chunk.size as i32
                        } else {
                            chunk.size as i32 - (voxel_pos.x % chunk.size as i32)
                        };
                        let z_size = if ray.direction.z < 0. {
                            (voxel_pos.z % chunk.size as i32) - 1
                        } else {
                            (chunk.size as i32 - (voxel_pos.z % chunk.size as i32)) - 1
                        };
                        //move_size = max(min(min(x_size, y_size), z_size), 1);
                        //step_size = calculate_size(move_size, ray.direction);
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
    }

    fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        let mut solutions = (0..3)
            .flat_map(|axis_index| {
                [
                    (
                        axis_index,
                        ray.intersect_axis(axis_index, 0.0),
                        Vector3::new(
                            if axis_index == 0 { -1.0 } else { 0.0 },
                            if axis_index == 1 { -1.0 } else { 0.0 },
                            if axis_index == 2 { -1.0 } else { 0.0 },
                        ),
                    ),
                    (
                        axis_index,
                        ray.intersect_axis(axis_index, self.size as RayScalar),
                        Vector3::new(
                            if axis_index == 0 { 1.0 } else { 0.0 },
                            if axis_index == 1 { 1.0 } else { 0.0 },
                            if axis_index == 2 { 1.0 } else { 0.0 },
                        ),
                    ),
                ]
            })
            .map(|(axis, time, normal)| (axis, time, ray.at(time), normal))
            .filter(|(axis, _time, at, _normal)| {
                ((at[0] >= 0. && at[0] <= self.size as RayScalar) || *axis == 0)
                    && ((at[1] >= 0. && at[1] <= self.size as RayScalar) || *axis == 1)
                    && ((at[2] >= 0. && at[2] <= self.size as RayScalar) || *axis == 2)
            })
            .collect::<Vec<_>>();
        solutions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if ray.origin.x >= 0.0
            && ray.origin.x <= self.size as RayScalar
            && ray.origin.y >= 0.0
            && ray.origin.y <= self.size as RayScalar
            && ray.origin.z >= 0.0
            && ray.origin.z <= self.size as RayScalar
        {
            self.trace(ray)
        } else {
            if let Some((_axis_idx, _b, position, normal)) = solutions.first() {
                if let Some(solid) = self
                    .get(position.map(|v| v as u32).map(|v| {
                        if v < self.size {
                            v
                        } else {
                            self.size - 1
                        }
                    }))
                    .try_solid()
                {
                    Some(OctTreeHitInfo {
                        depth: ray.distance(Vector3::new(position.x, position.y, position.z)),
                        hit_position: *position,
                        hit_value: solid,
                        normal: *normal,
                    })
                } else {
                    self.trace(Ray {
                        direction: ray.direction.normalize(),
                        origin: *position,
                        time: ray.time,
                    })
                }
            } else {
                None
            }
        }
    }
}
