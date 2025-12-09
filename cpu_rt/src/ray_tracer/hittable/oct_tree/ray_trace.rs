use super::{Leafable, OctTree, OctTreeChildren, OctTreeHitInfo, OctTreeNode, VoxelMaterial};
use crate::{
    prelude::{rand_scalar, Ray, RayScalar},
    ray_tracer::hittable::oct_tree::HitType,
};
use log::{error, warn};
use std::ops::Neg;

use cgmath::{prelude::*, Point3, Vector3};

impl OctTree<VoxelMaterial> {
    pub(crate) fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<VoxelMaterial>> {
        self.root_node.trace_ray(ray)
    }
}
impl OctTreeNode<VoxelMaterial> {
    fn in_range(&self, position: Point3<i32>) -> bool {
        let is_good = position.map(|v| v >= 0 && v < self.size as i32);
        is_good[0] && is_good[1] && is_good[2]
    }
    fn ray_iteration(
        &self,
        block_coordinates: Point3<i32>,
        ray: Ray,
    ) -> Option<OctTreeHitInfo<VoxelMaterial>> {
        #[derive(Clone, Copy, Debug)]
        struct VolumeDistanceLeftInfo {
            distance_left: RayScalar,
            first_density: RayScalar,
            last_position: Point3<RayScalar>,
        }
        #[derive(Clone, Copy, Debug)]
        struct RayTraceState {
            volume_distance_left: Option<VolumeDistanceLeftInfo>,
            block_coordinates: Point3<i32>,
            current_position: Point3<RayScalar>,
            previous_material: VoxelMaterial,
        }
        enum VolumeOutput {
            ContinueIteration(RayTraceState),
            StopIteration {
                stop_position: Point3<RayScalar>,
                hit_material: VoxelMaterial,
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
        fn get_step_size<T: Leafable>(s: &OctTreeNode<T>, coordinates: Point3<i32>) -> u32 {
            s.get_homogenous_chunk(coordinates.map(|v| v as u32))
                .expect("Index is out of range")
                .size
        }

        fn handle_volume(
            hit_material: VoxelMaterial,
            mut rt_state: RayTraceState,
            direction: Vector3<RayScalar>,
        ) -> VolumeOutput {
            if let Some(dist_info) = rt_state.volume_distance_left {
                // calculating ratio of previous materials density to starting density
                let previous_density = match rt_state.previous_material {
                    VoxelMaterial::Volume { density, .. } => density,
                    _ => panic!("must be volume"),
                };
                let density_ratio = dist_info.first_density / previous_density;
                let distance_traveled =
                    rt_state.current_position.distance(dist_info.last_position) * density_ratio;
                if distance_traveled > dist_info.distance_left {
                    let stop_position =
                        dist_info.last_position + dist_info.distance_left * direction.normalize();
                    VolumeOutput::StopIteration {
                        stop_position,
                        hit_material,
                    }
                } else {
                    let new_distance_left = dist_info.distance_left - distance_traveled;

                    rt_state.volume_distance_left = Some(VolumeDistanceLeftInfo {
                        distance_left: new_distance_left,
                        last_position: rt_state.current_position,
                        first_density: dist_info.first_density,
                    });
                    rt_state.previous_material = hit_material;
                    VolumeOutput::ContinueIteration(rt_state)
                }
            } else {
                rt_state.previous_material = hit_material;
                let density = match hit_material {
                    VoxelMaterial::Volume { density, .. } => density,
                    _ => panic!("invalid material"),
                };
                let distance_left = rand_scalar(0., 1.).ln() / (density.neg());

                rt_state.volume_distance_left = Some(VolumeDistanceLeftInfo {
                    distance_left,
                    last_position: rt_state.current_position,
                    first_density: density,
                });

                VolumeOutput::ContinueIteration(rt_state)
            }
        }
        fn handle_empty(
            mut rt_state: RayTraceState,
            direction: Vector3<RayScalar>,
        ) -> VolumeOutput {
            if let Some(dist_info) = rt_state.volume_distance_left {
                let distance_traveled = rt_state.current_position.distance(dist_info.last_position);
                if distance_traveled > dist_info.distance_left {
                    let stop_position =
                        dist_info.last_position + dist_info.distance_left * direction.normalize();
                    VolumeOutput::StopIteration {
                        stop_position,
                        hit_material: rt_state.previous_material,
                    }
                } else {
                    rt_state.volume_distance_left = None;
                    rt_state.previous_material = VoxelMaterial::Empty;
                    VolumeOutput::ContinueIteration(rt_state)
                }
            } else {
                rt_state.previous_material = VoxelMaterial::Empty;
                VolumeOutput::ContinueIteration(rt_state)
            }
        }
        const MAX_NUMBER_RAY_ITERATIONS: usize = 3000;
        let original_ray = ray;

        let mut rt_state = RayTraceState {
            block_coordinates: floor_point3_integer(
                block_coordinates,
                get_step_size(self, block_coordinates) as i32,
            ),
            current_position: ray.origin,
            volume_distance_left: None,
            previous_material: VoxelMaterial::Empty,
        };
        {
            let chunk = self
                .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                .expect("out of range");
            let leaf = match chunk.children {
                OctTreeChildren::Leaf(v) => v,
                OctTreeChildren::ParentNode(_) => panic!("should be leaf"),
            };
            match leaf {
                VoxelMaterial::Volume { .. } => {
                    match handle_volume(leaf, rt_state, ray.direction) {
                        VolumeOutput::ContinueIteration(new_state) => rt_state = new_state,
                        VolumeOutput::StopIteration { .. } => panic!("should never happen"),
                    }
                }
                VoxelMaterial::Solid { .. } => {}
                VoxelMaterial::Empty => {}
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
            let step_size = get_step_size(self, rt_state.block_coordinates);

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
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.block_coordinates.x,
                                rt_state.current_position.y as i32,
                                rt_state.current_position.z as i32,
                            ),
                        ) as i32;
                        rt_state.block_coordinates.y =
                            floor_value_integer(rt_state.current_position.y as i32, next_step_size);
                        rt_state.block_coordinates.z =
                            floor_value_integer(rt_state.current_position.z as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(-1., 0., 0.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        }
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        rt_state.block_coordinates.x as i32 - 1,
                        rt_state.block_coordinates.y as i32,
                        rt_state.block_coordinates.z as i32,
                    )) {
                        rt_state.current_position.x += t_x * ray.direction.x;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.block_coordinates.x - 1,
                                rt_state.current_position.y as i32,
                                rt_state.current_position.z as i32,
                            ),
                        );

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
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(0., 0., 1.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        }
                    } else {
                        return None;
                    }
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
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.current_position.x as i32,
                                rt_state.block_coordinates.y,
                                rt_state.current_position.z as i32,
                            ),
                        ) as i32;
                        rt_state.block_coordinates.x =
                            floor_value_integer(rt_state.current_position.x as i32, next_step_size);
                        rt_state.block_coordinates.z =
                            floor_value_integer(rt_state.current_position.z as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(0., -1., 0.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        };
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        rt_state.block_coordinates.x as i32,
                        rt_state.block_coordinates.y as i32 - 1,
                        rt_state.block_coordinates.z as i32,
                    )) {
                        rt_state.current_position.y += t_y * ray.direction.y;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.current_position.x as i32,
                                rt_state.block_coordinates.y - 1,
                                rt_state.current_position.z as i32,
                            ),
                        );
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
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(0., 1., 0.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        }
                    } else {
                        return None;
                    }
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
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.current_position.x as i32,
                                rt_state.current_position.y as i32,
                                rt_state.block_coordinates.z as i32,
                            ),
                        ) as i32;
                        rt_state.block_coordinates.y =
                            floor_value_integer(rt_state.current_position.y as i32, next_step_size);
                        rt_state.block_coordinates.x =
                            floor_value_integer(rt_state.current_position.x as i32, next_step_size);

                        let node = self
                            .get_chunk(rt_state.block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(0., 0., -1.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        }
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        rt_state.block_coordinates.x as i32,
                        rt_state.block_coordinates.y as i32,
                        rt_state.block_coordinates.z as i32 - 1,
                    )) {
                        rt_state.current_position.z += t_z * ray.direction.z;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                rt_state.current_position.x as i32,
                                rt_state.current_position.y as i32,
                                rt_state.block_coordinates.z - 1,
                            ),
                        );
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
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        match node_leaf.hit_type() {
                            HitType::Solid => {
                                let normal = Vector3::new(0., 0., 1.);

                                return Some(OctTreeHitInfo::Solid {
                                    hit_value: node_leaf,
                                    hit_position: rt_state.current_position,
                                    normal,
                                });
                            }
                            HitType::Volume => {
                                match handle_volume(*node_leaf, rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                            HitType::Empty => {
                                match handle_empty(rt_state, ray.direction) {
                                    VolumeOutput::ContinueIteration(state) => rt_state = state,
                                    VolumeOutput::StopIteration {
                                        stop_position,
                                        hit_material,
                                    } => {
                                        return Some(OctTreeHitInfo::Volume {
                                            hit_value: hit_material,
                                            hit_position: stop_position,
                                        })
                                    }
                                };
                            }
                        }
                    } else {
                        return None;
                    }
                }
            }
        }
        warn!(
            "Max number of iterations reached, num_iterations: {}",
            MAX_NUMBER_RAY_ITERATIONS
        );
        None
    }

    fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<VoxelMaterial>> {
        struct PlaneIntersection {
            normal_axis: usize,
            intersect_time: RayScalar,
            normal_vector: Vector3<RayScalar>,
            intersect_position: Point3<RayScalar>,
            block_coordinate: Point3<i32>,
        }

        if ray.origin.x >= 0.0
            && ray.origin.x <= self.size as RayScalar
            && ray.origin.y >= 0.0
            && ray.origin.y <= self.size as RayScalar
            && ray.origin.z >= 0.0
            && ray.origin.z <= self.size as RayScalar
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
            )
        } else {
            let mut solutions = (0..3)
                .flat_map(|normal_axis| {
                    let zero_intersect_time = ray.intersect_axis(normal_axis, 0.);
                    let zero_intersect_position = ray.at(zero_intersect_time);
                    let size_intersect_time =
                        ray.intersect_axis(normal_axis, self.size as RayScalar);
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
                        && intersection.intersect_position[0] < self.size as RayScalar)
                        || intersection.normal_axis == 0)
                        && ((intersection.intersect_position[1] >= 0.
                            && intersection.intersect_position[1] < self.size as RayScalar)
                            || intersection.normal_axis == 1)
                        && ((intersection.intersect_position[2] >= 0.
                            && intersection.intersect_position[2] < self.size as RayScalar)
                            || intersection.normal_axis == 2)
                })
                .collect::<Vec<_>>();
            solutions.sort_by(|a, b| a.intersect_time.partial_cmp(&b.intersect_time).unwrap());
            if let Some(intersection) = solutions.first() {
                let block = self.get(intersection.block_coordinate.map(|v| v as u32).map(|v| v));
                match block.hit_type() {
                    HitType::Solid => Some(OctTreeHitInfo::Solid {
                        hit_position: intersection.intersect_position,
                        hit_value: block,
                        normal: intersection.normal_vector,
                    }),
                    HitType::Volume => self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                    ),
                    HitType::Empty => self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                    ),
                }
            } else {
                None
            }
        }
    }
}
