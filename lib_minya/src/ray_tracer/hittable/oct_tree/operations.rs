use super::{
    prelude::{aabb_intersect, get_children_offsets, get_next_power},
    LeafType, Leafable, OctTree, OctTreeChildren, OctTreeNode,
};
use cgmath::Point3;
use std::cmp::max;
impl<T: Leafable> OctTree<T> {
    pub fn combine(self, other: &Self, offset: Point3<i32>) -> Self {
        let offset = [offset.x, offset.y, offset.z];
        let other_size = offset
            .iter()
            .map(|s| s.abs() as u32 + other.size)
            .max()
            .unwrap();

        let size = get_next_power(max(self.size, other_size));
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
            // building aabb for checking if current selection collides
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
            if aabb_intersect(other_min, other_max, node_min, node_max) {
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
                                            let offsets = get_children_offsets();
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
                            let offsets = get_children_offsets();
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
                        let offsets = get_children_offsets();
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
            let a_intersects = aabb_intersect(
                cube_position_i32,
                current_max,
                [0, 0, 0],
                [
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                ],
            );
            let b_intersects = aabb_intersect(
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

        let size = get_next_power(max(self.size, other_size));

        Self {
            root_node: build_nodes(size, &self, other, offset, [0, 0, 0]),
            size,
        }
    }
}
