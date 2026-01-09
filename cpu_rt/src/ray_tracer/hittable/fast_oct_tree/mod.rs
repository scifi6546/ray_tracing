mod arena;
mod hit_info;
mod hittable;
mod leafable;
mod ray_trace;
mod voxel;

use cgmath::Point3;
use leafable::Leafable;

pub type IndexType = u32;
pub type TreePosition = Point3<IndexType>;
use arena::{Arena, ArenaIndex};

pub(crate) use voxel::{SolidVoxel, VolumeVoxel, Voxel, VoxelMaterial};

#[derive(Clone, Debug)]
enum NodeData<T: Leafable> {
    Parent { children: [ArenaIndex; 8] },
    Leaf(T),
    Empty,
}
#[derive(Clone, Debug)]
struct Node<T: Leafable> {
    data: NodeData<T>,
    // size is expressed in powers of 2, so if self.size = 2 the voxel size will be 2.pow(2) or 4 meters
    size: u32,
}
impl<T: Leafable> Node<T> {
    //sets children and returns  a copy of modified version of self
    fn set_child(mut self, value: T, position: TreePosition, arena: &mut Arena<Self>) -> Self {
        if self.size == 0 {
            assert_eq!(position, Point3::new(0, 0, 0));
            self.data = NodeData::Leaf(value);
            self
        } else {
            todo!("set larger data")
        }
    }

    fn get(&self, position: TreePosition) -> Option<&T> {
        match &self.data {
            NodeData::Empty => todo!("empty"),
            NodeData::Leaf(leaf) => Some(leaf),
            NodeData::Parent { children } => todo!("parent"),
        }
    }
    const fn empty() -> Self {
        Self {
            data: NodeData::Empty,
            size: 0,
        }
    }
    fn get_world_size(&self) -> IndexType {
        1 << self.size
    }
}

#[derive(Clone)]
/// Overall Tree data structure. Utilizes arena to maintain cache locality and to serve as a framework as I migrate towards GPU compute
pub(crate) struct FastOctTree<T: Leafable> {
    arena: Arena<Node<T>>,
}
impl<T: Leafable> FastOctTree<T> {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
    /// gets the size of the node in world units
    fn world_size(&self) -> IndexType {
        if let Some(root) = self.arena.get_root() {
            root.get_world_size()
        } else {
            0
        }
    }
    /// sets the value of the item at leaf. Automatically resizes as needed
    pub fn set(&mut self, value: T, position: TreePosition) {
        if let Some(root) = self.arena.get_root_mut() {
            todo!("set children")
        } else {
            self.arena.insert(Node::empty());
            let root = self.arena.get_root().expect("should exist").clone();
            let new_root = root.set_child(value, position, &mut self.arena);
            self.arena.update_root(new_root);
        }
    }
    pub fn get(&self, position: TreePosition) -> Option<&T> {
        if let Some(r) = self.arena.get_root() {
            r.get(position)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn new() {
        let _ = FastOctTree::<u32>::new();
    }
    #[test]
    fn get_empty() {
        let t = FastOctTree::<u32>::new();
        assert_eq!(t.get(Point3::new(0, 0, 0)), None);
    }
    #[test]
    fn get_and_set() {
        let mut t = FastOctTree::<u32>::new();
        let l = 0;
        t.set(l, Point3::new(0, 0, 0));
        assert_eq!(*t.get(Point3::new(0, 0, 0)).unwrap(), 0);
    }
    #[test]
    fn size_empty() {
        let t = FastOctTree::<u32>::new();
        assert_eq!(t.world_size(), 0)
    }
    #[test]
    fn size_one() {
        let mut t = FastOctTree::new();
        t.set(0u32, Point3::new(0, 0, 0));
        assert_eq!(t.world_size(), 1)
    }
}
