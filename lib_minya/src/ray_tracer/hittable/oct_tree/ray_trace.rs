use super::{Leafable, OctTree, OctTreeHitInfo, OctTreeNode};
use crate::prelude::{Ray, RayScalar};
use std::ops::Neg;

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
    fn trace(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        let step_size = 1.0 / ray.direction.map(|e| e.abs());
        let mut step_dir = Vector3::<RayScalar>::zero();
        let mut next_dist = Vector3::zero();
        if ray.direction.x < 0.0 {
            step_dir.x = -1.0;
            next_dist.x = -1.0 * (ray.origin.x.fract()) / ray.direction.x;
        } else {
            step_dir.x = 1.0;
            next_dist.x = (1.0 - ray.origin.x.fract()) / ray.direction.x;
        }

        if ray.direction.y < 0.0 {
            step_dir.y = -1.0;
            next_dist.y = (ray.origin.y.fract().neg()) / ray.direction.y;
        } else {
            step_dir.y = 1.0;
            next_dist.y = (1.0 - ray.origin.y.fract()) / ray.direction.y;
        }
        if ray.direction.z < 0.0 {
            step_dir.z = -1.0;
            next_dist.z = (ray.origin.z.fract().neg()) / ray.direction.z;
        } else {
            step_dir.z = 1.0;
            next_dist.z = (1.0 - ray.origin.z.fract()) / ray.direction.z;
        }

        let mut voxel_pos = ray.origin.map(|e| e as isize);
        let mut current_pos = ray.origin;

        loop {
            let min_idx = min_idx_vec(next_dist);
            let normal = if min_idx == 0 {
                //min_idx = 0
                voxel_pos.x += if step_dir.x.is_sign_positive() { 1 } else { -1 };
                current_pos += ray.direction * next_dist.x;
                next_dist = next_dist.map(|f| f - next_dist.x);
                next_dist.x += step_size.x;
                Vector3::new(step_dir.x.neg(), 0.0, 0.0).normalize()
            } else if min_idx == 1 {
                //min_idx = 1
                voxel_pos.y += if step_dir.y.is_sign_positive() { 1 } else { -1 };
                current_pos += ray.direction * next_dist.y;
                next_dist = next_dist.map(|f| f - next_dist.y);
                next_dist.y += step_size.y;
                Vector3::new(0.0, step_dir.y.neg(), 0.0).normalize()
            } else if min_idx == 2 {
                //min_idx = 2
                voxel_pos.z += if step_dir.z.is_sign_positive() { 1 } else { -1 };
                current_pos += ray.direction * next_dist.z;
                next_dist = next_dist.map(|f| f - next_dist.z);
                next_dist.z += step_size.z;
                Vector3::new(0.0, 0.0, step_dir.z.neg()).normalize()
            } else {
                panic!("invalid min_idx")
            };
            let x_pos = voxel_pos.x;
            let y_pos = voxel_pos.y;
            let z_pos = voxel_pos.z;
            if self.in_range(voxel_pos.map(|v| v as i32)) {
                let voxel = self.get(Point3::new(x_pos as u32, y_pos as u32, z_pos as u32));
                if let Some(solid_voxel) = voxel.try_solid() {
                    return Some(OctTreeHitInfo {
                        hit_value: solid_voxel,
                        depth: ray.origin.distance(current_pos),
                        hit_position: current_pos,
                        normal,
                    });
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
                    let offset = position - 0.1 * ray.direction.normalize();
                    self.trace(Ray {
                        origin: offset,
                        direction: ray.direction,
                        time: ray.time - 0.1,
                    })
                }
            } else {
                None
            }
        }
    }
}
