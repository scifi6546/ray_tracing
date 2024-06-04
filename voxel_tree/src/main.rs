use image::{Rgb, RgbImage};
use std::cmp::max;
#[derive(Clone, Debug)]
struct OctTree {
    root_node: OctTreeNode,
    size: u32,
}
impl OctTree {
    pub fn cube() -> Self {
        let size = 2u32.pow(2);
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(true),
                size,
            },
            size,
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
    pub fn sphere(radius: u32) -> Self {
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
        fn handle_leaf(size: u32, radius: u32, center: u32, corner: [u32; 3]) -> OctTreeNode {
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
                        children: OctTreeChildren::Leaf(d0),
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
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1], corner[2] + size / 2],
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1] + size / 2, corner[2]],
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0], corner[1] + size / 2, corner[2] + size / 2],
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1], corner[2]],
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1], corner[2] + size / 2],
                            ),
                            handle_leaf(
                                size / 2,
                                radius,
                                center,
                                [corner[0] + size / 2, corner[1] + size / 2, corner[2]],
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
                            ),
                        ])),
                        size,
                    }
                }
            } else {
                OctTreeNode {
                    children: OctTreeChildren::Leaf(
                        distance(corner[0], corner[1], corner[2], center) <= (radius as f32),
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
                        handle_leaf(size / 2, radius, center, [0, 0, 0]),
                        handle_leaf(size / 2, radius, center, [0, 0, size / 2]),
                        handle_leaf(size / 2, radius, center, [0, size / 2, 0]),
                        handle_leaf(size / 2, radius, center, [0, size / 2, size / 2]),
                        handle_leaf(size / 2, radius, center, [size / 2, 0, 0]),
                        handle_leaf(size / 2, radius, center, [size / 2, 0, size / 2]),
                        handle_leaf(size / 2, radius, center, [size / 2, size / 2, 0]),
                        handle_leaf(size / 2, radius, center, [size / 2, size / 2, size / 2]),
                    ])),
                    size,
                },
                size,
            }
        } else {
            Self {
                root_node: OctTreeNode {
                    children: OctTreeChildren::Leaf(true),
                    size,
                },
                size,
            }
        }
    }
    pub fn half_cube() -> Self {
        let size = 32;
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::ParentNode(Box::new([
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(true),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(true),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(false),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(false),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(true),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(true),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(false),
                        size: size / 2,
                    },
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(false),
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
    fn get_children_offsets() -> [[u32; 3]; 8] {
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
    // from https://gdbooks.gitbooks.io/3dcollisions/content/Chapter2/static_aabb_aabb.html
    fn aabb_intersect(a_min: [i32; 3], a_max: [i32; 3], b_min: [i32; 3], b_max: [i32; 3]) -> bool {
        (a_min[0] <= b_max[0] && a_max[0] >= b_min[0])
            && (a_min[1] <= b_max[1] && a_max[1] >= b_min[1])
            && (a_min[2] <= b_max[2] && a_max[2] >= b_min[2])
    }
    fn combine_no_resize(self, other: &Self, offset: [i32; 3]) -> Self {
        /// checks if AABB a is fully inside of b
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
        fn try_simplify(nodes: [OctTreeNode; 8]) -> OctTreeChildren {
            let first_value = match &nodes[0].children {
                OctTreeChildren::Leaf(leaf_value) => *leaf_value,
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
        fn modify_node(
            node: OctTreeNode,
            node_offset: [i32; 3],
            other: &OctTree,
            other_offset: [i32; 3],
        ) -> OctTreeNode {
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
            if OctTree::aabb_intersect(other_min, other_max, node_min, node_max) {
                match node.children {
                    OctTreeChildren::Leaf(v) => {
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

                            let mut val = v;
                            if node.size == 1 {
                                return OctTreeNode {
                                    children: OctTreeChildren::Leaf(
                                        v || other.get_contents(
                                            start[0] as u32,
                                            start[1] as u32,
                                            start[2] as u32,
                                        ),
                                    ),
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
                                            let offsets = OctTree::get_children_offsets();
                                            let children = offsets.map(|offset| {
                                                modify_node(
                                                    OctTreeNode {
                                                        children: OctTreeChildren::Leaf(v),
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
                                            let mut val: Option<bool> = None;
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
                                            val = val || get_val;
                                        }
                                    }
                                }
                            }
                            OctTreeNode {
                                children: OctTreeChildren::Leaf(val),
                                size: node.size,
                            }
                        } else {
                            let offsets = OctTree::get_children_offsets();
                            let mut nodes = [
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                                OctTreeNode {
                                    children: OctTreeChildren::Leaf(true),
                                    size: 0,
                                },
                            ];
                            let mut val: Option<bool> = Some(v);

                            for (i, offset) in offsets.iter().enumerate() {
                                let node = modify_node(
                                    OctTreeNode {
                                        children: OctTreeChildren::Leaf(v),
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
                                        OctTreeChildren::ParentNode(v) => val = None,
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
                        let offsets = OctTree::get_children_offsets();
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
        fn build_nodes(
            size: u32,
            a: &OctTree,
            b: &OctTree,
            b_offset: [i32; 3],
            // lower left corner of current cube
            cube_position: [u32; 3],
        ) -> OctTreeNode {
            let cube_position_i32 = cube_position.map(|d| d as i32);
            let current_max = cube_position.map(|d| d as i32 + size as i32 - 1);
            let a_intersects = OctTree::aabb_intersect(
                cube_position_i32,
                current_max,
                [0, 0, 0],
                [
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                ],
            );
            let b_intersects = OctTree::aabb_intersect(
                cube_position_i32,
                current_max,
                b_offset,
                b_offset.map(|p| p + b.size as i32 - 1),
            );
            if OctTree::aabb_intersect(
                cube_position_i32,
                current_max,
                [0, 0, 0],
                [
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                    0 + a.size as i32 - 1,
                ],
            ) || OctTree::aabb_intersect(
                cube_position_i32,
                current_max,
                b_offset,
                b_offset.map(|p| p + b.size as i32 - 1),
            ) {
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
                    let mut cube_val: Option<bool> = None;
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
                        false
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
                        false
                    };
                    OctTreeNode {
                        children: OctTreeChildren::Leaf(a_val || b_val),
                        size,
                    }
                }
            } else {
                OctTreeNode {
                    children: OctTreeChildren::Leaf(false),
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
    pub fn get_contents(&self, x: u32, y: u32, z: u32) -> bool {
        self.root_node.get(x, y, z)
    }
    fn trace_xyz(&self, x: u32, y: u32, z: u32) -> Option<u32> {
        if z < self.size {
            if self.get_contents(x, y, z) {
                Some(z)
            } else {
                self.trace_xyz(x, y, z + 1)
            }
        } else {
            None
        }
    }
    pub fn render(&self, path: impl AsRef<std::path::Path>) {
        let image_size = 1024 * 4;
        let mut image_render = RgbImage::new(image_size, image_size);

        for x in 0..image_size {
            for y in 0..image_size {
                let x_f = (x as f32 / image_size as f32) * self.size as f32;
                let y_f = (y as f32 / image_size as f32) * self.size as f32;
                let pixel_color = if let Some(depth) = self.trace_ray(Ray {
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
    pub fn trace_ray(&self, ray: Ray) -> Option<f32> {
        self.root_node.trace_ray(ray)
    }
}
#[derive(Clone, Debug)]
struct OctTreeNode {
    children: OctTreeChildren,
    size: u32,
}
impl OctTreeNode {
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
    pub fn trace_ray(&self, ray: Ray) -> Option<f32> {
        if ray.origin[0] > self.size as f32 || ray.origin[1] > self.size as f32 {
            println!("larger??, ray: {}", ray.origin);
        }
        // getting the min distances

        match &self.children {
            OctTreeChildren::Leaf(val) => {
                if *val {
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
                        .filter(|(idx, time)| ray.distance(ray.at(*time)).is_finite())
                        .fold((4, f32::MAX), |acc, x| if acc.1 < x.1 { acc } else { x });
                    if axis != 4 {
                        let d = ray.distance(ray.at(time));
                        if d.is_infinite() {
                            println!("INFINITE!!!!");
                            println!("time: {}, idx: {}", time, axis);
                            panic!()
                        }

                        Some(d)
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
                    .map(|(_idx, dist, pos)| {
                        let [x, y, z] =
                            pos.map_arr(|v| (2.0 * v / self.size as f32).floor() as u32);
                        (Self::get_child_index_size2(x, y, z), pos)
                    })
                    .collect::<Vec<_>>();

                let mut total_distance = 0.0;

                let mut last_pos = ray.origin;
                for (index, pos) in tiles {
                    if let Some(ray_dist) = children[index].trace_ray(Ray {
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
                        return Some(total_distance + ray_dist + distance(*pos, last_pos));
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
    pub fn get(&self, x: u32, y: u32, z: u32) -> bool {
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
#[derive(Clone, Copy, Debug)]
struct Vector3([f32; 3]);
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
#[derive(Clone, Debug)]
enum OctTreeChildren {
    Leaf(bool),
    ParentNode(Box<[OctTreeNode; 8]>),
}
fn main() {
    println!("*************CUBE*************");
    let cube = OctTree::cube();
    cube.render("cube.png");
    println!("*************HALF CUBE*************");
    let tree = OctTree::half_cube();
    tree.render("flat_cube.png");
    let sphere = OctTree::sphere(64);
    println!("*************SPHERE*************");
    sphere.render("sphere.png");
    println!("*************COMBINED 1/2*************");
    let combined = OctTree::sphere(32).combine(&OctTree::sphere(20), [32, 32, 32]);
    assert!(combined.is_optimal());
    println!("rendering");
    combined.render("combined_1_2.png");
    println!("*************COMBINED REORDER*************");
    let combined = OctTree::sphere(32)
        .combine(&OctTree::sphere(20), [32, 32, 32])
        .combine(&OctTree::sphere(32), [64, 64, 0]);
    assert!(combined.is_optimal());
    println!("rendering");
    combined.render("combined_reorder.png");
    println!("*************COMBINED*************");
    let combined = OctTree::sphere(32)
        .combine(&OctTree::sphere(32), [64, 64, 0])
        .combine(&OctTree::sphere(20), [32, 32, 32]);
    println!("rendering");
    combined.render("combined.png");
    assert!(combined.is_optimal());
    {
        println!("************* VERY BIG *************");
        let sphere_radius = 100;
        let big_sphere = OctTree::sphere(sphere_radius);
        let mut tree = OctTree::sphere(4);
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
        tree.render("very_big.png");
    }
    {
        println!("*************BIG*************");
        let mut tree = OctTree::sphere(4);
        let sphere = OctTree::sphere(20);
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
        tree.render("big.png");
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_index() {
        let t = OctTreeNode {
            children: OctTreeChildren::Leaf(false),
            size: 16,
        };
        assert_eq!(t.get_child_index(0, 0, 0), 0);
    }
}
