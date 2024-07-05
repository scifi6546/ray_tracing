use super::{prelude::distance, Leafable, OctTree, OctTreeChildren, OctTreeHitInfo, OctTreeNode};
use crate::prelude::Ray;
use cgmath::{prelude::*, Point3, Vector3};
use log::{error, info};

impl<T: Leafable> OctTree<T> {
    pub fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        self.root_node.trace_ray(ray)
    }
}
impl<T: Leafable> OctTreeNode<T> {
    fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        // getting the min distances

        match &self.children {
            OctTreeChildren::Leaf(val) => {
                if val.is_solid() {
                    if ray.origin.x > 0.0
                        && ray.origin.y > 0.0
                        && ray.origin.z > 0.0
                        && ray.origin.x < self.size as f32
                        && ray.origin.y < self.size as f32
                        && ray.origin.z < self.size as f32
                    {
                        let (axis, closest_time, normal) = (0..3)
                            .flat_map(|axis_index| {
                                [
                                    (
                                        axis_index,
                                        ray.intersect_axis(axis_index, 0.0),
                                        Vector3::new(
                                            if axis_index == 0 { -1.0f32 } else { 0.0 },
                                            if axis_index == 1 { -1.0f32 } else { 0.0 },
                                            if axis_index == 2 { -1.0f32 } else { 0.0 },
                                        ),
                                    ),
                                    (
                                        axis_index,
                                        ray.intersect_axis(axis_index, self.size as f32),
                                        Vector3::new(
                                            if axis_index == 0 { 1.0f32 } else { 0.0 },
                                            if axis_index == 1 { 1.0f32 } else { 0.0 },
                                            if axis_index == 2 { 1.0f32 } else { 0.0 },
                                        ),
                                    ),
                                ]
                            })
                            .filter(|(axis_index, time, _normal)| {
                                let pos = ray.local_at(*time);
                                let pos_good = [
                                    *axis_index == 0
                                        || (pos[0] >= 0. && pos[0] <= self.size as f32),
                                    *axis_index == 1
                                        || (pos[1] >= 0. && pos[1] <= self.size as f32),
                                    *axis_index == 2
                                        || (pos[2] >= 0. && pos[2] <= self.size as f32),
                                ];
                                pos_good[0] && pos_good[1] && pos_good[2]
                            })
                            .filter(|(_axis_index, time, _normal)| *time < 0.0)
                            .fold((4, f32::MAX, Vector3::new(0.0, 0.0, 0.0)), |acc, b| {
                                if acc.1 < b.1 {
                                    acc
                                } else {
                                    b
                                }
                            });
                        if axis != 4 {
                            let ray_pos = ray.local_at(closest_time);
                            Some(OctTreeHitInfo {
                                depth: 0.0,
                                hit_value: val.unwrap_ref(),
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
                                            Vector3::new(
                                                if axis == 0 { -1.0f32 } else { 0.0 },
                                                if axis == 1 { -1.0 } else { 0.0 },
                                                if axis == 2 { -1.0 } else { 0.0 },
                                            ),
                                        ),
                                        (
                                            axis,
                                            ray.intersect_axis(axis, self.size as f32),
                                            Vector3::new(
                                                if axis == 0 { -1.0f32 } else { 0.0 },
                                                if axis == 1 { -1.0 } else { 0.0 },
                                                if axis == 2 { -1.0 } else { 0.0 },
                                            ),
                                        ),
                                    ]
                                } else {
                                    [
                                        (
                                            axis,
                                            ray.intersect_axis(axis, self.size as f32),
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
                                    *idx == 0 || (pos[0] >= 0. && pos[0] <= self.size as f32),
                                    *idx == 1 || (pos[1] >= 0. && pos[1] <= self.size as f32),
                                    *idx == 2 || (pos[2] >= 0. && pos[2] <= self.size as f32),
                                ];
                                pos_good[0] && pos_good[1] && pos_good[2]
                            })
                            .filter(|(_idx, time, _normal)| {
                                ray.distance(ray.local_at(*time)).is_finite()
                            })
                            .fold((4, f32::MAX, Vector3::new(0.0f32, 0.0, 0.0)), |acc, x| {
                                if acc.1 < x.1 {
                                    acc
                                } else {
                                    x
                                }
                            });
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
                                hit_value: val.unwrap_ref(),
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
            OctTreeChildren::ParentNode(children) => {
                let mut tiles = (0..3)
                    .flat_map(|idx| {
                        if ray.direction[idx] >= 0. {
                            [
                                (ray.intersect_axis(idx, 0.0), 0u32),
                                (ray.intersect_axis(idx, self.size as f32 / 2.0), 1),
                            ]
                        } else {
                            [
                                (ray.intersect_axis(idx, self.size as f32 / 2.0), 0),
                                (ray.intersect_axis(idx, self.size as f32), 1),
                            ]
                        }
                        .map(|(time, idx_pos)| (idx, time, ray.local_at(time), idx_pos))
                    })
                    .filter(|(_idx, time, _pos, _axis_pos)| time.is_finite() && *time >= 0.)
                    .filter(|(idx, _dist, pos, _idx_pos)| {
                        let is_valid = pos.map(|v| v >= 0. && v <= self.size as f32);

                        (is_valid[0] || *idx == 0)
                            && (is_valid[1] || *idx == 1)
                            && (is_valid[2] || *idx == 2)
                    })
                    .filter_map(|(index, _dist, pos, idx_pos)| {
                        let floored_pos =
                            pos.map(|v| if v as u32 >= (self.size / 2) { 1 } else { 0 });

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
                            Some((
                                Self::get_child_index_size2(x, y, z),
                                Vector3::new(x, y, z),
                                pos,
                            ))
                        }
                    })
                    .collect::<Vec<_>>();

                tiles.sort_by(|a, b| {
                    let a_dist =
                        distance(Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z), a.2);
                    let b_dist =
                        distance(Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z), b.2);
                    a_dist.partial_cmp(&b_dist).unwrap()
                });
                for (index, tile_index, pos) in tiles {
                    let tile_pos_floored = tile_index.map(|v| (v * self.size / 2) as f32);

                    let origin = Point3::new(
                        pos.x - tile_pos_floored.x,
                        pos.y - tile_pos_floored.y,
                        pos.z - tile_pos_floored.z,
                    );
                    if let Some(hit_info) = children[index].trace_ray(Ray {
                        direction: ray.direction,
                        origin,
                        time: ray.time
                            + (origin - ray.origin).magnitude() / ray.direction.magnitude(),
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
        }
    }
}
