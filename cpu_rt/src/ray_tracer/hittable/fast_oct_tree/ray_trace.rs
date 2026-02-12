use super::{
    hit_info::HitInfo,
    leafable::Leafable,
    voxel::{VolumeEdgeEffect, VolumeVoxel, Voxel, VoxelMaterial},
    FastOctTree, Node, NodeData,
};
use crate::{
    prelude::{Ray, RayScalar},
    rand_scalar,
};
use cgmath::{prelude::*, Point3, Vector3};
use log::{error, warn};
use std::ops::Neg;
impl<T: Leafable> FastOctTree<T> {
    fn in_range(&self, position: Point3<i32>) -> bool {
        let world_size = self.world_size();
        let all_in_range = position.map(|v| v >= 0 && v < world_size as i32);
        all_in_range.x && all_in_range.y && all_in_range.z
    }
    fn get_step_size(&self, position: Point3<i32>) -> u32 {
        if !self.in_range(position) {
            panic!(
                "out of range, coordinates: <{}, {}, {}> world_size: {}",
                position.x,
                position.y,
                position.z,
                self.world_size()
            )
        }
        let root = self.arena.get_root().expect("should have root");
        let mut current_voxel = root.clone();
        let mut current_position = position.map(|v| v as u32);
        loop {
            match &current_voxel.data {
                NodeData::Parent { children } => {
                    let index = Node::<Voxel>::world_pos_to_child_index(
                        current_position,
                        current_voxel.size,
                    );
                    current_position =
                        Node::<T>::world_pos_to_child_pos(current_position, current_voxel.size);
                    current_voxel = self
                        .arena
                        .get(children[index as usize])
                        .expect("child should exist")
                        .clone();
                }
                NodeData::Leaf(_) => return current_voxel.get_world_size(),
                NodeData::Empty => return current_voxel.get_world_size(),
            }
        }
    }

    /// return the chunk that the pos is contained in, if the pos is inside of a leaf returns the entire leaf
    /// returns none if pos is out of range
    fn get_chunk(&self, position: Point3<u32>) -> Option<Node<T>> {
        if !self.in_range(position.map(|v| v as i32)) {
            return None;
        }

        if let Some(root) = self.arena.get_root() {
            let mut current_voxel = root.clone();
            let mut current_position = position;
            loop {
                match &current_voxel.data {
                    NodeData::Parent { children } => {
                        let index = Node::<Voxel>::world_pos_to_child_index(
                            current_position,
                            current_voxel.size,
                        );
                        current_position = Node::<Voxel>::world_pos_to_child_pos(
                            current_position,
                            current_voxel.size,
                        );
                        current_voxel = self
                            .arena
                            .get(children[index as usize])
                            .expect("child should exist")
                            .clone();
                    }
                    NodeData::Leaf(_) => return Some(current_voxel),
                    NodeData::Empty => return Some(current_voxel),
                }
            }
        } else {
            None
        }
    }
}
impl FastOctTree<Voxel> {
    fn ray_iteration(
        &self,
        block_coordinates: Point3<i32>,
        ray: Ray,
        initial_normal: Vector3<RayScalar>,
    ) -> Option<HitInfo<Voxel>> {
        #[derive(Clone, Copy, Debug)]
        struct VolumeDistanceLeftInfo {
            distance_left: RayScalar,
            first_density: RayScalar,
            last_position: Point3<RayScalar>,
            previous_volume: VolumeVoxel,
        }
        #[derive(Clone, Copy, Debug)]
        struct RayTraceState {
            volume_distance_left: Option<VolumeDistanceLeftInfo>,
            block_coordinates: Point3<i32>,
            current_position: Point3<RayScalar>,
        }
        enum HitOutput {
            ContinueIteration(RayTraceState),
            StopIterationVolume {
                stop_position: Point3<RayScalar>,
                hit_material: VoxelMaterial,
            },
            StopIterationSolid {
                hit_material: VoxelMaterial,
                normal: Vector3<RayScalar>,
            },
        }
        fn floor_value_integer(a: i32, b: i32) -> i32 {
            a - (a % b)
        }
        fn floor_point3_integer(coord: Point3<i32>, b: i32) -> Point3<i32> {
            coord.map(|v| floor_value_integer(v, b))
        }
        // gets the sign of the value, for example -123 -> -1 or 123 -> 1.
        fn int_sign(a: RayScalar) -> i32 {
            if a.is_sign_positive() {
                1
            } else {
                -1
            }
        }

        fn handle_volume(
            volume_material: VolumeVoxel,
            mut rt_state: RayTraceState,
            direction: Vector3<RayScalar>,
            initial_normal: Vector3<RayScalar>,
        ) -> HitOutput {
            if let Some(dist_info) = rt_state.volume_distance_left {
                // calculating ratio of previous materials density to starting density
                let previous_volume = dist_info.previous_volume;
                let previous_density = previous_volume.density;

                let density_ratio = dist_info.first_density / previous_density;
                let distance_traveled =
                    rt_state.current_position.distance(dist_info.last_position) * density_ratio;
                if distance_traveled > dist_info.distance_left {
                    let stop_position =
                        dist_info.last_position + dist_info.distance_left * direction.normalize();

                    HitOutput::StopIterationVolume {
                        stop_position,
                        hit_material: previous_volume.volume_material(),
                    }
                } else {
                    let new_distance_left = dist_info.distance_left - distance_traveled;

                    rt_state.volume_distance_left = Some(VolumeDistanceLeftInfo {
                        distance_left: new_distance_left,
                        last_position: rt_state.current_position,
                        first_density: dist_info.first_density,
                        previous_volume,
                    });

                    HitOutput::ContinueIteration(rt_state)
                }
            } else {
                match volume_material.edge_effect {
                    VolumeEdgeEffect::None => (),
                    VolumeEdgeEffect::Solid {
                        hit_probability,
                        solid_material,
                    } => {
                        let random_number = rand_scalar(0., 1.) as f32;
                        if random_number < hit_probability {
                            return HitOutput::StopIterationSolid {
                                hit_material: solid_material.to_material(),
                                normal: initial_normal,
                            };
                        }
                    }
                }
                let distance_left = rand_scalar(0., 1.).ln() / (volume_material.density.neg());

                rt_state.volume_distance_left = Some(VolumeDistanceLeftInfo {
                    distance_left,
                    last_position: rt_state.current_position,
                    first_density: volume_material.density,
                    previous_volume: volume_material,
                });

                HitOutput::ContinueIteration(rt_state)
            }
        }
        fn handle_empty(mut rt_state: RayTraceState, direction: Vector3<RayScalar>) -> HitOutput {
            if let Some(dist_info) = rt_state.volume_distance_left {
                let distance_traveled = rt_state.current_position.distance(dist_info.last_position);
                if distance_traveled > dist_info.distance_left {
                    let stop_position =
                        dist_info.last_position + dist_info.distance_left * direction.normalize();

                    HitOutput::StopIterationVolume {
                        stop_position,
                        hit_material: dist_info.previous_volume.volume_material(),
                    }
                } else {
                    rt_state.volume_distance_left = None;

                    HitOutput::ContinueIteration(rt_state)
                }
            } else {
                HitOutput::ContinueIteration(rt_state)
            }
        }
        const MAX_NUMBER_RAY_ITERATIONS: usize = 3000;
        let original_ray = ray;

        let mut rt_state = RayTraceState {
            block_coordinates: floor_point3_integer(
                block_coordinates,
                self.get_step_size(block_coordinates) as i32,
            ),
            current_position: ray.origin,
            volume_distance_left: None,
        };
        fn handle_hit(
            node_leaf: Option<Voxel>,
            rt_state: RayTraceState,
            ray: Ray,
            normal: Vector3<RayScalar>,
        ) -> HitOutput {
            if let Some(voxel) = node_leaf {
                match voxel {
                    Voxel::Solid(solid_material) => HitOutput::StopIterationSolid {
                        hit_material: solid_material.to_material(),
                        normal,
                    },
                    Voxel::Volume(volume) => handle_volume(volume, rt_state, ray.direction, normal),
                }
            } else {
                handle_empty(rt_state, ray.direction)
            }
        }

        {
            let chunk = self
                .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                .expect("out of range");
            let leaf = match chunk.data {
                NodeData::Leaf(v) => Some(v),
                NodeData::Parent { .. } => panic!("should not be parent"),
                NodeData::Empty => None,
            };
            if let Some(leaf) = leaf {
                match leaf {
                    Voxel::Volume(volume) => {
                        match handle_volume(volume, rt_state, ray.direction, initial_normal) {
                            HitOutput::ContinueIteration(new_state) => rt_state = new_state,
                            HitOutput::StopIterationVolume {
                                stop_position,
                                hit_material,
                            } => {
                                return Some(HitInfo::Volume {
                                    hit_value: hit_material,
                                    hit_position: stop_position,
                                })
                            }
                            HitOutput::StopIterationSolid {
                                hit_material,
                                normal,
                            } => {
                                return Some(HitInfo::Solid {
                                    hit_value: hit_material,
                                    hit_position: rt_state.current_position,
                                    normal,
                                })
                            }
                        }
                    }
                    Voxel::Solid(..) => {}
                }
            }
        }

        let x_sign = if ray.direction.x.is_sign_positive() {
            1
        } else {
            0
        };
        let y_sign = if ray.direction.y.is_sign_positive() {
            1
        } else {
            0
        };
        let z_sign = if ray.direction.z.is_sign_positive() {
            1
        } else {
            0
        };

        for _ in 0..MAX_NUMBER_RAY_ITERATIONS {
            let step_size = self.get_step_size(rt_state.block_coordinates);

            if !self.in_range(rt_state.block_coordinates) {
                return None;
            }

            let t_x = (rt_state.block_coordinates.x as RayScalar
                + step_size as RayScalar * x_sign as RayScalar
                - rt_state.current_position.x)
                / ray.direction.x;

            let t_y = (rt_state.block_coordinates.y as RayScalar
                + step_size as RayScalar * y_sign as RayScalar
                - rt_state.current_position.y)
                / ray.direction.y;

            let t_z = (rt_state.block_coordinates.z as RayScalar
                + step_size as RayScalar * z_sign as RayScalar
                - rt_state.current_position.z)
                / ray.direction.z;
            if t_x < 0. {
                error!("t_x < 0., t_x = {}", t_x);
                error!(
                    "ray origin: <{},{},{}>",
                    rt_state.current_position.x,
                    rt_state.current_position.y,
                    rt_state.current_position.z
                );
                error!(
                    "ray direction: <{}, {}, {}>",
                    ray.direction.x, ray.direction.y, ray.direction.z
                );
                error!("step size: {}", step_size);
                error!(
                    "block coordiates: <{},{},{}>",
                    rt_state.block_coordinates.x,
                    rt_state.block_coordinates.y,
                    rt_state.block_coordinates.z
                );
                error!(
                    "position: <{}, {}, {}>",
                    rt_state.current_position.x,
                    rt_state.current_position.y,
                    rt_state.current_position.z
                );
                return None;
            }
            if t_y < 0. {
                error!("t_y < 0., t_y = {}, original ray: {}", t_y, original_ray);
                return None;
            }
            if t_z < 0. {
                error!("t_z < 0., t_z = {}, original ray: {}", t_z, original_ray);
                return None;
            }

            if t_x < t_y && t_x < t_z {
                // t_x is the min
                rt_state.current_position.y += t_x * ray.direction.y;
                rt_state.current_position.z += t_x * ray.direction.z;
                if ray.direction.x >= 0. {
                    rt_state.block_coordinates.x += step_size as i32 * int_sign(ray.direction.x);
                    rt_state.current_position.x += t_x * ray.direction.x;
                    if self.in_range(Point3::new(
                        rt_state.block_coordinates.x as i32,
                        rt_state.current_position.y as i32,
                        rt_state.current_position.z as i32,
                    )) {
                        let next_step_size = self.get_step_size(Point3::new(
                            rt_state.block_coordinates.x,
                            rt_state.current_position.y as i32,
                            rt_state.current_position.z as i32,
                        )) as i32;

                        rt_state.block_coordinates.y =
                            floor_value_integer(rt_state.current_position.y as i32, next_step_size);
                        rt_state.block_coordinates.z =
                            floor_value_integer(rt_state.current_position.z as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match node.data {
                            NodeData::Leaf(leaf) => Some(leaf),
                            NodeData::Empty => None,
                            NodeData::Parent { .. } => panic!("should not be parent node"),
                        };

                        match handle_hit(node_leaf, rt_state, ray, Vector3::new(-1., 0., 0.)) {
                            HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                            HitOutput::StopIterationVolume {
                                stop_position,
                                hit_material,
                            } => {
                                return Some(HitInfo::Volume {
                                    hit_value: hit_material,
                                    hit_position: stop_position,
                                })
                            }
                            HitOutput::StopIterationSolid {
                                hit_material,
                                normal,
                            } => {
                                return Some(HitInfo::Solid {
                                    hit_value: hit_material,
                                    hit_position: rt_state.current_position,
                                    normal,
                                })
                            }
                        };
                    } else {
                        return None;
                    }
                } else if self.in_range(Point3::new(
                    rt_state.block_coordinates.x as i32 - 1,
                    rt_state.block_coordinates.y as i32,
                    rt_state.block_coordinates.z as i32,
                )) {
                    rt_state.current_position.x += t_x * ray.direction.x;
                    let next_step_size = self.get_step_size(Point3::new(
                        rt_state.block_coordinates.x - 1,
                        rt_state.current_position.y as i32,
                        rt_state.current_position.z as i32,
                    ));

                    rt_state.block_coordinates.y = floor_value_integer(
                        rt_state.current_position.y as i32,
                        next_step_size as i32,
                    );
                    rt_state.block_coordinates.z = floor_value_integer(
                        rt_state.current_position.z as i32,
                        next_step_size as i32,
                    );
                    rt_state.block_coordinates.x -= next_step_size as i32;

                    let node = self
                        .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                        .expect("should be in range");

                    let node_leaf = match node.data {
                        NodeData::Leaf(leaf) => Some(leaf),
                        NodeData::Empty => None,
                        NodeData::Parent { .. } => panic!("should not be parent node"),
                    };
                    match handle_hit(node_leaf, rt_state, ray, Vector3::new(1., 0., 0.)) {
                        HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                        HitOutput::StopIterationVolume {
                            stop_position,
                            hit_material,
                        } => {
                            return Some(HitInfo::Volume {
                                hit_value: hit_material,
                                hit_position: stop_position,
                            })
                        }
                        HitOutput::StopIterationSolid {
                            hit_material,
                            normal,
                        } => {
                            return Some(HitInfo::Solid {
                                hit_value: hit_material,
                                hit_position: rt_state.current_position,
                                normal,
                            })
                        }
                    };
                } else {
                    return None;
                }
            } else if t_y < t_x && t_y < t_z {
                // y is the min
                rt_state.current_position.x += t_y * ray.direction.x;
                rt_state.current_position.z += t_y * ray.direction.z;
                if ray.direction.y >= 0. {
                    rt_state.block_coordinates.y += step_size as i32 * int_sign(ray.direction.y);
                    rt_state.current_position.y += t_y * ray.direction.y;
                    if self.in_range(Point3::new(
                        rt_state.current_position.x as i32,
                        rt_state.block_coordinates.y as i32,
                        rt_state.current_position.z as i32,
                    )) {
                        let next_step_size = self.get_step_size(Point3::new(
                            rt_state.current_position.x as i32,
                            rt_state.block_coordinates.y,
                            rt_state.current_position.z as i32,
                        )) as i32;

                        rt_state.block_coordinates.x =
                            floor_value_integer(rt_state.current_position.x as i32, next_step_size);
                        rt_state.block_coordinates.z =
                            floor_value_integer(rt_state.current_position.z as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match node.data {
                            NodeData::Leaf(leaf) => Some(leaf),
                            NodeData::Empty => None,
                            NodeData::Parent { .. } => panic!("should not be parent node"),
                        };
                        match handle_hit(node_leaf, rt_state, ray, Vector3::new(0., -1., 0.)) {
                            HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                            HitOutput::StopIterationVolume {
                                stop_position,
                                hit_material,
                            } => {
                                return Some(HitInfo::Volume {
                                    hit_value: hit_material,
                                    hit_position: stop_position,
                                })
                            }
                            HitOutput::StopIterationSolid {
                                hit_material,
                                normal,
                            } => {
                                return Some(HitInfo::Solid {
                                    hit_value: hit_material,
                                    hit_position: rt_state.current_position,
                                    normal,
                                })
                            }
                        };
                    } else {
                        return None;
                    }
                } else if self.in_range(Point3::new(
                    rt_state.block_coordinates.x as i32,
                    rt_state.block_coordinates.y as i32 - 1,
                    rt_state.block_coordinates.z as i32,
                )) {
                    rt_state.current_position.y += t_y * ray.direction.y;
                    let next_step_size = self.get_step_size(Point3::new(
                        rt_state.current_position.x as i32,
                        rt_state.block_coordinates.y - 1,
                        rt_state.current_position.z as i32,
                    )) as i32;

                    rt_state.block_coordinates.x = floor_value_integer(
                        rt_state.current_position.x as i32,
                        next_step_size as i32,
                    );
                    rt_state.block_coordinates.z = floor_value_integer(
                        rt_state.current_position.z as i32,
                        next_step_size as i32,
                    );
                    rt_state.block_coordinates.y -= next_step_size as i32;
                    let node = self
                        .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                        .expect("should be in range");
                    let node_leaf = match node.data {
                        NodeData::Leaf(leaf) => Some(leaf),
                        NodeData::Empty => None,
                        NodeData::Parent { .. } => panic!("should not be parent node"),
                    };
                    match handle_hit(node_leaf, rt_state, ray, Vector3::new(0., 1., 0.)) {
                        HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                        HitOutput::StopIterationVolume {
                            stop_position,
                            hit_material,
                        } => {
                            return Some(HitInfo::Volume {
                                hit_value: hit_material,
                                hit_position: stop_position,
                            })
                        }
                        HitOutput::StopIterationSolid {
                            hit_material,
                            normal,
                        } => {
                            return Some(HitInfo::Solid {
                                hit_value: hit_material,
                                hit_position: rt_state.current_position,
                                normal,
                            })
                        }
                    };
                } else {
                    return None;
                }
            } else {
                // z is the min
                rt_state.current_position.y += t_z * ray.direction.y;
                rt_state.current_position.x += t_z * ray.direction.x;
                if ray.direction.z >= 0. {
                    rt_state.block_coordinates.z += step_size as i32 * int_sign(ray.direction.z);
                    rt_state.current_position.z += t_z * ray.direction.z;
                    if self.in_range(Point3::new(
                        rt_state.current_position.x as i32,
                        rt_state.current_position.y as i32,
                        rt_state.block_coordinates.z as i32,
                    )) {
                        let next_step_size = self.get_step_size(Point3::new(
                            rt_state.current_position.x as i32,
                            rt_state.current_position.y as i32,
                            rt_state.block_coordinates.z as i32,
                        )) as i32;

                        rt_state.block_coordinates.y =
                            floor_value_integer(rt_state.current_position.y as i32, next_step_size);
                        rt_state.block_coordinates.x =
                            floor_value_integer(rt_state.current_position.x as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match node.data {
                            NodeData::Leaf(leaf) => Some(leaf),
                            NodeData::Empty => None,
                            NodeData::Parent { .. } => panic!("should not be parent node"),
                        };
                        match handle_hit(node_leaf, rt_state, ray, Vector3::new(0., 0., -1.)) {
                            HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                            HitOutput::StopIterationVolume {
                                stop_position,
                                hit_material,
                            } => {
                                return Some(HitInfo::Volume {
                                    hit_value: hit_material,
                                    hit_position: stop_position,
                                })
                            }
                            HitOutput::StopIterationSolid {
                                hit_material,
                                normal,
                            } => {
                                return Some(HitInfo::Solid {
                                    hit_value: hit_material,
                                    hit_position: rt_state.current_position,
                                    normal,
                                })
                            }
                        };
                    } else {
                        return None;
                    }
                } else if self.in_range(Point3::new(
                    rt_state.block_coordinates.x as i32,
                    rt_state.block_coordinates.y as i32,
                    rt_state.block_coordinates.z as i32 - 1,
                )) {
                    rt_state.current_position.z += t_z * ray.direction.z;
                    let next_step_size = self.get_step_size(Point3::new(
                        rt_state.current_position.x as i32,
                        rt_state.current_position.y as i32,
                        rt_state.block_coordinates.z - 1,
                    )) as i32;

                    rt_state.block_coordinates.x = floor_value_integer(
                        rt_state.current_position.x as i32,
                        next_step_size as i32,
                    );
                    rt_state.block_coordinates.y = floor_value_integer(
                        rt_state.current_position.y as i32,
                        next_step_size as i32,
                    );

                    rt_state.block_coordinates.z -= next_step_size as i32;

                    let node = self
                        .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                        .expect("should be in range");
                    let node_leaf = match node.data {
                        NodeData::Leaf(leaf) => Some(leaf),
                        NodeData::Empty => None,
                        NodeData::Parent { .. } => panic!("should not be parent node"),
                    };
                    match handle_hit(node_leaf, rt_state, ray, Vector3::new(0., 0., 1.)) {
                        HitOutput::ContinueIteration(new_rt_state) => rt_state = new_rt_state,
                        HitOutput::StopIterationVolume {
                            stop_position,
                            hit_material,
                        } => {
                            return Some(HitInfo::Volume {
                                hit_value: hit_material,
                                hit_position: stop_position,
                            })
                        }
                        HitOutput::StopIterationSolid {
                            hit_material,
                            normal,
                        } => {
                            return Some(HitInfo::Solid {
                                hit_value: hit_material,
                                hit_position: rt_state.current_position,
                                normal,
                            })
                        }
                    }
                } else {
                    return None;
                }
            }
        }
        warn!(
            "Max number of iterations reached, num_iterations: {}",
            MAX_NUMBER_RAY_ITERATIONS
        );
        None
    }
    pub fn trace_ray(&self, ray: Ray) -> Option<HitInfo<Voxel>> {
        #[derive(Debug)]
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
                                    size_intersect_position.x.round() as i32 - 1
                                } else {
                                    size_intersect_position.x as i32
                                },
                                if normal_axis == 1 {
                                    size_intersect_position.y.round() as i32 - 1
                                } else {
                                    size_intersect_position.y as i32
                                },
                                if normal_axis == 2 {
                                    size_intersect_position.z.round() as i32 - 1
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
                if let Some(voxel) =
                    self.get(intersection.block_coordinate.map(|v| v as u32).map(|v| v))
                {
                    match voxel {
                        Voxel::Solid(solid_material) => Some(HitInfo::Solid {
                            hit_value: solid_material.to_material(),
                            hit_position: intersection.intersect_position,
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
                    }
                } else {
                    self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                        intersection.normal_vector,
                    )
                }
            } else {
                None
            }
        }
    }
}
#[cfg(test)]
mod test {
    use crate::{prelude::RgbColor, ray_tracer::hittable::fast_oct_tree::SolidVoxel};

    use super::*;
    #[test]
    fn get_empty_chunk() {
        let t = FastOctTree::<Voxel>::new();
        assert!(t.get_chunk(Point3 { x: 0, y: 0, z: 0 }).is_none())
    }
    #[test]
    fn get_size_zero_chunk() {
        let mut t = FastOctTree::<Voxel>::new();
        t.set(
            Voxel::Solid(SolidVoxel::Lambertian {
                albedo: RgbColor::WHITE,
            }),
            Point3::new(0, 0, 0),
        );
        let c = t.get_chunk(Point3 { x: 0, y: 0, z: 0 }).unwrap();
        assert_eq!(c.size, 0);
    }
    #[test]
    fn get_size_one_chunk() {
        let mut t = FastOctTree::<Voxel>::new();
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    t.set(
                        Voxel::Solid(SolidVoxel::Lambertian {
                            albedo: RgbColor::WHITE,
                        }),
                        Point3::new(x, y, z),
                    );
                }
            }
        }
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let c = t.get_chunk(Point3 { x, y, z }).unwrap();
                    assert_eq!(c.size, 1);
                }
            }
        }
    }
    #[test]
    fn get_chunk_with_hole() {
        let mut t = FastOctTree::<Voxel>::new();
        let empty_position = Point3::new(0, 0, 0);
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let set_position = Point3::new(x, y, z);
                    if set_position != empty_position {
                        t.set(
                            Voxel::Solid(SolidVoxel::Lambertian {
                                albedo: RgbColor::WHITE,
                            }),
                            set_position,
                        );
                    }
                }
            }
        }
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let get_position = Point3 { x, y, z };
                    let chunk = t.get_chunk(get_position).unwrap();

                    if get_position == empty_position {
                        match chunk.data {
                            NodeData::Empty => {}
                            NodeData::Leaf(_) => panic!("node data should be empty"),
                            NodeData::Parent { .. } => panic!("node data should be leaf"),
                        }
                    } else {
                        match chunk.data {
                            NodeData::Empty => panic!("node data should be leaf"),
                            NodeData::Leaf(_) => {}
                            NodeData::Parent { .. } => panic!("node data should be leaf"),
                        }
                    }

                    assert_eq!(chunk.size, 0)
                }
            }
        }
    }
    #[test]
    fn get_step_size_single_block() {
        let mut t = FastOctTree::<u32>::new();
        t.set(0, Point3::new(0, 0, 0));
        assert_eq!(t.get_step_size(Point3::new(0, 0, 0)), 1);
    }
    #[test]
    fn get_step_size() {
        let mut t = FastOctTree::<u32>::new();
        let empty_position = Point3::new(0, 0, 0);
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let set_position = Point3::new(x, y, z);
                    if set_position != empty_position {
                        t.set(0, set_position);
                    }
                }
            }
        }
        for x in 0..2 {
            for y in 0..2 {
                for z in 0..2 {
                    let coordinates = Point3 { x, y, z };
                    assert_eq!(t.get_step_size(coordinates), 1);
                }
            }
        }
    }
}
