use super::{
    super::material::Material, super::pdf::LambertianPDF, HitRay, HitRecord, Hittable, Ray,
    RayAreaInfo,
};
use crate::ray_tracer::bvh::Aabb;

use crate::ray_tracer::pdf::ScatterRecord;

use crate::ray_tracer::hittable::oct_tree::prelude::get_children_offsets;
use base_lib::RgbColor;
use cgmath::num_traits::Signed;
use cgmath::{num_traits::FloatConst, prelude::*, InnerSpace, Point2, Point3, Vector3};
use image::{Rgb, RgbImage};
use log::error;
use std::{cmp::max, rc::Rc};

fn f32_min(a: f32, b: f32) -> f32 {
    if a <= b {
        a
    } else {
        b
    }
}
mod prelude {
    use super::Leafable;
    use cgmath::Point3;
    // from https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_aabb.html
    pub fn aabb_intersect(
        a_min: [i32; 3],
        a_max: [i32; 3],
        b_min: [i32; 3],
        b_max: [i32; 3],
    ) -> bool {
        (a_min[0] <= b_max[0] && a_max[0] >= b_min[0])
            && (a_min[1] <= b_max[1] && a_max[1] >= b_min[1])
            && (a_min[2] <= b_max[2] && a_max[2] >= b_min[2])
    }
    #[derive(Copy, Clone, PartialEq, Eq)]
    pub struct RgbColor(pub [u8; 3]);
    impl Leafable for RgbColor {}
    pub fn get_children_offsets() -> [Point3<u32>; 8] {
        [
            Point3::new(0, 0, 0),
            Point3::new(0, 0, 1),
            Point3::new(0, 1, 0),
            Point3::new(0, 1, 1),
            Point3::new(1, 0, 0),
            Point3::new(1, 0, 1),
            Point3::new(1, 1, 0),
            Point3::new(1, 1, 1),
        ]
    }
}
#[derive(Debug)]
pub struct OctTreeHitInfo<T: Leafable> {
    pub hit_value: T,
    pub depth: f32,
    pub hit_position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub hit_positions: Vec<(u32, Vector3<u32>)>,
}
#[derive(Clone, Debug)]
pub struct OctTree<T: Leafable> {
    root_node: OctTreeNode<T>,
    size: u32,
    material: VoxelMaterial,
}
impl<T: Leafable> OctTree<T> {
    pub fn cube(power_value: u32, hit_val: T) -> Self {
        let size = 2u32.pow(power_value);
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                size,
            },
            size,
            material: VoxelMaterial {},
        }
    }
    pub fn rectangle(rectangle_size: Vector3<u32>, hit_val: T) -> Self {
        let size = Self::get_next_power(max(
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
            material: VoxelMaterial {},
        }
    }

    /// Gets the next powr of 2 that is closest to the given value, if the value is already a power of 2 it returns the same value
    /// using https://stackoverflow.com/questions/1322510/given-an-integer-how-do-i-find-the-next-largest-power-of-two-using-bit-twiddlin/1322548#1322548
    fn get_next_power(v: u32) -> u32 {
        let mut v1 = v - 1;
        v1 |= v1 >> 1;
        v1 |= v1 >> 2;
        v1 |= v1 >> 4;
        v1 |= v1 >> 8;
        v1 |= v1 >> 16;
        v1 + 1
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
        let size = 2 * Self::get_next_power(radius);
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
                material: VoxelMaterial {},
            }
        } else {
            Self {
                root_node: OctTreeNode {
                    children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                    size,
                },
                size,
                material: VoxelMaterial {},
            }
        }
    }
    pub fn half_cube(hit_val: T) -> Self {
        let size = 32;
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::ParentNode(Box::new([
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size: size / 2,
                    },
                ])),
                size,
            },
            size,
            material: VoxelMaterial {},
        }
    }
    /// for debug purposes
    pub(crate) fn suboptimal_cube(hit_val: T) -> Self {
        let size = 2u32.pow(4);
        let children = OctTreeNode {
            children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
            size: size / 2,
        };
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::ParentNode(Box::new([
                    children.clone(),
                    children.clone(),
                    children.clone(),
                    children.clone(),
                    children.clone(),
                    children.clone(),
                    children.clone(),
                    children.clone(),
                ])),
                size,
            },
            size,
            material: VoxelMaterial {},
        }
    }
    pub fn combine(self, other: &Self, offset: Point3<i32>) -> Self {
        let offset = [offset.x, offset.y, offset.z];
        let other_size = offset
            .iter()
            .map(|s| s.abs() as u32 + other.size)
            .max()
            .unwrap();

        let size = Self::get_next_power(max(self.size, other_size));
        if size > self.size {
            self.combine_resize(other, offset)
        } else {
            self.combine_no_resize(other, offset)
        }
    }
    pub fn is_optimal(&self) -> bool {
        self.root_node.is_optimal()
    }
    // gets offsets of children

    fn combine_no_resize(self, other: &Self, offset: [i32; 3]) -> Self {
        /// checks if AABB a is fully inside b
        fn a_fully_in_b(
            a_min: [i32; 3],
            a_max: [i32; 3],
            b_min: [i32; 3],
            b_max: [i32; 3],
        ) -> bool {
            (a_min[0] >= b_min[0] && a_min[1] >= b_min[1] && a_min[2] >= b_min[2])
                && (a_max[0] <= b_max[0] && a_max[1] <= b_max[1] && a_max[2] <= b_max[2])
        }
        /// takes in children of a node and if possible simplifies the node
        fn try_simplify<T: Leafable>(nodes: [OctTreeNode<T>; 8]) -> OctTreeChildren<T> {
            let first_value = match &nodes[0].children {
                OctTreeChildren::Leaf(leaf_value) => leaf_value.clone(),
                OctTreeChildren::ParentNode(_) => {
                    return OctTreeChildren::ParentNode(Box::new(nodes))
                }
            };
            for node in nodes.iter().skip(1) {
                match node.children {
                    OctTreeChildren::ParentNode(_) => {
                        return OctTreeChildren::ParentNode(Box::new(nodes))
                    }
                    OctTreeChildren::Leaf(leaf_value) => {
                        if leaf_value != first_value {
                            return OctTreeChildren::ParentNode(Box::new(nodes));
                        }
                    }
                }
            }
            OctTreeChildren::Leaf(first_value)
        }
        fn modify_node<T: Leafable>(
            node: OctTreeNode<T>,
            node_offset: [i32; 3],
            other: &OctTree<T>,
            other_offset: [i32; 3],
        ) -> OctTreeNode<T> {
            assert!(node.size >= 1);
            // building aabb for checking if current selection colides
            let other_min = other_offset;
            let other_max = [
                other_min[0] + other.size as i32 - 1,
                other_min[1] + other.size as i32 - 1,
                other_min[2] + other.size as i32 - 1,
            ];
            let node_min = node_offset;
            let node_max = [
                node_min[0] + node.size as i32 - 1,
                node_min[1] + node.size as i32 - 1,
                node_min[2] + node.size as i32 - 1,
            ];
            if prelude::aabb_intersect(other_min, other_max, node_min, node_max) {
                match node.children {
                    OctTreeChildren::Leaf(leaf_value) => {
                        if a_fully_in_b(node_min, node_max, other_min, other_max) {
                            let start = [
                                node_offset[0] - other_offset[0],
                                node_offset[1] - other_offset[1],
                                node_offset[2] - other_offset[2],
                            ];
                            let end = [
                                node_offset[0] - other_offset[0] + node.size as i32,
                                node_offset[1] - other_offset[1] + node.size as i32,
                                node_offset[2] - other_offset[2] + node.size as i32,
                            ];

                            let mut val = leaf_value;
                            if node.size == 1 {
                                let other_child = other.get_contents(
                                    start[0] as u32,
                                    start[1] as u32,
                                    start[2] as u32,
                                );
                                let child_leaf = if leaf_value.is_solid() {
                                    leaf_value
                                } else if other_child.is_solid() {
                                    other_child
                                } else {
                                    LeafType::Empty
                                };
                                return OctTreeNode {
                                    children: OctTreeChildren::Leaf(child_leaf),
                                    size: 1,
                                };
                            }
                            // optimization idea: get chunks rather than individual nodes
                            for x in start[0]..end[0] {
                                for y in start[1]..end[1] {
                                    for z in start[2]..end[2] {
                                        let get_val =
                                            other.get_contents(x as u32, y as u32, z as u32);
                                        if get_val != val && node.size >= 2 {
                                            let offsets = prelude::get_children_offsets();
                                            let children = offsets.map(|offset| {
                                                modify_node(
                                                    OctTreeNode {
                                                        children: OctTreeChildren::Leaf(leaf_value),
                                                        size: node.size / 2,
                                                    },
                                                    [
                                                        node_offset[0]
                                                            + offset[0] as i32 * node.size as i32
                                                                / 2,
                                                        node_offset[1]
                                                            + offset[1] as i32 * node.size as i32
                                                                / 2,
                                                        node_offset[2]
                                                            + offset[2] as i32 * node.size as i32
                                                                / 2,
                                                    ],
                                                    other,
                                                    other_offset,
                                                )
                                            });
                                            let mut val: Option<LeafType<T>> = None;
                                            for (i, child) in children.iter().enumerate() {
                                                if i == 0 {
                                                    match &child.children {
                                                        OctTreeChildren::Leaf(v) => val = Some(*v),
                                                        OctTreeChildren::ParentNode(_) => {
                                                            return OctTreeNode {
                                                                children:
                                                                    OctTreeChildren::ParentNode(
                                                                        Box::new(children),
                                                                    ),
                                                                size: node.size,
                                                            }
                                                        }
                                                    }
                                                } else {
                                                    match &child.children {
                                                        OctTreeChildren::Leaf(v) => {
                                                            if Some(*v) != val {
                                                                return OctTreeNode {
                                                                    children:
                                                                        OctTreeChildren::ParentNode(
                                                                            Box::new(children),
                                                                        ),
                                                                    size: node.size,
                                                                };
                                                            }
                                                        }
                                                        OctTreeChildren::ParentNode(_) => {
                                                            return OctTreeNode {
                                                                children:
                                                                    OctTreeChildren::ParentNode(
                                                                        Box::new(children),
                                                                    ),
                                                                size: node.size,
                                                            };
                                                        }
                                                    }
                                                }
                                            }
                                            return OctTreeNode {
                                                children: OctTreeChildren::Leaf(val.unwrap()),
                                                size: node.size,
                                            };
                                        } else {
                                            val = if val.is_solid() {
                                                val
                                            } else if get_val.is_solid() {
                                                get_val
                                            } else {
                                                LeafType::Empty
                                            };
                                        }
                                    }
                                }
                            }
                            OctTreeNode {
                                children: OctTreeChildren::Leaf(val),
                                size: node.size,
                            }
                        } else {
                            let offsets = prelude::get_children_offsets();
                            let mut nodes = [
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(LeafType::Empty),
                                    size: 0,
                                },
                            ];
                            let mut val = Some(leaf_value);

                            for (i, offset) in offsets.iter().enumerate() {
                                let node = modify_node(
                                    OctTreeNode {
                                        children: OctTreeChildren::Leaf(leaf_value),
                                        size: node.size / 2,
                                    },
                                    [
                                        node_offset[0] + offset[0] as i32 * node.size as i32 / 2,
                                        node_offset[1] + offset[1] as i32 * node.size as i32 / 2,
                                        node_offset[2] + offset[2] as i32 * node.size as i32 / 2,
                                    ],
                                    other,
                                    other_offset,
                                );
                                if val.is_some() {
                                    match &node.children {
                                        OctTreeChildren::Leaf(child_val) => {
                                            if Some(*child_val) != val {
                                                val = None
                                            }
                                        }
                                        OctTreeChildren::ParentNode(_v) => val = None,
                                    };
                                }
                                nodes[i] = node;
                            }
                            if let Some(val) = val {
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(val),
                                    size: node.size,
                                }
                            } else {
                                OctTreeNode {
                                    children: OctTreeChildren::ParentNode(Box::new(nodes)),
                                    size: node.size,
                                }
                            }
                        }
                    }
                    OctTreeChildren::ParentNode(children) => {
                        let offsets = prelude::get_children_offsets();
                        let mut i = 0;
                        let children = offsets.map(|offset| {
                            let out = modify_node(
                                children[i].clone(),
                                [
                                    (offset[0] * node.size / 2) as i32 + node_offset[0],
                                    (offset[1] * node.size / 2) as i32 + node_offset[1],
                                    (offset[2] * node.size / 2) as i32 + node_offset[2],
                                ],
                                other,
                                other_offset,
                            );
                            i += 1;
                            out
                        });
                        OctTreeNode {
                            children: try_simplify(children),
                            size: node.size,
                        }
                    }
                }
            } else {
                //nothing needs to be done as no intersection
                node
            }
        }

        Self {
            root_node: modify_node(self.root_node, [0, 0, 0], other, offset),
            size: self.size,
            material: VoxelMaterial {},
        }
    }
    fn combine_resize(self, other: &Self, offset: [i32; 3]) -> Self {
        fn build_nodes<T: Leafable>(
            size: u32,
            a: &OctTree<T>,
            b: &OctTree<T>,
            b_offset: [i32; 3],
            // lower left corner of current cube
            cube_position: [u32; 3],
        ) -> OctTreeNode<T> {
            let cube_position_i32 = cube_position.map(|d| d as i32);
            let current_max = cube_position.map(|d| d as i32 + size as i32 - 1);
            let a_intersects = prelude::aabb_intersect(
                cube_position_i32,
                current_max,
                [0, 0, 0],
                [
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                ],
            );
            let b_intersects = prelude::aabb_intersect(
                cube_position_i32,
                current_max,
                b_offset,
                b_offset.map(|p| p + b.size as i32 - 1),
            );
            if a_intersects || b_intersects {
                if size >= 2 {
                    let x0 = cube_position[0];
                    let y0 = cube_position[1];
                    let z0 = cube_position[2];

                    let x1 = x0 + size / 2;
                    let y1 = y0 + size / 2;
                    let z1 = z0 + size / 2;

                    let cubes = [
                        build_nodes(size / 2, a, b, b_offset, [x0, y0, z0]),
                        build_nodes(size / 2, a, b, b_offset, [x0, y0, z1]),
                        build_nodes(size / 2, a, b, b_offset, [x0, y1, z0]),
                        build_nodes(size / 2, a, b, b_offset, [x0, y1, z1]),
                        // top x
                        build_nodes(size / 2, a, b, b_offset, [x1, y0, z0]),
                        build_nodes(size / 2, a, b, b_offset, [x1, y0, z1]),
                        build_nodes(size / 2, a, b, b_offset, [x1, y1, z0]),
                        build_nodes(size / 2, a, b, b_offset, [x1, y1, z1]),
                    ];

                    let mut same = true;
                    let mut cube_val: Option<LeafType<T>> = None;
                    for cube in cubes.iter() {
                        match cube.children {
                            OctTreeChildren::Leaf(val) => {
                                if let Some(cube) = cube_val {
                                    if val != cube {
                                        same = false;
                                        break;
                                    }
                                } else {
                                    cube_val = Some(val);
                                }
                            }
                            OctTreeChildren::ParentNode(_) => {
                                same = false;

                                break;
                            }
                        }
                    }
                    if same {
                        OctTreeNode {
                            children: OctTreeChildren::Leaf(cube_val.unwrap()),
                            size,
                        }
                    } else {
                        OctTreeNode {
                            children: OctTreeChildren::ParentNode(Box::new(cubes)),
                            size,
                        }
                    }
                } else {
                    let a_val = if cube_position[0] < a.size
                        && cube_position[1] < a.size
                        && cube_position[2] < a.size
                    {
                        a.get_contents(cube_position[0], cube_position[1], cube_position[2])
                    } else {
                        LeafType::Empty
                    };
                    let b_pos = [
                        cube_position[0] as i32 - b_offset[0],
                        cube_position[1] as i32 - b_offset[1],
                        cube_position[2] as i32 - b_offset[2],
                    ];
                    let b_val = if b_pos[0] >= 0
                        && b_pos[0] < b.size as i32
                        && b_pos[1] >= 0
                        && b_pos[1] < b.size as i32
                        && b_pos[2] >= 0
                        && b_pos[2] < b.size as i32
                    {
                        b.get_contents(b_pos[0] as u32, b_pos[1] as u32, b_pos[2] as u32)
                    } else {
                        LeafType::Empty
                    };

                    OctTreeNode {
                        children: OctTreeChildren::Leaf(if a_val.is_solid() {
                            a_val
                        } else if b_val.is_solid() {
                            b_val
                        } else {
                            LeafType::Empty
                        }),
                        size,
                    }
                }
            } else {
                OctTreeNode {
                    children: OctTreeChildren::Leaf(LeafType::Empty),
                    size,
                }
            }
        }
        let other_size = offset
            .iter()
            .map(|s| s.abs() as u32 + other.size)
            .max()
            .unwrap();

        let size = Self::get_next_power(max(self.size, other_size));

        Self {
            root_node: build_nodes(size, &self, other, offset, [0, 0, 0]),
            size,
            material: VoxelMaterial {},
        }
    }
    fn get_contents(&self, x: u32, y: u32, z: u32) -> LeafType<T> {
        self.root_node.get(x, y, z)
    }

    pub fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        self.root_node.trace_ray(ray)
    }
}

#[derive(Clone, Debug)]
struct OctTreeNode<T: Leafable> {
    children: OctTreeChildren<T>,
    size: u32,
}
impl<T: Leafable> OctTreeNode<T> {
    pub fn is_optimal(&self) -> bool {
        match &self.children {
            OctTreeChildren::Leaf(_) => true,
            OctTreeChildren::ParentNode(children) => {
                let mut val = match &children[0].children {
                    OctTreeChildren::Leaf(val) => Some(val),
                    OctTreeChildren::ParentNode(_) => None,
                };
                if val.is_some() {
                    for i in 1..8 {
                        match &children[i].children {
                            OctTreeChildren::Leaf(val2) => {
                                if Some(val2) != val {
                                    val = None;
                                    break;
                                }
                            }
                            OctTreeChildren::ParentNode(_) => {
                                val = None;
                                break;
                            }
                        }
                    }
                }
                if val.is_some() {
                    false
                } else {
                    children
                        .iter()
                        .map(|c| c.is_optimal())
                        .fold(true, |acc, x| acc && x)
                }
            }
        }
    }
    /// returns ray in distance it hit
    pub fn trace_ray(&self, ray: Ray) -> Option<OctTreeHitInfo<T>> {
        // getting the min distances

        match &self.children {
            OctTreeChildren::Leaf(val) => {
                if val.is_solid() {
                    let t = vec![(0, Vector3::new(0u32, 0, 0))];
                    let (axis, time, normal) = (0..3)
                        .map(|axis| {
                            if ray.direction[axis] >= 0.0 {
                                (
                                    axis,
                                    ray.intersect_axis(axis, 0.0),
                                    Vector3::new(
                                        if axis == 0 { -1.0f32 } else { 0.0 },
                                        if axis == 1 { -1.0 } else { 0.0 },
                                        if axis == 2 { -1.0 } else { 0.0 },
                                    ),
                                )
                            } else {
                                (
                                    axis,
                                    ray.intersect_axis(axis, self.size as f32),
                                    Vector3::new(
                                        if axis == 0 { 1.0 } else { 0.0 },
                                        if axis == 1 { 1.0 } else { 0.0 },
                                        if axis == 2 { 1.0 } else { 0.0 },
                                    ),
                                )
                            }
                        })
                        .filter(|(_idx, t, _normal)| *t >= 0.)
                        .filter(|(idx, time, _normal)| {
                            let pos = ray.local_at(*time);
                            let pos_good = [
                                *idx == 0 || (pos[0] >= 0. && pos[0] <= self.size as f32),
                                *idx == 1 || (pos[1] >= 0. && pos[1] <= self.size as f32),
                                *idx == 2 || (pos[2] >= 0. && pos[2] <= self.size as f32),
                            ];
                            pos_good[0] && pos_good[1] && pos_good[2]
                        })
                        .filter(|(_idx, time, normal)| {
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

                        Some((d, val.unwrap(), pos, normal));
                        Some(OctTreeHitInfo {
                            depth: d,
                            hit_value: val.unwrap(),
                            hit_position: Point3::new(pos.x, pos.y, pos.z),
                            normal,
                            hit_positions: vec![(self.size, Vector3::new(0, 0, 0))],
                        })
                    } else {
                        None
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
                        let is_valid = pos.map(|v| v >= 0. && v < self.size as f32);

                        (is_valid[0] || *idx == 0)
                            && (is_valid[1] || *idx == 1)
                            && (is_valid[2] || *idx == 2)
                    })
                    .filter_map(|(index, _dist, pos, idx_pos)| {
                        let floored_pos = pos.map(|v| (v / (self.size / 2) as f32).floor() as u32);

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
                    let tile_pos_floored = Vector3::new(
                        tile_index.x as f32 * (self.size / 2) as f32,
                        tile_index.y as f32 * (self.size / 2) as f32,
                        tile_index.z as f32 * (self.size / 2) as f32,
                    );
                    if let Some(mut hit_info) = children[index].trace_ray(Ray {
                        direction: ray.direction,
                        origin: Point3::new(
                            pos.x - tile_pos_floored.x,
                            pos.y - tile_pos_floored.y,
                            pos.z - tile_pos_floored.z,
                        ),
                        time: ray.time,
                    }) {
                        let hit_position = hit_info.hit_position + tile_pos_floored;
                        let mut hit_positions = vec![(self.size / 2, (self.size / 2) * tile_index)];
                        hit_positions.append(&mut hit_info.hit_positions);
                        return Some(OctTreeHitInfo {
                            depth: distance(
                                Vector3::new(ray.origin.x, ray.origin.y, ray.origin.z),
                                Vector3::new(hit_position.x, hit_position.y, hit_position.z),
                            ),
                            hit_value: hit_info.hit_value,
                            hit_position,
                            normal: hit_info.normal,
                            hit_positions,
                        });
                    }
                }
                None
            }
        }
    }
    pub fn get_child_index(&self, x: u32, y: u32, z: u32) -> usize {
        let x_v = x / (self.size / 2);
        let y_v = y / (self.size / 2);
        let z_v = z / (self.size / 2);
        Self::get_child_index_size2(x_v, y_v, z_v)
    }
    /// gets the size given self size is 2
    pub fn get_child_index_size2(x: u32, y: u32, z: u32) -> usize {
        assert!(x < 2);
        assert!(y < 2);
        assert!(z < 2);
        x as usize * 4 + y as usize * 2 + z as usize
    }
    pub fn get(&self, x: u32, y: u32, z: u32) -> LeafType<T> {
        match &self.children {
            OctTreeChildren::Leaf(val) => *val,
            OctTreeChildren::ParentNode(children) => {
                let idx = self.get_child_index(x, y, z);
                if idx >= children.len() {
                    println!("idx: {}, x: {}, y: {}, z: {}", idx, x, y, z);
                }
                //
                children[idx].get(
                    x % (self.size / 2),
                    y % (self.size / 2),
                    z % (self.size / 2),
                )
            }
        }
    }
}

#[derive(Clone, Debug)]
enum OctTreeChildren<T: Leafable> {
    Leaf(LeafType<T>),
    ParentNode(Box<[OctTreeNode<T>; 8]>),
}
/// Leaf of tree
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LeafType<T: Leafable> {
    /// Leaf has something in it
    Solid(T),
    /// leaf is empty
    Empty,
}
impl<T: Leafable> LeafType<T> {
    fn is_solid(&self) -> bool {
        match self {
            Self::Solid(_) => true,
            Self::Empty => false,
        }
    }
    fn unwrap(&self) -> T {
        match self {
            Self::Solid(T) => *T,
            Self::Empty => panic!("leaf empty"),
        }
    }
}
pub trait Leafable: Clone + Copy + PartialEq + Eq {}
impl Leafable for bool {}
impl Leafable for () {}

fn distance(a: Vector3<f32>, b: Vector3<f32>) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}
impl Ray {
    pub fn intersect_axis(&self, axis: usize, at: f32) -> f32 {
        (at - self.origin[axis]) / self.direction[axis]
    }
    /// gets the time at which the item intersects the x plane
    pub fn intersect_x(&self, at: f32) -> f32 {
        self.intersect_axis(0, at)
    }
    /// gets the time at which the item intersects the y plane
    pub fn intersect_y(&self, at: f32) -> f32 {
        self.intersect_axis(1, at)
    }
    /// gets the time at which the item intersects the z plane
    pub fn intersect_z(&self, at: f32) -> f32 {
        self.intersect_axis(2, at)
    }
    pub fn distance(&self, point: Vector3<f32>) -> f32 {
        distance(
            Vector3::new(self.origin.x, self.origin.y, self.origin.z),
            point,
        )
    }
    fn local_at(&self, dist: f32) -> Vector3<f32> {
        Vector3::new(
            self.origin[0] + dist * self.direction[0],
            self.origin[1] + dist * self.direction[1],
            self.origin[2] + dist * self.direction[2],
        )
    }
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VoxelMaterial {}
impl Leafable for VoxelMaterial {}
impl VoxelMaterial {
    fn scattering_pdf_fn(_ray_in: Ray, record_in: &HitRecord, scattered_ray: Ray) -> Option<f32> {
        let cosine = record_in.normal.dot(scattered_ray.direction.normalize());
        if cosine < 0.0 {
            None
        } else {
            Some(cosine / f32::PI())
        }
    }
}
impl Material for VoxelMaterial {
    fn name(&self) -> &'static str {
        "Voxel Material"
    }

    fn scatter(&self, ray_in: Ray, record_in: &HitRay) -> Option<ScatterRecord> {
        Some(ScatterRecord {
            specular_ray: None,
            attenuation: RgbColor::new(0.5, 0.5, 0.5),
            pdf: Some(Rc::new(LambertianPDF::new(record_in.normal()))),
            scattering_pdf: Self::scattering_pdf_fn,
        })
    }

    fn scattering_pdf(
        &self,
        ray_in: Ray,
        record_in: &HitRecord,
        scattered_ray: Ray,
    ) -> Option<f32> {
        todo!()
    }
}
impl Hittable for OctTree<VoxelMaterial> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let aabb = self.bounding_box(0., 1.).unwrap();
        if aabb.hit(*ray, t_min, t_max) {
            if let Some(hit_info) = self.trace_ray(*ray) {
                Some(HitRecord::new(
                    ray,
                    hit_info.hit_position,
                    hit_info.normal,
                    hit_info.depth,
                    Point2::new(0.5, 0.5),
                    &self.material,
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    fn bounding_box(&self, _time_0: f32, _time_1: f32) -> Option<Aabb> {
        Some(Aabb {
            minimum: Point3::new(0.0, 0.0, 0.0),
            maximum: Point3::new(self.size as f32, self.size as f32, self.size as f32),
        })
    }

    fn prob(&self, ray: crate::prelude::Ray) -> f32 {
        todo!()
    }

    fn generate_ray_in_area(&self, origin: Point3<f32>, time: f32) -> RayAreaInfo {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_index() {
        let t = OctTreeNode {
            children: OctTreeChildren::Leaf(LeafType::Solid(true)),
            size: 16,
        };
        assert_eq!(t.get_child_index(0, 0, 0), 0);
    }
}
