mod from_voxel_world;
mod hittable;
mod material;
mod node;
mod operations;
mod prelude;
mod ray_trace;
mod shapes;

use super::{HitRecord, Hittable, Ray, RayAreaInfo};
use crate::{prelude::RayScalar, ray_tracer::bvh::Aabb};
pub use material::VoxelMaterial;
use node::OctTreeNode;

use prelude::distance;

use cgmath::{InnerSpace, Point2, Point3, Vector3};

#[derive(Debug)]
pub struct OctTreeHitInfo<'a, T: Leafable> {
    pub hit_value: &'a T,
    pub depth: RayScalar,
    pub hit_position: Point3<RayScalar>,
    pub normal: Vector3<RayScalar>,
}
#[derive(Clone)]
pub struct OctTree<T: Leafable> {
    pub(crate) root_node: OctTreeNode<T>,
    pub(crate) size: u32,
}
impl<T: Leafable> OctTree<T> {
    fn get_contents(&self, x: u32, y: u32, z: u32) -> LeafType<T> {
        *self.root_node.get(Point3::new(x, y, z))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum OctTreeChildren<T: Leafable> {
    Leaf(LeafType<T>),
    ParentNode(Box<[OctTreeNode<T>; 8]>),
}
/// Leaf of tree
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum LeafType<T: Leafable> {
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
}
pub trait Leafable: Clone + Copy + PartialEq {}
impl Leafable for bool {}
impl Leafable for () {}

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
