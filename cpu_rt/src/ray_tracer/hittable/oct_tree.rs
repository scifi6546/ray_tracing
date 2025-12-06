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
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HitType {
    Solid,
    Volume,
    Empty,
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
    fn hit_type(&self) -> HitType;

    fn merge(&self, rhs: &Self) -> Self {
        match self.hit_type() {
            HitType::Solid => *self,
            HitType::Volume => match rhs.hit_type() {
                HitType::Solid => *rhs,
                HitType::Volume => *self,
                HitType::Empty => *self,
            },

            HitType::Empty => match rhs.hit_type() {
                HitType::Solid => *rhs,
                HitType::Volume => *rhs,
                HitType::Empty => Self::empty(),
            },
        }
    }
    fn empty() -> Self;
}
impl Leafable for bool {
    fn hit_type(&self) -> HitType {
        if *self {
            HitType::Solid
        } else {
            HitType::Empty
        }
    }
    fn empty() -> Self {
        false
    }
}
impl Leafable for () {
    fn hit_type(&self) -> HitType {
        HitType::Empty
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
    impl Leafable for i32 {
        fn hit_type(&self) -> HitType {
            match self {
                0 => HitType::Empty,
                1 => HitType::Volume,
                2 => HitType::Solid,
                _ => panic!("invalid leaf: {}", self),
            }
        }
        fn empty() -> Self {
            0
        }
    }
    #[test]
    fn merge() {
        let e = 0;
        let v = 1;
        let s = 2;
        assert_eq!(e.merge(&e), e);
        assert_eq!(e.merge(&v), v);
        assert_eq!(e.merge(&s), s);

        assert_eq!(v.merge(&e), v);
        assert_eq!(v.merge(&v), v);
        assert_eq!(v.merge(&s), s);

        assert_eq!(s.merge(&e), s);
        assert_eq!(s.merge(&v), s);
        assert_eq!(s.merge(&s), s);
    }
}
