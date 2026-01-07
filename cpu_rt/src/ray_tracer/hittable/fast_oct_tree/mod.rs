mod arena;
mod hittable;
use cgmath::Point3;
pub type TreePosition = Point3<u32>;
use arena::{Arena, ArenaIndex};
#[derive(Clone, Debug)]
enum NodeData {
    Parent { children: [ArenaIndex; 8] },
    Leaf(Leaf),
    Empty,
}
#[derive(Clone, Debug)]
struct Node {
    data: NodeData,
    // size is expressed in powers of 2, so if self.size = 2 the voxel size will be 2.pow(2) or 4 meters
    size: u32,
}
impl Node {
    //sets children and returns  a copy of modified version of self
    fn set_child(mut self, value: Leaf, position: TreePosition, arena: &mut Arena<Self>) -> Self {
        if self.size == 0 {
            assert_eq!(position, Point3::new(0, 0, 0));
            self.data = NodeData::Leaf(value);
            self
        } else {
            todo!("set larger data")
        }
    }

    fn get(&self, position: TreePosition) -> Option<Leaf> {
        match self.data {
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
}
#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Leaf {}
#[derive(Clone)]
/// Overall Tree data structure. Utilizes arena to maintain cache locality and to serve as a framework as I migrate towards GPU compute
pub(crate) struct Tree {
    arena: Arena<Node>,
}
impl Tree {
    pub fn new() -> Self {
        Self {
            arena: Arena::new(),
        }
    }
    /// sets the value of the item at leaf. Automatically resizes as needed
    pub fn set(&mut self, value: Leaf, position: TreePosition) {
        if let Some(root) = self.arena.get_root_mut() {
            todo!("set children")
        } else {
            self.arena.insert(Node::empty());
            let root = self.arena.get_root().expect("should exist").clone();
            let new_root = root.set_child(value, position, &mut self.arena);
            self.arena.update_root(new_root);
        }
    }
    pub fn get(&self, position: TreePosition) -> Option<Leaf> {
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
        let _ = Tree::new();
    }
    #[test]
    fn get_empty() {
        let t = Tree::new();
        assert_eq!(t.get(Point3::new(0, 0, 0)), None);
    }
    #[test]
    fn get_and_set() {
        let mut t = Tree::new();
        let l = Leaf {};
        t.set(l, Point3::new(0, 0, 0));
        assert_eq!(t.get(Point3::new(0, 0, 0)).unwrap(), l);
    }
}
