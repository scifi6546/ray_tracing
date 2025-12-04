mod from_voxel_world;
mod hittable;
mod material;
mod node;
mod operations;
mod prelude;
mod ray_trace;
mod shapes;

use crate::prelude::RayScalar;
pub use material::VoxelMaterial;
use node::OctTreeNode;

use cgmath::{Point3, Vector3};

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
    fn get_contents(&self, x: u32, y: u32, z: u32) -> T {
        *self.root_node.get(Point3::new(x, y, z))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum OctTreeChildren<T: Leafable> {
    Leaf(T),
    ParentNode(Box<[OctTreeNode<T>; 8]>),
}

pub trait Leafable: Clone + Copy + PartialEq + Eq {
    fn is_solid(&self) -> bool;
    fn empty() -> Self;
}
impl Leafable for bool {
    fn is_solid(&self) -> bool {
        *self
    }
    fn empty() -> Self {
        false
    }
}
impl Leafable for () {
    fn is_solid(&self) -> bool {
        false
    }
    fn empty() -> Self {
        ()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn get_index() {
        let t = OctTreeNode {
            children: OctTreeChildren::Leaf(true),
            size: 16,
        };
        assert_eq!(t.get_child_index(0, 0, 0), 0);
    }
    #[test]
    fn empty() {
        assert_eq!(bool::empty(), false);
    }
}
