use image::{Rgb, RgbImage};
#[derive(Clone, Debug)]
struct OctTree {
    root_node: OctTreeNode,
    size: u32,
}
impl OctTree {
    pub fn new_cube() -> Self {
        let size = 2u32.pow(2);
        Self {
            root_node: OctTreeNode {
                children: OctTreeChildren::Leaf(true),
                size,
            },
            size: size,
        }
    }
    /// Creates a sphere with the given radius
    pub fn sphere(radius: u32) -> Self {
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
                    corner[2],
                    center + size - 1,
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
        let size = 2 * get_next_power(radius);
        let center = size / 2;

        println!("size: {}", size);
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
        let image_size = 1024;
        let mut image_render = RgbImage::new(image_size, image_size);

        for x in 0..image_size {
            for y in 0..image_size {
                let x_f = (x as f32 / image_size as f32) * self.size as f32;
                let y_f = (y as f32 / image_size as f32) * self.size as f32;
                let pixel_color = if let Some(depth) =
                    self.trace_xyz(x_f.floor() as u32, y_f.floor() as u32, 0)
                {
                    let c_val = (((self.size - depth) as f32 / self.size as f32) * 255.) as u8;
                    Rgb([c_val, 255, c_val])
                } else {
                    Rgb([0, 0, 0])
                };

                image_render.put_pixel(x, image_size - y - 1, pixel_color);
            }
        }
        image_render.save(path).expect("failed to save");
    }
    pub fn trace_ray(&self, ray: Ray) {}
}
#[derive(Clone, Debug)]
struct OctTreeNode {
    children: OctTreeChildren,
    size: u32,
}
impl OctTreeNode {
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
pub struct Ray {
    pub origin: [f32; 3],
    pub direction: [f32; 3],
}
#[derive(Clone, Debug)]
enum OctTreeChildren {
    Leaf(bool),
    ParentNode(Box<[OctTreeNode; 8]>),
}
fn main() {
    let tree = OctTree::half_cube();
    tree.render("flat_cube.png");
    let sphere = OctTree::sphere(64);
    sphere.render("sphere.png");
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
