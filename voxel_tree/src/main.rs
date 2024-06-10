use image::{Rgb, RgbImage};
use std::cmp::max;
fn f32_min(a: f32, b: f32) -> f32 {
    if a <= b {
        a
    } else {
        b
    }
}
mod prelude {
    use super::Leafable;
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
    pub fn get_children_offsets() -> [[u32; 3]; 8] {
        [
            [0, 0, 0],
            [0, 0, 1],
            [0, 1, 0],
            [0, 1, 1],
            [1, 0, 0],
            [1, 0, 1],
            [1, 1, 0],
            [1, 1, 1],
        ]
    }
}
#[derive(Clone, Debug)]
struct OctTree<T: Leafable> {
    root_node: OctTreeNode<T>,
    size: u32,
}
impl<T: Leafable> OctTree<T> {
    pub fn cube(hit_val: T) -> Self {
        let size = 2u32.pow(2);
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(LeafType::Solid(hit_val)),
                size,
            },
            size,
        }
    }

    /// fun takes in aabb and returns whether the box collides
    pub fn build_from_fn(
        fun: &dyn Fn([u32; 3], u32) -> CollideResult,
        size: u32,
        hit_value: T,
    ) -> Self {
        fn build_leaf<T: Leafable>(
            fun: &dyn Fn([u32; 3], u32) -> CollideResult,
            offset: [u32; 3],
            size: u32,
            hit_value: T,
        ) -> OctTreeNode<T> {
            if size >= 2 {
                match fun(offset, size - 1) {
                    CollideResult::FullyIn => todo!("fully in"),
                    CollideResult::FullyOut => OctTreeNode {
                        children: OctTreeChildren::Leaf(LeafType::Empty),
                        size,
                    },
                    CollideResult::PartialInOut => {
                        let child_offsets = prelude::get_children_offsets().map(|child| {
                            println!("child: [{},{},{}]", child[0], child[1], child[2]);
                            [
                                child[0] * size / 2 + offset[0],
                                child[1] * size / 2 + offset[1],
                                child[2] * size / 2 + offset[2],
                            ]
                        });
                        OctTreeNode {
                            children: OctTreeChildren::ParentNode(Box::new(
                                child_offsets
                                    .map(|offset| build_leaf(fun, offset, size / 2, hit_value)),
                            )),
                            size: size / 2,
                        }
                    }
                }
            } else {
                OctTreeNode {
                    children: OctTreeChildren::Leaf(match fun(offset, 1) {
                        CollideResult::FullyIn => LeafType::Solid(hit_value),
                        CollideResult::FullyOut => LeafType::Empty,
                        CollideResult::PartialInOut => LeafType::Solid(hit_value),
                    }),
                    size: 1,
                }
            }
        }
        Self {
            root_node: build_leaf(fun, [0, 0, 0], size, hit_value),
            size,
        }
    }
    pub fn cone(radius: u32, height: u32, hit_val: T) -> Self {
        fn cone_point_intercept(
            radius: u32,
            height: u32,
            point: [u32; 3],
            box_size: u32,
        ) -> CollideResult {
            fn distance(point: [f32; 3], center: f32) -> f32 {
                ((point[0] - center).powi(2)
                    + (point[1] - center).powi(2)
                    + (point[2] - center).powi(2))
                .sqrt()
            }
            fn is_fully_inside(radius: u32, height: u32, point: [u32; 3], box_size: u32) -> bool {
                let slope = radius as f32 / height as f32;
                let expected_radius_bottom = (height as f32 - point[1] as f32) * slope;
                let expected_radius_top =
                    (height as f32 - (point[1] as f32 + box_size as f32)) * slope;
                let x0y0z0 = distance(
                    [point[0] as f32, point[1] as f32, point[2] as f32],
                    radius as f32,
                ) <= expected_radius_bottom;
                let x0y0z1 = distance(
                    [
                        point[0] as f32,
                        point[1] as f32,
                        point[2] as f32 + box_size as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_bottom;
                let x0y1z0 = distance(
                    [
                        point[0] as f32,
                        point[1] as f32 + box_size as f32,
                        point[2] as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_top;
                let x0y1z1 = distance(
                    [
                        point[0] as f32,
                        point[1] as f32 + box_size as f32,
                        point[2] as f32 + box_size as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_top;
                // x==1
                let x1y0z0 = distance(
                    [
                        point[0] as f32 + box_size as f32,
                        point[1] as f32,
                        point[2] as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_bottom;
                let x1y0z1 = distance(
                    [
                        point[0] as f32 + box_size as f32,
                        point[1] as f32,
                        point[2] as f32 + box_size as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_bottom;
                let x1y1z0 = distance(
                    [
                        point[0] as f32 + box_size as f32,
                        point[1] as f32 + box_size as f32,
                        point[2] as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_top;
                let x1y1z1 = distance(
                    [
                        point[0] as f32 + box_size as f32,
                        point[1] as f32 + box_size as f32,
                        point[2] as f32 + box_size as f32,
                    ],
                    radius as f32,
                ) <= expected_radius_top;
                x0y0z0 && x0y0z1 && x0y1z0 && x0y1z1 && x1y0z0 && x1y0z1 && x1y1z0 && x1y1z1
            }
            if point[0] <= radius
                && point[0] + box_size >= box_size
                && point[2] <= radius
                && point[2] + box_size >= box_size
            {
                // box surrounds center in some way

                if is_fully_inside(radius, height, point, box_size) {
                    CollideResult::FullyIn
                } else {
                    CollideResult::PartialInOut
                }
            } else {
                if is_fully_inside(radius, height, point, box_size) {
                    todo!("fully inside")
                } else {
                    let slope = radius as f32 / height as f32;

                    let distance_x = if point[0] as f32 <= radius as f32
                        && (point[0] as f32 + box_size as f32) >= radius as f32
                    {
                        f32_min(
                            (point[2] as f32 - radius as f32).abs(),
                            (point[2] as f32 + box_size as f32 - radius as f32).abs(),
                        )
                    } else {
                        f32_min(
                            distance(point.map(|n| n as f32), radius as f32),
                            distance(
                                [
                                    point[0] as f32 + box_size as f32,
                                    point[1] as f32,
                                    point[2] as f32,
                                ],
                                radius as f32,
                            ),
                        )
                    };
                    let distance_z = if point[2] as f32 <= radius as f32
                        && (point[2] as f32 + box_size as f32 >= radius as f32)
                    {
                        f32_min(
                            (point[0] as f32 - radius as f32).abs(),
                            (point[0] as f32 + box_size as f32 - radius as f32).abs(),
                        )
                    } else {
                        f32_min(
                            distance(point.map(|n| n as f32), radius as f32),
                            distance(
                                [
                                    point[0] as f32,
                                    point[1] as f32,
                                    point[2] as f32 + box_size as f32,
                                ],
                                radius as f32,
                            ),
                        )
                    };

                    let radius_top = (height as f32 - point[1] as f32) * slope;
                    let radius_bottom =
                        (height as f32 - (point[1] as f32 + box_size as f32)) * slope;
                    if radius_bottom < f32_min(distance_x, distance_z) {
                        CollideResult::PartialInOut
                    } else {
                        CollideResult::FullyOut
                    }
                }
            }
        }

        Self::build_from_fn(
            &|offset, box_size| cone_point_intercept(radius, height, offset, box_size),
            Self::get_next_power(radius * 2),
            hit_val,
        )
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
            sphere_center: Vector3,
            sphere_radius: f32,
            box_min: Vector3,
            box_max: Vector3,
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
        }
    }
    pub fn combine(self, other: &Self, offset: [i32; 3]) -> Self {
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
        }
    }
    pub fn get_contents(&self, x: u32, y: u32, z: u32) -> LeafType<T> {
        self.root_node.get(x, y, z)
    }

    pub fn render(&self, path: impl AsRef<std::path::Path>) {
        let image_size = 1024 * 4;
        let mut image_render = RgbImage::new(image_size, image_size);

        for x in 0..image_size {
            for y in 0..image_size {
                let x_f = (x as f32 / image_size as f32) * self.size as f32;
                let y_f = (y as f32 / image_size as f32) * self.size as f32;
                let pixel_color = if let Some((depth, color)) = self.trace_ray(Ray {
                    direction: Vector3([0.0, 0.0, 1.0]),
                    origin: Vector3([x_f, y_f, 0.0]),
                }) {
                    let c_val = (depth / self.size as f32 * 255.) as u8;
                    Rgb([c_val, c_val, c_val])
                } else {
                    Rgb([255, 0, 0])
                };

                image_render.put_pixel(x, image_size - y - 1, pixel_color);
            }
        }
        image_render.save(path).expect("failed to save");
    }
    pub fn trace_ray(&self, ray: Ray) -> Option<(f32, T)> {
        self.root_node.trace_ray(ray)
    }
}
enum CollideResult {
    FullyIn,
    FullyOut,
    PartialInOut,
}
impl OctTree<prelude::RgbColor> {
    pub fn render_rgb(&self, path: impl AsRef<std::path::Path>) {
        let image_size = 1024 * 4;
        let mut image_render = RgbImage::new(image_size, image_size);

        for x in 0..image_size {
            for y in 0..image_size {
                let x_f = (x as f32 / image_size as f32) * self.size as f32;
                let y_f = (y as f32 / image_size as f32) * self.size as f32;
                let pixel_color = if let Some((depth, color)) = self.trace_ray(Ray {
                    direction: Vector3([0.0, 0.0, 1.0]),
                    origin: Vector3([x_f, y_f, 0.0]),
                }) {
                    Rgb(color.0.map(|c| (c as f32 * depth / self.size as f32) as u8))
                } else {
                    Rgb([255, 0, 0])
                };

                image_render.put_pixel(x, image_size - y - 1, pixel_color);
            }
        }
        image_render.save(path).expect("failed to save");
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
    pub fn trace_ray(&self, ray: Ray) -> Option<(f32, T)> {
        if ray.origin[0] > self.size as f32 || ray.origin[1] > self.size as f32 {
            println!("larger??, ray: {}", ray.origin);
        }
        // getting the min distances

        match &self.children {
            OctTreeChildren::Leaf(val) => {
                if val.is_solid() {
                    let (axis, time) = (0..3)
                        .map(|idx| {
                            (
                                idx,
                                [
                                    ray.intersect_axis(idx, 0.0),
                                    ray.intersect_axis(idx, self.size as f32),
                                ]
                                .iter()
                                .fold(f32::MAX, |acc, x| {
                                    if acc < *x {
                                        acc
                                    } else {
                                        *x
                                    }
                                }),
                            )
                        })
                        .filter(|(idx, time)| {
                            let pos = ray.at(*time);
                            let pos_good = [
                                *idx == 0 || (pos[0] >= 0. && pos[0] <= self.size as f32),
                                *idx == 1 || (pos[1] >= 0. && pos[1] <= self.size as f32),
                                *idx == 2 || (pos[2] >= 0. && pos[2] <= self.size as f32),
                            ];
                            pos_good[0] && pos_good[1] && pos_good[2]
                        })
                        .filter(|(_idx, time)| ray.distance(ray.at(*time)).is_finite())
                        .fold((4, f32::MAX), |acc, x| if acc.1 < x.1 { acc } else { x });
                    if axis != 4 {
                        let d = ray.distance(ray.at(time));
                        if d.is_infinite() {
                            println!("INFINITE!!!!");
                            println!("time: {}, idx: {}", time, axis);
                            panic!()
                        }

                        Some((d, val.unwrap()))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            OctTreeChildren::ParentNode(children) => {
                let mut intersection = (0..3)
                    .flat_map(|idx| {
                        [
                            ray.intersect_axis(idx, 0.0),
                            ray.intersect_axis(idx, self.size as f32 / 2.0),
                        ]
                        .map(|time| (idx, time, ray.at(time)))
                    })
                    .filter(|(_idx, time, _pos)| time.is_finite())
                    .filter(|(idx, _dist, pos)| {
                        let is_valid = pos.map_arr(|v| v >= 0. && v < self.size as f32);
                        let is_valid = [
                            is_valid[0] || *idx == 0,
                            is_valid[1] || *idx == 1,
                            is_valid[2] || *idx == 2,
                        ];
                        is_valid[0] && is_valid[1] && is_valid[2]
                    })
                    .collect::<Vec<_>>();
                intersection.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
                let tiles = intersection
                    .iter()
                    .map(|(_idx, _dist, pos)| {
                        let [x, y, z] =
                            pos.map_arr(|v| (2.0 * v / self.size as f32).floor() as u32);
                        (Self::get_child_index_size2(x, y, z), pos)
                    })
                    .collect::<Vec<_>>();

                let mut total_distance = 0.0;

                let mut last_pos = ray.origin;
                for (index, pos) in tiles {
                    if let Some((ray_dist, hit_cube)) = children[index].trace_ray(Ray {
                        origin: Vector3([
                            if pos[0] >= self.size as f32 / 2.0 {
                                pos[0] - self.size as f32 / 2.0
                            } else {
                                pos[0]
                            },
                            if pos[1] >= self.size as f32 / 2.0 {
                                pos[1] - self.size as f32 / 2.0
                            } else {
                                pos[1]
                            },
                            if pos[2] >= self.size as f32 / 2.0 {
                                pos[2] - self.size as f32 / 2.0
                            } else {
                                pos[2]
                            },
                        ]),
                        direction: ray.direction,
                    }) {
                        return Some((
                            total_distance + ray_dist + distance(*pos, last_pos),
                            hit_cube,
                        ));
                    } else {
                        total_distance += distance(last_pos, *pos);
                        last_pos = *pos;
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
#[derive(Clone, Copy, Debug)]
pub struct Vector3(pub [f32; 3]);
impl Vector3 {
    pub fn map_arr<T, F: Fn(f32) -> T>(self, f: F) -> [T; 3] {
        [f(self.0[0]), f(self.0[1]), f(self.0[2])]
    }
}
impl std::ops::Mul<f32> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f32) -> Self::Output {
        Self([self.0[0] * rhs, self.0[1] * rhs, self.0[2] * rhs])
    }
}
impl std::fmt::Display for Vector3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{},{}]", self.0[0], self.0[1], self.0[2])
    }
}
impl std::ops::Index<usize> for Vector3 {
    type Output = f32;
    fn index(&self, idx: usize) -> &Self::Output {
        &self.0[idx]
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Ray {
    pub origin: Vector3,
    pub direction: Vector3,
}
fn distance(a: Vector3, b: Vector3) -> f32 {
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
    pub fn distance(&self, point: Vector3) -> f32 {
        distance(self.origin, point)
    }
    pub fn at(&self, dist: f32) -> Vector3 {
        Vector3([
            self.origin[0] + dist * self.direction[0],
            self.origin[1] + dist * self.direction[1],
            self.origin[2] + dist * self.direction[2],
        ])
    }
}

fn main() {
    use prelude::RgbColor;
    /*
    {
        println!("************* CONE *************");
        let cone = OctTree::cone(64, 64, RgbColor([15, 255, 0]));
        cone.render_rgb("cone.png");
    }

     */
    println!("*************CUBE*************");
    let cube = OctTree::cube(RgbColor([0, 255, 0]));
    cube.render_rgb("cube.png");
    println!("*************HALF CUBE*************");
    let tree = OctTree::half_cube(RgbColor([0, 255, 0]));
    tree.render_rgb("flat_cube.png");
    let sphere = OctTree::sphere(64, RgbColor([0, 255, 0]));
    println!("*************SPHERE*************");
    sphere.render_rgb("sphere.png");
    {
        println!("*************SPHERE HUGE*************");
        let sphere = OctTree::sphere(10_000, RgbColor([0, 255, 0]));
        sphere.render_rgb("sphere_huge.png");
    }
    println!("*************COMBINED 1/2*************");
    let combined = OctTree::sphere(32, RgbColor([255, 25, 0]))
        .combine(&OctTree::sphere(20, RgbColor([0, 255, 0])), [32, 32, 32]);
    assert!(combined.is_optimal());
    println!("rendering");
    combined.render("combined_1_2.png");
    println!("*************COMBINED REORDER*************");
    let combined = OctTree::sphere(32, RgbColor([0, 255, 0]))
        .combine(&OctTree::sphere(20, RgbColor([0, 255, 255])), [32, 32, 32])
        .combine(&OctTree::sphere(32, RgbColor([255, 255, 0])), [64, 64, 0]);
    assert!(combined.is_optimal());
    println!("rendering");
    combined.render_rgb("combined_reorder.png");
    println!("*************COMBINED*************");
    let combined = OctTree::sphere(32, RgbColor([0, 255, 0]))
        .combine(&OctTree::sphere(32, RgbColor([0, 255, 0])), [64, 64, 0])
        .combine(&OctTree::sphere(20, RgbColor([0, 255, 0])), [32, 32, 32]);
    println!("rendering");
    combined.render("combined.png");
    assert!(combined.is_optimal());

    {
        println!("************* VERY BIG *************");
        let sphere_radius = 100;
        let big_sphere = OctTree::sphere(sphere_radius, RgbColor([0, 255, 0]));
        let mut tree = OctTree::sphere(4, RgbColor([0, 255, 0]));
        let center = 10_000;
        let z_max = 10;
        for z in 0..z_max {
            println!("z: {}", z);
            let step_size = center / z_max;

            tree = tree
                .combine(&big_sphere, [step_size * z, 0, z * 1000])
                .combine(&big_sphere, [step_size * (z_max - z), 0, z * 1000]);
        }
        println!("rendering");
        tree.render_rgb("very_big.png");
    }
    {
        println!("*************BIG*************");
        let mut tree = OctTree::sphere(4, RgbColor([0, 255, 0]));
        let sphere = OctTree::sphere(20, RgbColor([0, 255, 0]));
        for x in 0..100 {
            println!("x: {}", x);
            for y in 0..100 {
                if x == 0 {
                    println!("x: {}, y: {}", x, y);
                }

                // assert!(sphere.is_optimal());
                tree = tree.combine(&sphere, [x * 10, y * 10, 0]);
                //assert!(tree.is_optimal());
            }
        }
        println!("rendering");
        tree.render_rgb("big.png");
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
