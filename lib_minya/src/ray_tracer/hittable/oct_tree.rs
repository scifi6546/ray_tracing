mod material;
mod operations;
mod ray_trace;
mod shapes;

use super::{HitRecord, Hittable, Ray, RayAreaInfo};
use crate::ray_tracer::bvh::Aabb;
pub use material::VoxelMaterial;

use prelude::distance;

use cgmath::{InnerSpace, Point2, Point3, Vector3};

mod prelude {

    use cgmath::{Point3, Vector3};
    pub fn distance(a: Vector3<f32>, b: Vector3<f32>) -> f32 {
        ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
    }
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

    pub(crate) fn get_children_offsets() -> [Point3<u32>; 8] {
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
    /// Gets the next powr of 2 that is closest to the given value, if the value is already a power of 2 it returns the same value
    /// using https://stackoverflow.com/questions/1322510/given-an-integer-how-do-i-find-the-next-largest-power-of-two-using-bit-twiddlin/1322548#1322548
    pub(crate) fn get_next_power(v: u32) -> u32 {
        let mut v1 = v - 1;
        v1 |= v1 >> 1;
        v1 |= v1 >> 2;
        v1 |= v1 >> 4;
        v1 |= v1 >> 8;
        v1 |= v1 >> 16;
        v1 + 1
    }
}
#[derive(Debug)]
pub struct OctTreeHitInfo<'a, T: Leafable> {
    pub hit_value: &'a T,
    pub depth: f32,
    pub hit_position: Point3<f32>,
    pub normal: Vector3<f32>,
}
#[derive(Clone, Debug)]
pub struct OctTree<T: Leafable> {
    root_node: OctTreeNode<T>,
    size: u32,
}
impl<T: Leafable> OctTree<T> {
    fn get_contents(&self, x: u32, y: u32, z: u32) -> LeafType<T> {
        self.root_node.get(x, y, z)
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
    /// gets reference to underlying data
    fn unwrap_ref(&self) -> &T {
        match self {
            Self::Solid(data) => data,
            Self::Empty => panic!("leaf empty"),
        }
    }
}
pub trait Leafable: Clone + Copy + PartialEq {}
impl Leafable for bool {}
impl Leafable for () {}

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

impl Hittable for OctTree<VoxelMaterial> {
    fn hit(&self, ray: &Ray, t_min: f32, t_max: f32) -> Option<HitRecord> {
        let aabb = self.bounding_box(0., 1.).unwrap();
        if aabb.hit(*ray, t_min, t_max) {
            if let Some(hit_info) = self.trace_ray(Ray {
                origin: ray.origin,
                time: ray.time,
                direction: ray.direction.normalize(),
            }) {
                Some(HitRecord::new(
                    ray,
                    hit_info.hit_position,
                    hit_info.normal,
                    hit_info.depth,
                    Point2::new(0.5, 0.5),
                    hit_info.hit_value,
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

    fn prob(&self, _ray: crate::prelude::Ray) -> f32 {
        todo!()
    }

    fn generate_ray_in_area(&self, _origin: Point3<f32>, _time: f32) -> RayAreaInfo {
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
