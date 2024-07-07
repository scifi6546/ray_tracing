use super::{
    get_child_index_size2, prelude::distance, LeafType, Leafable, OctTree, OctTreeChildren,
    OctTreeHitInfo, OctTreeNode,
};
use crate::prelude::{Ray, RayScalar};
use std::ops::Neg;

use cgmath::{prelude::*, Point3, Vector3};
use log::{error, info};
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
    fn trace_v2<'a>(&'a self, ray: Ray) -> Option<OctTreeHitInfo<'a, T>> {
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
                let voxel = self.get(x_pos as u32, y_pos as u32, z_pos as u32);
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
    fn leaf_trace<'a>(
        ray: &Ray,
        leaf: &'a LeafType<T>,
        size: u32,
    ) -> Option<OctTreeHitInfo<'a, T>> {
        if leaf.is_solid() {
            if ray.origin.x > 0.0
                && ray.origin.y > 0.0
                && ray.origin.z > 0.0
                && ray.origin.x < size as RayScalar
                && ray.origin.y < size as RayScalar
                && ray.origin.z < size as RayScalar
            {
                let (axis, closest_time, normal) = (0..3)
                    .flat_map(|axis_index| {
                        [
                            (
                                axis_index,
                                ray.intersect_axis(axis_index, 0.0),
                                Vector3::<RayScalar>::new(
                                    if axis_index == 0 { -1.0 } else { 0.0 },
                                    if axis_index == 1 { -1.0 } else { 0.0 },
                                    if axis_index == 2 { -1.0 } else { 0.0 },
                                ),
                            ),
                            (
                                axis_index,
                                ray.intersect_axis(axis_index, size as RayScalar),
                                Vector3::<RayScalar>::new(
                                    if axis_index == 0 { 1.0 } else { 0.0 },
                                    if axis_index == 1 { 1.0 } else { 0.0 },
                                    if axis_index == 2 { 1.0 } else { 0.0 },
                                ),
                            ),
                        ]
                    })
                    .filter(|(axis_index, time, _normal)| {
                        let pos = ray.local_at(*time);
                        let pos_good = [
                            *axis_index == 0 || (pos[0] >= 0. && pos[0] <= size as RayScalar),
                            *axis_index == 1 || (pos[1] >= 0. && pos[1] <= size as RayScalar),
                            *axis_index == 2 || (pos[2] >= 0. && pos[2] <= size as RayScalar),
                        ];
                        pos_good[0] && pos_good[1] && pos_good[2]
                    })
                    .filter(|(_axis_index, time, _normal)| *time < 0.0)
                    .fold(
                        (4, RayScalar::MAX, Vector3::new(0.0, 0.0, 0.0)),
                        |acc, b| {
                            if acc.1 < b.1 {
                                acc
                            } else {
                                b
                            }
                        },
                    );
                if axis != 4 {
                    let ray_pos = ray.local_at(closest_time);
                    Some(OctTreeHitInfo {
                        depth: 0.0,
                        hit_value: leaf.unwrap_ref(),
                        hit_position: Point3::new(ray_pos.x, ray_pos.y, ray_pos.z),
                        normal,
                    })
                } else {
                    info!("ray?? : {:#?}", ray);
                    None
                }
            } else {
                let (axis, time, normal) = (0..3)
                    .flat_map(|axis| {
                        if ray.direction[axis] >= 0.0 {
                            [
                                (
                                    axis,
                                    ray.intersect_axis(axis, 0.0),
                                    Vector3::<RayScalar>::new(
                                        if axis == 0 { -1.0 } else { 0.0 },
                                        if axis == 1 { -1.0 } else { 0.0 },
                                        if axis == 2 { -1.0 } else { 0.0 },
                                    ),
                                ),
                                (
                                    axis,
                                    ray.intersect_axis(axis, size as RayScalar),
                                    Vector3::<RayScalar>::new(
                                        if axis == 0 { -1.0 } else { 0.0 },
                                        if axis == 1 { -1.0 } else { 0.0 },
                                        if axis == 2 { -1.0 } else { 0.0 },
                                    ),
                                ),
                            ]
                        } else {
                            [
                                (
                                    axis,
                                    ray.intersect_axis(axis, size as RayScalar),
                                    Vector3::new(
                                        if axis == 0 { 1.0 } else { 0.0 },
                                        if axis == 1 { 1.0 } else { 0.0 },
                                        if axis == 2 { 1.0 } else { 0.0 },
                                    ),
                                ),
                                (
                                    axis,
                                    ray.intersect_axis(axis, 0.0),
                                    Vector3::new(
                                        if axis == 0 { 1.0 } else { 0.0 },
                                        if axis == 1 { 1.0 } else { 0.0 },
                                        if axis == 2 { 1.0 } else { 0.0 },
                                    ),
                                ),
                            ]
                        }
                    })
                    .filter(|(_idx, t, _normal)| *t + 0.1 >= 0. && true)
                    .filter(|(idx, time, _normal)| {
                        let pos = ray.local_at(*time);
                        let pos_good = [
                            *idx == 0 || (pos[0] >= 0. && pos[0] <= size as RayScalar),
                            *idx == 1 || (pos[1] >= 0. && pos[1] <= size as RayScalar),
                            *idx == 2 || (pos[2] >= 0. && pos[2] <= size as RayScalar),
                        ];
                        pos_good[0] && pos_good[1] && pos_good[2]
                    })
                    .filter(|(_idx, time, _normal)| ray.distance(ray.local_at(*time)).is_finite())
                    .fold(
                        (4, RayScalar::MAX, Vector3::<RayScalar>::new(0.0, 0.0, 0.0)),
                        |acc, x| {
                            if acc.1 < x.1 {
                                acc
                            } else {
                                x
                            }
                        },
                    );
                if axis != 4 {
                    let d = ray.distance(ray.local_at(time));
                    if d.is_infinite() {
                        println!("INFINITE!!!!");
                        println!("time: {}, idx: {}", time, axis);
                        panic!()
                    }
                    let pos = ray.local_at(time);

                    Some(OctTreeHitInfo {
                        depth: d,
                        hit_value: leaf.unwrap_ref(),
                        hit_position: Point3::new(pos.x, pos.y, pos.z),
                        normal,
                    })
                } else {
                    None
                }
            }
        } else {
            None
        }
    }
    fn parent_trace<'a>(
        ray: &Ray,
        children: &'a Box<[OctTreeNode<T>; 8]>,
        size: u32,
    ) -> Option<OctTreeHitInfo<'a, T>> {
        let mut tiles = (0..3)
            .flat_map(|idx| {
                if ray.direction[idx] >= 0. {
                    [
                        (ray.intersect_axis(idx, 0.0), 0u32),
                        (ray.intersect_axis(idx, size as RayScalar / 2.0), 1),
                    ]
                } else {
                    [
                        (ray.intersect_axis(idx, size as RayScalar / 2.0), 0),
                        (ray.intersect_axis(idx, size as RayScalar), 1),
                    ]
                }
                .map(|(time, idx_pos)| (idx, time, ray.local_at(time), idx_pos))
            })
            .filter(|(_idx, time, _pos, _axis_pos)| time.is_finite() && *time >= 0.)
            .filter(|(idx, _dist, pos, _idx_pos)| {
                let is_valid = pos.map(|v| v >= 0. && v <= size as RayScalar);

                (is_valid[0] || *idx == 0)
                    && (is_valid[1] || *idx == 1)
                    && (is_valid[2] || *idx == 2)
            })
            .filter_map(|(index, _dist, pos, idx_pos)| {
                let floored_pos = pos.map(|v| if v as u32 >= (size / 2) { 1 } else { 0 });

                let x = if index == 0 { idx_pos } else { floored_pos.x };
                let y = if index == 1 { idx_pos } else { floored_pos.y };
                let z = if index == 2 { idx_pos } else { floored_pos.z };
                if x >= 2 || y >= 2 || z >= 2 {
                    error!("get index larger");
                    error!(
                        "ray {:#?},x: {}, y:{},z: {},\nindex: {}",
                        ray, x, y, z, index
                    );
                    error!("pos: {:#?}", pos);

                    None
                } else {
                    Some((get_child_index_size2(x, y, z), Vector3::new(x, y, z), pos))
                }
            })
            .collect::<Vec<_>>();

        tiles.sort_by(|a, b| {
            let a_dist = distance(Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z), a.2);
            let b_dist = distance(Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z), b.2);
            a_dist.partial_cmp(&b_dist).unwrap()
        });
        for (index, tile_index, pos) in tiles {
            let tile_pos_floored = tile_index.map(|v| (v * size / 2) as RayScalar);

            let origin = Point3::new(
                pos.x - tile_pos_floored.x,
                pos.y - tile_pos_floored.y,
                pos.z - tile_pos_floored.z,
            );
            if let Some(hit_info) = children[index].trace_ray(Ray {
                direction: ray.direction,
                origin,
                time: ray.time + (origin - ray.origin).magnitude() / ray.direction.magnitude(),
            }) {
                let hit_position = hit_info.hit_position + tile_pos_floored;

                return Some(OctTreeHitInfo {
                    depth: distance(
                        Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z),
                        Vector3::new(hit_position.x, hit_position.y, hit_position.z),
                    ),
                    hit_value: hit_info.hit_value,
                    hit_position,
                    normal: hit_info.normal,
                });
            }
        }
        None
    }
    fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        // getting the min distances

        match &self.children {
            OctTreeChildren::Leaf(val) => Self::leaf_trace(&ray, val, self.size),
            OctTreeChildren::ParentNode(children) => Self::parent_trace(&ray, children, self.size),
        }
    }
}
