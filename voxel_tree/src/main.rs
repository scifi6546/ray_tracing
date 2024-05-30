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
    pub fn combine(&self, other: &Self, offset: [i32; 3]) -> Self {
        fn build_nodes(
            size: u32,
            a: &OctTree,
            b: &OctTree,
            offset: [i32; 3],
            // lower left corner of current cube
            cube_position: [u32; 3],
        ) -> OctTreeNode {
            if size >= 2 {
                let x0 = cube_position[0];
                let y0 = cube_position[1];
                let z0 = cube_position[2];

                let x1 = x0 + size / 2;
                let y1 = y0 + size / 2;
                let z1 = z0 + size / 2;

                let cubes = [
                    build_nodes(size / 2, a, b, offset, [x0, y0, z0]),
                    build_nodes(size / 2, a, b, offset, [x0, y0, z1]),
                    build_nodes(size / 2, a, b, offset, [x0, y1, z0]),
                    build_nodes(size / 2, a, b, offset, [x0, y1, z1]),
                    // top x
                    build_nodes(size / 2, a, b, offset, [x1, y0, z0]),
                    build_nodes(size / 2, a, b, offset, [x1, y0, z1]),
                    build_nodes(size / 2, a, b, offset, [x1, y1, z0]),
                    build_nodes(size / 2, a, b, offset, [x1, y1, z1]),
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
                    cube_position[0] as i32 - offset[0],
                    cube_position[1] as i32 - offset[1],
                    cube_position[2] as i32 - offset[2],
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
        }
        let other_size = offset
            .iter()
            .map(|s| s.abs() as u32 + other.size)
            .max()
            .unwrap();

        let size = Self::get_next_power(max(self.size, other_size));
        println!(
            "new size: {}, self size: {}, other size: {}, offset: [{},{},{}]",
            size, self.size, other.size, offset[0], offset[1], offset[2]
        );
        Self {
            root_node: build_nodes(size, self, other, offset, [0, 0, 0]),
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
        let image_size = 100;
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
    println!("*************COMBINED*************");
    let combined = OctTree::sphere(32)
        .combine(&OctTree::sphere(32), [64, 64, 0])
        .combine(&OctTree::sphere(20), [32, 32, 32]);
    println!("rendering");
    combined.render("combined.png");
    {
        println!("*************BIG*************");
        let mut tree = OctTree::sphere(4);
        for x in 0..100 {
            for y in 0..100 {
                tree = tree.combine(&OctTree::sphere(6), [x * 10, 0, y * 10]);
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
