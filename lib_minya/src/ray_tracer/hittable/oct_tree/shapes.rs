use super::{
    prelude::{get_children_offsets, get_next_power},
    LeafType, Leafable, OctTree, OctTreeChildren, OctTreeNode,
};
use cgmath::{Point3, Vector3};
use std::cmp::max;
impl<T: Leafable> OctTree<T> {
    /// returns an empty Oct Tree
    pub fn empty() -> Self {
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(LeafType::Empty),
                size: 1,
            },
            size: 1,
        }
    }
    pub fn cube(power_value: u32, hit_val: T) -> Self {
        let size = 2u32.pow(power_value);
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                size,
            },
            size,
        }
    }
    pub fn rectangle(rectangle_size: Vector3<u32>, hit_val: T) -> Self {
        let size = get_next_power(max(
            max(rectangle_size[0], rectangle_size[1]),
            rectangle_size[2],
        ));
        fn rectangle_recurse<T: Leafable>(
            rectangle_size: Vector3<u32>,
            hit_val: T,
            offset: Point3<u32>,
            size: u32,
        ) -> OctTreeNode<T> {
            if size > 1 {
                let top = Point3::new(offset[0] + size, offset[1] + size, offset[2] + size);
                if offset[0] < rectangle_size[0]
                    && offset[1] < rectangle_size[1]
                    && offset[2] < rectangle_size[2]
                {
                    if top[0] >= rectangle_size[0]
                        || top[1] >= rectangle_size[1]
                        || top[2] >= rectangle_size[2]
                    {
                        OctTreeNode {
                            children: OctTreeChildren::ParentNode(Box::new(
                                get_children_offsets().map(|child_offset| {
                                    rectangle_recurse(
                                        rectangle_size,
                                        hit_val,
                                        Point3::new(
                                            offset.x + child_offset.x * size / 2,
                                            offset.y + child_offset.y * size / 2,
                                            offset.z + child_offset.z * size / 2,
                                        ),
                                        size / 2,
                                    )
                                }),
                            )),
                            size,
                        }
                    } else {
                        OctTreeNode {
                            children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                            size,
                        }
                    }
                } else {
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size,
                    }
                }
            } else {
                let is_solid = offset[0] < rectangle_size[0]
                    && offset[1] < rectangle_size[1]
                    && offset[2] < rectangle_size[2];
                OctTreeNode {
                    children: OctTreeChildren::Leaf(if is_solid {
                        LeafType::Solid(hit_val)
                    } else {
                        LeafType::Empty
                    }),
                    size: 1,
                }
            }
        }
        Self {
            root_node: rectangle_recurse(rectangle_size, hit_val, Point3::new(0, 0, 0), size),
            size,
        }
    }

    /// Creates a sphere with the given radius
    pub fn sphere(radius: u32, hit_val: T) -> Self {
        #[allow(dead_code)]
        ///from https://web.archive.org/web/20100323053111/http://www.ics.uci.edu/%7Earvo/code/BoxSphereIntersect.c
        fn sphere_box_intersection(
            sphere_center: Vector3<f32>,
            sphere_radius: f32,
            box_min: Vector3<f32>,
            box_max: Vector3<f32>,
        ) -> bool {
            let r2 = sphere_radius.sqrt();
            let mut d_min = 0.0;
            for i in 0..3 {
                if sphere_center[i] < box_min[i] {
                    d_min += (sphere_center[i] - box_min[i]).sqrt()
                } else if sphere_center[i] > box_max[i] {
                    d_min += (sphere_center[i] - box_max[i]).sqrt()
                }
            }
            d_min <= r2
        }
        fn handle_leaf<T: Leafable>(
            size: u32,
            radius: u32,
            center: u32,
            corner: [u32; 3],
            hit_val: T,
        ) -> OctTreeNode<T> {
            fn distance(x: u32, y: u32, z: u32, center: u32) -> f32 {
                ((x as f32 - center as f32).powi(2)
                    + (y as f32 - center as f32).powi(2)
                    + (z as f32 - center as f32).powi(2))
                .sqrt()
            }
            if size >= 2 {
                let d0 = distance(corner[0], corner[1], corner[2], center) < radius as f32;
                let d1 =
                    distance(corner[0], corner[1], corner[2] + size - 1, center) < radius as f32;
                let d2 =
                    distance(corner[0], corner[1] + size - 1, corner[2], center) < radius as f32;
                let d3 = distance(
                    corner[0],
                    corner[1] + size - 1,
                    corner[2] + size - 1,
                    center,
                ) < radius as f32;
                let d4 =
                    distance(corner[0] + size - 1, corner[1], corner[2], center) < radius as f32;
                let d5 = distance(
                    corner[0] + size - 1,
                    corner[1],
                    corner[2] + size - 1,
                    center,
                ) < radius as f32;
                let d6 = distance(
                    corner[0] + size - 1,
                    corner[1] + size - 1,
                    corner[2],
                    center,
                ) < radius as f32;
                let d7 = distance(
                    corner[0] + size - 1,
                    corner[1] + size - 1,
                    corner[2] + size - 1,
                    center,
                ) < radius as f32;

                if d0 == d1 && d1 == d2 && d2 == d3 && d3 == d4 && d4 == d5 && d5 == d6 && d6 == d7
                {
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(if d0 {
                            LeafType::Solid(hit_val)
                        } else {
                            LeafType::Empty
                        }),
                        size,
                    }
                } else {
                    OctTreeNode {
                        children: OctTreeChildren::ParentNode(Box::new([
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1], corner[2]],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1], corner[2] + size / 2],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1] + size / 2, corner[2]],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1] + size / 2, corner[2] + size / 2],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1], corner[2]],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1], corner[2] + size / 2],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1] + size / 2, corner[2]],
                                hit_val,
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [
                                    corner[0] + size / 2,
                                    corner[1] + size / 2,
                                    corner[2] + size / 2,
                                ],
                                hit_val,
                            ),
                        ])),
                        size,
                    }
                }
            } else {
                OctTreeNode {
                    children: OctTreeChildren::Leaf(
                        if distance(corner[0], corner[1], corner[2], center) <= (radius as f32) {
                            LeafType::Solid(hit_val)
                        } else {
                            LeafType::Empty
                        },
                    ),
                    size: 1,
                }
            }
        }
        let size = 2 * get_next_power(radius);
        let center = size / 2;

        if size >= 2 {
            Self {
                root_node: OctTreeNode {
                    children: OctTreeChildren::ParentNode(Box::new([
                        handle_leaf::<T>(size / 2, radius, center, [0, 0, 0], hit_val),
                        handle_leaf::<T>(size / 2, radius, center, [0, 0, size / 2], hit_val),
                        handle_leaf::<T>(size / 2, radius, center, [0, size / 2, 0], hit_val),
                        handle_leaf(size / 2, radius, center, [0, size / 2, size / 2], hit_val),
                        handle_leaf(size / 2, radius, center, [size / 2, 0, 0], hit_val),
                        handle_leaf(size / 2, radius, center, [size / 2, 0, size / 2], hit_val),
                        handle_leaf(size / 2, radius, center, [size / 2, size / 2, 0], hit_val),
                        handle_leaf(
                            size / 2,
                            radius,
                            center,
                            [size / 2, size / 2, size / 2],
                            hit_val,
                        ),
                    ])),
                    size,
                },
                size,
            }
        } else {
            Self {
                root_node: OctTreeNode {
                    children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                    size,
                },
                size,
            }
        }
    }
}
