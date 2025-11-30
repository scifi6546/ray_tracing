use super::{Leafable, OctTree, OctTreeChildren, OctTreeHitInfo, OctTreeNode};
use crate::prelude::{Ray, RayScalar};
use log::{error, warn};
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
    fn ray_iteration(
        &self,
        mut block_coordinates: Point3<i64>,
        mut ray: Ray,
    ) -> Option<OctTreeHitInfo<T>> {
        let original_ray = ray;
        // returns a floored to the nearest multiple of b
        fn floor_value_scalar(a: RayScalar, b: RayScalar) -> i64 {
            (a as i64) - (a as i64) % (b as i64)
        }
        fn floor_value_integer(a: i64, b: i64) -> i64 {
            a - (a % b)
        }
        fn floor_point3_integer(coord: Point3<i64>, b: i64) -> Point3<i64> {
            coord.map(|v| floor_value_integer(v, b))
        }
        // gets the sign of the value, for example -123 -> -1 or 123 -> 1.
        fn int_sign(a: RayScalar) -> i64 {
            if a.is_sign_positive() {
                1
            } else {
                -1
            }
        }
        fn get_step_size<T: Leafable>(s: &OctTreeNode<T>, coordinates: Point3<i64>) -> u32 {
            s.get_homogenous_chunk(coordinates.map(|v| v as u32))
                .expect("Index is out of range")
                .size
        }
        const MAX_NUMBER_RAY_ITERATIONS: usize = 300;
        let original_origin = ray.origin;
        let mut step_size = get_step_size(self, block_coordinates);
        block_coordinates = floor_point3_integer(block_coordinates, step_size as i64);
        for _ in 0..MAX_NUMBER_RAY_ITERATIONS {
            step_size = get_step_size(self, block_coordinates);

            if self.in_range(block_coordinates.map(|v| v as i32)) == false {
                return None;
            }
            let mut t = 0.;
            if ray.direction.x.is_sign_positive() {
                t = 1.;
            }
            let t_x = (block_coordinates.x as RayScalar + step_size as RayScalar * t as RayScalar
                - ray.origin.x)
                / ray.direction.x;
            let mut t = 0.;

            if ray.direction.y.is_sign_positive() {
                t = 1.;
            }
            let t_y = (block_coordinates.y as RayScalar + step_size as RayScalar * t as RayScalar
                - ray.origin.y)
                / ray.direction.y;

            let mut t = 0.;

            if ray.direction.z.is_sign_positive() {
                t = 1.;
            }
            let t_z = (block_coordinates.z as RayScalar + step_size as RayScalar * t as RayScalar
                - ray.origin.z)
                / ray.direction.z;
            if t_x < 0. {
                error!("t_x < 0., t_x = {}", t_x);
                error!(
                    "ray origin: <{},{},{}>",
                    ray.origin.x, ray.origin.y, ray.origin.z
                );
                error!(
                    "ray direction: <{}, {}, {}>",
                    ray.direction.x, ray.direction.y, ray.direction.z
                );
                error!("step size: {}", step_size);
                error!(
                    "block coordiates: <{},{},{}>",
                    block_coordinates.x, block_coordinates.y, block_coordinates.z
                );
                error!(
                    "position: <{}, {}, {}>",
                    ray.origin.x, ray.origin.y, ray.origin.z
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
                ray.origin.y = ray.origin.y + t_x * ray.direction.y;
                ray.origin.z = ray.origin.z + t_x * ray.direction.z;
                if ray.direction.x >= 0. {
                    block_coordinates.x =
                        block_coordinates.x + step_size as i64 * int_sign(ray.direction.x);
                    ray.origin.x = ray.origin.x + t_x * ray.direction.x;
                    if self.in_range(Point3::new(
                        block_coordinates.x as i32,
                        ray.origin.y as i32,
                        ray.origin.z as i32,
                    )) {
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                block_coordinates.x,
                                ray.origin.y as i64,
                                ray.origin.z as i64,
                            ),
                        ) as i64;
                        block_coordinates.y =
                            floor_value_integer(ray.origin.y as i64, next_step_size);
                        block_coordinates.z =
                            floor_value_integer(ray.origin.z as i64, next_step_size);

                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(-1., 0., 0.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
                        }
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        block_coordinates.x as i32 - 1,
                        block_coordinates.y as i32,
                        block_coordinates.z as i32,
                    )) {
                        ray.origin.x = ray.origin.x + t_x * ray.direction.x;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                block_coordinates.x - 1,
                                ray.origin.y as i64,
                                ray.origin.z as i64,
                            ),
                        );

                        block_coordinates.y =
                            floor_value_integer(ray.origin.y as i64, next_step_size as i64);
                        block_coordinates.z =
                            floor_value_integer(ray.origin.z as i64, next_step_size as i64);
                        block_coordinates.x = block_coordinates.x - next_step_size as i64;
                        step_size = next_step_size;

                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(0., 0., 1.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
                        }
                    } else {
                        return None;
                    }
                }
            } else if t_y < t_x && t_y < t_z {
                // y is the min
                ray.origin.x = ray.origin.x + t_y * ray.direction.x;
                ray.origin.z = ray.origin.z + t_y * ray.direction.z;
                if ray.direction.y >= 0. {
                    block_coordinates.y =
                        block_coordinates.y + step_size as i64 * int_sign(ray.direction.y);
                    ray.origin.y = ray.origin.y + t_y * ray.direction.y;
                    if self.in_range(Point3::new(
                        ray.origin.x as i32,
                        block_coordinates.y as i32,
                        ray.origin.z as i32,
                    )) {
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                ray.origin.x as i64,
                                block_coordinates.y,
                                ray.origin.z as i64,
                            ),
                        ) as i64;
                        block_coordinates.x =
                            floor_value_integer(ray.origin.x as i64, next_step_size);
                        block_coordinates.z =
                            floor_value_integer(ray.origin.z as i64, next_step_size);

                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(0., -1., 0.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
                        }
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        block_coordinates.x as i32,
                        block_coordinates.y as i32 - 1,
                        block_coordinates.z as i32,
                    )) {
                        ray.origin.y = ray.origin.y + t_y * ray.direction.y;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                ray.origin.x as i64,
                                block_coordinates.y as i64 - 1,
                                ray.origin.z as i64,
                            ),
                        );
                        block_coordinates.x =
                            floor_value_integer(ray.origin.x as i64, next_step_size as i64);
                        block_coordinates.z =
                            floor_value_integer(ray.origin.z as i64, next_step_size as i64);
                        block_coordinates.y = block_coordinates.y - next_step_size as i64;
                        step_size = next_step_size;
                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(0., 1., 0.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
                        }
                    } else {
                        return None;
                    }
                }
            } else {
                // z is the min
                ray.origin.y = ray.origin.y + t_z * ray.direction.y;
                ray.origin.x = ray.origin.x + t_z * ray.direction.x;
                if ray.direction.z >= 0. {
                    block_coordinates.z =
                        block_coordinates.z + step_size as i64 * int_sign(ray.direction.z);
                    ray.origin.z = ray.origin.z + t_z * ray.direction.z;
                    if self.in_range(Point3::new(
                        ray.origin.x as i32,
                        ray.origin.y as i32,
                        block_coordinates.z as i32,
                    )) {
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                ray.origin.x as i64,
                                ray.origin.y as i64,
                                block_coordinates.z as i64,
                            ),
                        ) as i64;
                        block_coordinates.y =
                            floor_value_integer(ray.origin.y as i64, next_step_size);
                        block_coordinates.x =
                            floor_value_integer(ray.origin.x as i64, next_step_size);
                        step_size = next_step_size as u32;
                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(0., 0., -1.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
                        }
                    } else {
                        return None;
                    }
                } else {
                    if self.in_range(Point3::new(
                        block_coordinates.x as i32,
                        block_coordinates.y as i32,
                        block_coordinates.z as i32 - 1,
                    )) {
                        ray.origin.z = ray.origin.z + t_z * ray.direction.z;
                        let next_step_size = get_step_size(
                            self,
                            Point3::new(
                                ray.origin.x as i64,
                                ray.origin.y as i64,
                                block_coordinates.z as i64 - 1,
                            ),
                        );
                        block_coordinates.x =
                            floor_value_integer(ray.origin.x as i64, next_step_size as i64);
                        block_coordinates.y =
                            floor_value_integer(ray.origin.y as i64, next_step_size as i64);

                        block_coordinates.z = block_coordinates.z - next_step_size as i64;

                        step_size = next_step_size;
                        let node = self
                            .get_chunk(block_coordinates.map(|v| v as u32))
                            .expect("should be in range");
                        let node_leaf = match &node.children {
                            OctTreeChildren::Leaf(v) => v,
                            OctTreeChildren::ParentNode(_) => panic!("should not be a parent node"),
                        };
                        if let Some(hit_value) = node_leaf.try_solid() {
                            let normal = Vector3::new(0., 0., 1.);

                            return Some(OctTreeHitInfo {
                                hit_value,
                                depth: ray.origin.distance(original_origin),
                                hit_position: ray.origin,
                                normal,
                            });
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
    ///traces, assumes that ray is either on boundry or on border
    fn trace(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        /// does the eqalivalent of fract but for different bases
        fn base_fract(a: RayScalar, b: RayScalar) -> RayScalar {
            (a / b).fract() * b
        }
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
            //next_dist.x =
            //    -1.0 * (ray.origin.x.rem_euclid(move_size as RayScalar)) / ray.direction.x;
            next_dist.x =
                -1.0 * (base_fract(ray.origin.x, move_size as RayScalar)) / ray.direction.x;
        } else {
            step_dir.x = 1.0;
            //next_dist.x = (1.0 - ray.origin.x.rem_euclid(move_size as RayScalar)) / ray.direction.x;

            next_dist.x =
                (1.0 - base_fract(ray.origin.x, move_size as RayScalar)) / ray.direction.x;
        }

        if ray.direction.y < 0.0 {
            step_dir.y = -1.0;
            //next_dist.y = (ray.origin.y.rem_euclid(move_size as RayScalar).neg()) / ray.direction.y;

            next_dist.y =
                -1.0 * (base_fract(ray.origin.y, move_size as RayScalar)) / ray.direction.y;
        } else {
            step_dir.y = 1.0;
            //next_dist.y = (1.0 - ray.origin.y.rem_euclid(move_size as RayScalar)) / ray.direction.y;
            next_dist.y =
                (1.0 - base_fract(ray.origin.y, move_size as RayScalar)) / ray.direction.y;
        }
        if ray.direction.z < 0.0 {
            step_dir.z = -1.0;
            //next_dist.z = (ray.origin.z.rem_euclid(move_size as RayScalar).neg()) / ray.direction.z;
            next_dist.z =
                -1.0 * (base_fract(ray.origin.z, move_size as RayScalar)) / ray.direction.z;
        } else {
            step_dir.z = 1.0;
            //next_dist.z = (1.0 - ray.origin.z.rem_euclid(move_size as RayScalar)) / ray.direction.z;
            next_dist.z =
                (1.0 - base_fract(ray.origin.z, move_size as RayScalar)) / ray.direction.z;
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
                current_pos += ray.direction * next_dist.x * move_size as RayScalar;
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
                current_pos += ray.direction * next_dist.y * move_size as RayScalar;
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
                current_pos += ray.direction * next_dist.z * move_size as RayScalar;
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
        struct PlaneIntersection {
            normal_axis: usize,
            intersect_time: RayScalar,
            normal_vector: Vector3<RayScalar>,
            intersect_position: Point3<RayScalar>,
            block_coordinate: Point3<i64>,
        }
        let mut solutions = (0..3)
            .flat_map(|normal_axis| {
                let zero_intersect_time = ray.intersect_axis(normal_axis, 0.);
                let zero_intersect_position = ray.at(zero_intersect_time);
                let size_intersect_time = ray.intersect_axis(normal_axis, self.size as RayScalar);
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
                            zero_intersect_position.x as i64,
                            zero_intersect_position.y as i64,
                            zero_intersect_position.z as i64,
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
                                size_intersect_position.x as i64 - 1
                            } else {
                                size_intersect_position.x as i64
                            },
                            if normal_axis == 1 {
                                size_intersect_position.y as i64 - 1
                            } else {
                                size_intersect_position.y as i64
                            },
                            if normal_axis == 2 {
                                size_intersect_position.z as i64 - 1
                            } else {
                                size_intersect_position.z as i64
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
        if ray.origin.x >= 0.0
            && ray.origin.x <= self.size as RayScalar
            && ray.origin.y >= 0.0
            && ray.origin.y <= self.size as RayScalar
            && ray.origin.z >= 0.0
            && ray.origin.z <= self.size as RayScalar
        {
            self.ray_iteration(
                Point3::new(
                    ray.origin.x as i64,
                    ray.origin.y as i64,
                    ray.origin.z as i64,
                ),
                Ray {
                    origin: ray.origin,
                    direction: ray.direction,
                    time: ray.time,
                },
            )
            //self.trace(ray)
        } else {
            if let Some(intersection) = solutions.first() {
                /*
                if let Some(solid) = self
                    .get(intersection.intersect_position.map(|v| v as u32).map(|v| {
                        if v < self.size {
                            v
                        } else {
                            self.size - 1
                        }
                    }))
                    .try_solid()
                {
                 */
                if let Some(solid) = self
                    .get(intersection.block_coordinate.map(|v| v as u32).map(|v| v))
                    .try_solid()
                {
                    Some(OctTreeHitInfo {
                        depth: ray.distance(Vector3::new(
                            intersection.intersect_position.x,
                            intersection.intersect_position.y,
                            intersection.intersect_position.z,
                        )),
                        hit_position: intersection.intersect_position,
                        hit_value: solid,
                        normal: intersection.normal_vector,
                    })
                } else {
                    self.ray_iteration(
                        intersection.block_coordinate,
                        Ray {
                            origin: intersection.intersect_position,
                            direction: ray.direction,
                            time: 0.,
                        },
                    )
                }
            } else {
                None
            }
        }
    }
}
