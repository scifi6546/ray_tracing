use super::{Leafable, OctTree, OctTreeChildren, OctTreeHitInfo, OctTreeNode};
use crate::prelude::{Ray, RayScalar};
use log::{error, warn};

use cgmath::{prelude::*, Point3, Vector3};

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
        const MAX_NUMBER_RAY_ITERATIONS: usize = 3000;
        let original_origin = ray.origin;

        block_coordinates = floor_point3_integer(
            block_coordinates,
            get_step_size(self, block_coordinates) as i64,
        );
        for _ in 0..MAX_NUMBER_RAY_ITERATIONS {
            let step_size = get_step_size(self, block_coordinates);

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
        } else {
            if let Some(intersection) = solutions.first() {
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
